// Javier: Captura de input para Windows usando hooks de bajo nivel.
// Patrón idéntico a macos.rs: CapturedEvent + start_capture + capture_loop.
//
// WH_MOUSE_LL intercepta el ratón antes de que llegue a ninguna app.
// WH_KEYBOARD_LL intercepta el teclado de la misma forma.
// Equivalente al CGEventTap de macOS.
//
// No se necesitan permisos especiales en Windows.
// En algunos sistemas puede requerir ejecutar como Administrador.

use crate::input::protocol::{InputEvent, MouseButton, Modifiers};
use anyhow::Result;
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, GetSystemMetrics,
    SetWindowsHookExW, UnhookWindowsHookEx,
    HHOOK, KBDLLHOOKSTRUCT, MSG, MSLLHOOKSTRUCT,
    SM_CXSCREEN, SM_CYSCREEN,
    WH_KEYBOARD_LL, WH_MOUSE_LL,
    WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MOUSEMOVE, WM_MOUSEWHEEL,
    WM_RBUTTONDOWN, WM_RBUTTONUP,
    WM_SYSKEYDOWN, WM_SYSKEYUP,
};

/// Evento capturado + decisión (mismo patrón que macos.rs)
pub enum CapturedEvent {
    /// El ratón llegó al borde — cambiar al PC remoto
    EdgeReached,
    /// Evento para enviar al PC remoto (modo SENDER activo)
    Forward(InputEvent),
    /// Hotkey Ctrl+Alt+Tab — volver al PC local
    HotkeyPressed,
}

/// Arrancar la captura de input en Windows (modo SENDER).
/// `event_tx`: canal donde se publican los eventos capturados.
/// `edge_side`: "right" | "left" | "top" | "bottom".
/// `running`: se pone a false cuando el servicio quiere parar.
pub fn start_capture(
    event_tx: Sender<CapturedEvent>,
    edge_side: String,
    running: Arc<AtomicBool>,
) {
    std::thread::spawn(move || {
        if let Err(e) = capture_loop(event_tx, edge_side, running) {
            tracing::error!("Error en captura Windows: {}", e);
        }
    });
}

// ─── Estado thread-local para los callbacks (no pueden capturar variables) ───

struct HookContext {
    tx: Sender<CapturedEvent>,
    edge_side: String,
    screen_width: f64,
    screen_height: f64,
    edge_threshold: f64,
    /// true = MODO SENDER (reenviar al peer); false = MODO LOCAL (vigilar borde)
    forwarding: Cell<bool>,
}

// SAFETY: HookContext vive en un thread_local — solo se accede desde el
// hilo del bucle de mensajes donde se instalaron los hooks.
unsafe impl Send for HookContext {}

thread_local! {
    static HOOK_CTX: std::cell::RefCell<Option<HookContext>> =
        std::cell::RefCell::new(None);
}

// VK codes para modificadores y Tab (Win32 Virtual-Key Codes)
const VK_LCONTROL: i32 = 0xA2;
const VK_RCONTROL: i32 = 0xA3;
const VK_LMENU: i32    = 0xA4; // Alt izquierdo
const VK_RMENU: i32    = 0xA5; // Alt derecho
const VK_LSHIFT: i32   = 0xA0;
const VK_RSHIFT: i32   = 0xA1;
const VK_LWIN: i32     = 0x5B;
const VK_RWIN: i32     = 0x5C;
const VK_TAB: u32      = 0x09;

/// Comprueba si una tecla virtual está pulsada en este instante.
/// Bit 15 del valor retornado por GetKeyState indica si está pulsada.
fn key_down(vk: i32) -> bool {
    unsafe { (GetKeyState(vk) as u16 & 0x8000) != 0 }
}

fn capture_loop(
    event_tx: Sender<CapturedEvent>,
    edge_side: String,
    running: Arc<AtomicBool>,
) -> Result<()> {
    let screen_width  = unsafe { GetSystemMetrics(SM_CXSCREEN) as f64 };
    let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) as f64 };
    let edge_threshold = 2.0;

    tracing::info!(
        "📺 Pantalla Windows: {}x{} — borde activo: {}",
        screen_width, screen_height, edge_side
    );

    // Instalar contexto thread-local (los callbacks lo leerán)
    HOOK_CTX.with(|ctx| {
        *ctx.borrow_mut() = Some(HookContext {
            tx: event_tx,
            edge_side,
            screen_width,
            screen_height,
            edge_threshold,
            forwarding: Cell::new(false),
        });
    });

    // Instalar los dos hooks de bajo nivel
    let mouse_hook = unsafe {
        SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), None, 0)?
    };
    let kb_hook = unsafe {
        SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0)?
    };

    tracing::info!("✅ Hooks WH_MOUSE_LL + WH_KEYBOARD_LL instalados — capturando input");

    // Bucle de mensajes (requerido para que los LL hooks funcionen en este hilo)
    let mut msg = MSG::default();
    loop {
        if !running.load(Ordering::Relaxed) {
            break;
        }
        let ret = unsafe { GetMessageW(&mut msg, None, 0, 0) };
        if ret.0 <= 0 {
            // 0 = WM_QUIT, -1 = error
            break;
        }
        unsafe { DispatchMessageW(&msg) };
    }

    // Desinstalar hooks al salir
    unsafe {
        let _ = UnhookWindowsHookEx(mouse_hook);
        let _ = UnhookWindowsHookEx(kb_hook);
    }

    // Limpiar contexto thread-local
    HOOK_CTX.with(|ctx| {
        *ctx.borrow_mut() = None;
    });

    tracing::info!("🛑 Captura Windows detenida");
    Ok(())
}

/// Callback del hook de ratón (WH_MOUSE_LL).
/// Javier: Se llama por cada evento de ratón antes de que llegue a ninguna app.
/// Retorna LRESULT(1) para consumirlo, o CallNextHookEx para dejarlo pasar.
unsafe extern "system" fn mouse_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code < 0 {
        return unsafe { CallNextHookEx(HHOOK(0), code, w_param, l_param) };
    }

    let ms = unsafe { &*(l_param.0 as *const MSLLHOOKSTRUCT) };
    let wm = w_param.0 as u32;

    let consumed = HOOK_CTX.with(|ctx| {
        let borrow = ctx.borrow();
        let Some(ref c) = *borrow else { return false };

        if c.forwarding.get() {
            // ── MODO SENDER: reenviar eventos al PC remoto ────────────────────
            let maybe_ev = if wm == WM_MOUSEMOVE {
                // Coordenadas absolutas en píxeles de la pantalla Windows.
                // El receptor (macOS) las usa directamente para posicionar el cursor.
                Some(InputEvent::MouseMove {
                    dx: ms.pt.x as f64,
                    dy: ms.pt.y as f64,
                })
            } else if wm == WM_LBUTTONDOWN {
                Some(InputEvent::MouseButton { button: MouseButton::Left,  pressed: true  })
            } else if wm == WM_LBUTTONUP {
                Some(InputEvent::MouseButton { button: MouseButton::Left,  pressed: false })
            } else if wm == WM_RBUTTONDOWN {
                Some(InputEvent::MouseButton { button: MouseButton::Right, pressed: true  })
            } else if wm == WM_RBUTTONUP {
                Some(InputEvent::MouseButton { button: MouseButton::Right, pressed: false })
            } else if wm == WM_MOUSEWHEEL {
                // mouseData high word = delta firmado (120 = un tick de rueda)
                // Nota: en windows-rs 0.58, mouseData puede ser MOUSEHOOKSTRUCTEX_MOUSE_DATA(u32)
                let raw = ms.mouseData.0;
                let delta = ((raw >> 16) as u16 as i16) as f64 / 120.0;
                Some(InputEvent::MouseScroll { dx: 0.0, dy: delta })
            } else {
                None
            };

            if let Some(ev) = maybe_ev {
                let _ = c.tx.try_send(CapturedEvent::Forward(ev));
            }
            true // consumir: no llega a ninguna app local
        } else {
            // ── MODO LOCAL: vigilar el borde de pantalla ──────────────────────
            if wm == WM_MOUSEMOVE {
                let x = ms.pt.x as f64;
                let y = ms.pt.y as f64;
                let at_edge = match c.edge_side.as_str() {
                    "right"  => x >= c.screen_width  - c.edge_threshold,
                    "left"   => x <= c.edge_threshold,
                    "top"    => y <= c.edge_threshold,
                    "bottom" => y >= c.screen_height - c.edge_threshold,
                    _        => false,
                };

                if at_edge {
                    tracing::info!("🔀 Borde alcanzado en Windows — cambiando al PC remoto");
                    c.forwarding.set(true);
                    let _ = c.tx.try_send(CapturedEvent::EdgeReached);
                    return true; // consumir el movimiento de borde
                }
            }
            false // dejar pasar al sistema
        }
    });

    if consumed {
        LRESULT(1)
    } else {
        unsafe { CallNextHookEx(HHOOK(0), code, w_param, l_param) }
    }
}

/// Callback del hook de teclado (WH_KEYBOARD_LL).
/// Javier: Se llama por cada tecla antes de que llegue a ninguna app.
unsafe extern "system" fn keyboard_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code < 0 {
        return unsafe { CallNextHookEx(HHOOK(0), code, w_param, l_param) };
    }

    let kb = unsafe { &*(l_param.0 as *const KBDLLHOOKSTRUCT) };
    let wm = w_param.0 as u32;
    let is_down = wm == WM_KEYDOWN || wm == WM_SYSKEYDOWN;

    let consumed = HOOK_CTX.with(|ctx| {
        let borrow = ctx.borrow();
        let Some(ref c) = *borrow else { return false };

        // ── Hotkey Ctrl+Alt+Tab → volver al PC local ──────────────────────────
        if is_down && kb.vkCode == VK_TAB {
            let ctrl = key_down(VK_LCONTROL) || key_down(VK_RCONTROL);
            let alt  = key_down(VK_LMENU)    || key_down(VK_RMENU);
            if ctrl && alt {
                c.forwarding.set(false);
                let _ = c.tx.try_send(CapturedEvent::HotkeyPressed);
                return true; // consumir la hotkey
            }
        }

        if c.forwarding.get() {
            // ── MODO SENDER: reenviar tecla al PC remoto ───────────────────────
            let modifiers = Modifiers {
                ctrl:  key_down(VK_LCONTROL) || key_down(VK_RCONTROL),
                alt:   key_down(VK_LMENU)    || key_down(VK_RMENU),
                shift: key_down(VK_LSHIFT)   || key_down(VK_RSHIFT),
                meta:  key_down(VK_LWIN)     || key_down(VK_RWIN),
            };
            let ev = InputEvent::KeyPress {
                key_code: kb.vkCode,
                pressed: is_down,
                modifiers,
            };
            let _ = c.tx.try_send(CapturedEvent::Forward(ev));
            true // consumir: no llega a ninguna app local
        } else {
            false // dejar pasar al sistema
        }
    });

    if consumed {
        LRESULT(1)
    } else {
        unsafe { CallNextHookEx(HHOOK(0), code, w_param, l_param) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_down_non_pressed_key_returns_false() {
        // VK_F24 (0x87) rara vez está pulsada — verificar que key_down no paniquea
        let result = key_down(0x87);
        assert!(!result, "F24 no debería estar pulsada en este contexto");
    }
}
