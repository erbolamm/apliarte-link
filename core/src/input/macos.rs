// Javier: Implementación de input para macOS.
// Usamos CGEvent — la API oficial de Apple para capturar e inyectar teclado y ratón.
//
// CAPTURA (modo SENDER): CGEventTap intercepta todos los eventos antes de que
// lleguen a las apps. Cuando el ratón llega al borde de la pantalla, los
// redirigimos al PC remoto en lugar de procesarlos localmente.
//
// INYECCIÓN (modo RECEIVER): CGEvent crea eventos sintéticos que macOS trata
// como si vinieran de un teclado/ratón físico.
//
// Permisos necesarios en macOS:
//   Sistema → Privacidad y Seguridad → Accesibilidad → apliarte-switch ✅
//   Sistema → Privacidad y Seguridad → Monitor de Teclado → apliarte-switch ✅

use crate::input::protocol::{InputEvent, MouseButton, Modifiers};
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

// ─── INYECCIÓN ───────────────────────────────────────────────────────────────

/// Inyecta un evento de input en macOS (modo RECEIVER)
/// Javier: "inyectar" = decirle a macOS que el ratón/teclado hizo algo,
/// aunque físicamente no se haya movido nada
pub async fn inject(event: InputEvent) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        inject_sync(event)
    }).await??;
    Ok(())
}

fn inject_sync(event: InputEvent) -> Result<()> {
    use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
    use core_graphics::geometry::CGPoint;

    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| anyhow::anyhow!("No se pudo crear CGEventSource"))?;

    match event {
        InputEvent::MouseMove { dx, dy } => {
            // Javier: Para el MVP usamos coordenadas absolutas.
            // dx/dy vienen del PC remoto ya como posición absoluta.
            let evt = CGEvent::new_mouse_event(
                source,
                CGEventType::MouseMoved,
                CGPoint::new(dx, dy),
                CGMouseButton::Left,
            ).map_err(|_| anyhow::anyhow!("Error creando evento de ratón"))?;
            evt.post(CGEventTapLocation::HID);
        }

        InputEvent::MouseButton { button, pressed } => {
            let btn = match button {
                MouseButton::Left => CGMouseButton::Left,
                MouseButton::Right => CGMouseButton::Right,
                MouseButton::Middle => CGMouseButton::Center,
                _ => CGMouseButton::Left,
            };
            let event_type = match (&button, pressed) {
                (MouseButton::Left, true) => CGEventType::LeftMouseDown,
                (MouseButton::Left, false) => CGEventType::LeftMouseUp,
                (MouseButton::Right, true) => CGEventType::RightMouseDown,
                (MouseButton::Right, false) => CGEventType::RightMouseUp,
                _ => CGEventType::OtherMouseDown,
            };
            let evt = CGEvent::new_mouse_event(
                source,
                event_type,
                CGPoint::new(0.0, 0.0),
                btn,
            ).map_err(|_| anyhow::anyhow!("Error creando evento de click"))?;
            evt.post(CGEventTapLocation::HID);
        }

        InputEvent::MouseScroll { dx: _, dy: _ } => {
            // TODO Fase 2: implementar scroll con CGEventCreateScrollWheelEvent
            tracing::debug!("Scroll recibido (pendiente implementar)");
        }

        InputEvent::KeyPress { key_code, pressed, modifiers: _ } => {
            let evt = CGEvent::new_keyboard_event(
                source,
                key_code as u16,
                pressed,
            ).map_err(|_| anyhow::anyhow!("Error creando evento de teclado"))?;
            evt.post(CGEventTapLocation::HID);
        }
    }

    Ok(())
}

// ─── CAPTURA ─────────────────────────────────────────────────────────────────

/// Arranca la captura de input en macOS (modo SENDER).
/// Javier: Esta función intercepta TODO lo que haces con el teclado y el ratón.
/// Cuando el ratón llega al borde configurado, manda los eventos al canal
/// en vez de dejarlos pasar al sistema operativo local.
///
/// IMPORTANTE: Necesita permisos de Accesibilidad en System Preferences.
/// Si no los tiene, CGEventTap falla silenciosamente.
pub fn start_capture(
    event_tx: Sender<CapturedEvent>,
    edge_side: String,
    running: Arc<AtomicBool>,
) {
    // Javier: spawn_blocking porque CFRunLoop bloquea el hilo — es una API C clásica.
    // No es async, así que la metemos en un hilo dedicado.
    std::thread::spawn(move || {
        if let Err(e) = capture_loop(event_tx, edge_side, running) {
            tracing::error!("Error en captura macOS: {}", e);
        }
    });
}

/// Evento capturado + decisión de si bloquearlo o dejarlo pasar
pub enum CapturedEvent {
    /// El ratón llegó al borde — cambiar al PC remoto
    EdgeReached,
    /// Evento que hay que enviar al PC remoto (estamos en modo SENDER)
    Forward(InputEvent),
    /// Hotkey pulsada — cambiar de vuelta al PC local
    HotkeyPressed,
}

fn capture_loop(
    event_tx: Sender<CapturedEvent>,
    edge_side: String,
    running: Arc<AtomicBool>,
) -> Result<()> {
    use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop};
    use core_graphics::display::CGDisplay;
    use core_graphics::event::{
        CGEventFlags, CGEventTap, CGEventTapLocation,
        CGEventTapOptions, CGEventTapPlacement, CGEventType,
    };
    use std::cell::Cell;

    // Obtener dimensiones de la pantalla principal
    let display = CGDisplay::main();
    let screen_width = display.pixels_wide() as f64;
    let screen_height = display.pixels_high() as f64;
    let edge_threshold = 2.0;

    tracing::info!(
        "📺 Pantalla principal: {}x{} — borde activo: {}",
        screen_width, screen_height, edge_side
    );

    // Tipos de eventos que queremos interceptar.
    // Javier: CGEventTap::new en core-graphics 0.24 recibe Vec<CGEventType>,
    // no una máscara bitmask — la crate lo convierte internamente.
    let events = vec![
        CGEventType::MouseMoved,
        CGEventType::LeftMouseDown,
        CGEventType::LeftMouseUp,
        CGEventType::RightMouseDown,
        CGEventType::RightMouseUp,
        CGEventType::KeyDown,
        CGEventType::KeyUp,
        CGEventType::ScrollWheel,
    ];

    // Estado compartido con el callback.
    // IMPORTANTE: CGEventTap::new requiere Fn (no FnMut), así que usamos
    // Cell<bool> para mutabilidad interior sin RefCell ni Mutex.
    // Javier: Cell<bool> es el equivalente de un campo mutable en un closure
    // inmutable. get() lee, set() escribe — sin bloqueos, solo en el mismo hilo.
    struct CallbackState {
        tx: Sender<CapturedEvent>,
        edge_side: String,
        screen_width: f64,
        screen_height: f64,
        edge_threshold: f64,
        forwarding: Cell<bool>,
    }

    let state = Box::new(CallbackState {
        tx: event_tx,
        edge_side,
        screen_width,
        screen_height,
        edge_threshold,
        forwarding: Cell::new(false),
    });

    // Javier: CGEventTap::new crea el gancho a nivel de sistema.
    // El closure recibe CADA evento antes de que llegue a cualquier app.
    // Devolver None = bloquar el evento; Some(event) = dejarlo pasar.
    let tap = CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        events,
        move |_proxy, event_type, event| {
            // ── Hotkey Ctrl+Alt+Tab → volver al PC local ────────────────────
            if matches!(event_type, CGEventType::KeyDown) {
                let flags = event.get_flags();
                let keycode = event.get_integer_value_field(
                    core_graphics::event::EventField::KEYBOARD_EVENT_KEYCODE,
                ) as u16;

                let is_ctrl = flags.contains(CGEventFlags::CGEventFlagControl);
                let is_alt = flags.contains(CGEventFlags::CGEventFlagAlternate);

                if is_ctrl && is_alt && keycode == 48 {
                    // Tab = 48 en CGKeyCode
                    state.forwarding.set(false);
                    let _ = state.tx.try_send(CapturedEvent::HotkeyPressed);
                    return None; // Bloquear la hotkey
                }
            }

            if state.forwarding.get() {
                // ── MODO SENDER: reenviar todo al PC remoto ─────────────────
                let input_event = match event_type {
                    CGEventType::MouseMoved => {
                        let pos = event.location();
                        Some(InputEvent::MouseMove { dx: pos.x, dy: pos.y })
                    }
                    CGEventType::LeftMouseDown => Some(InputEvent::MouseButton {
                        button: MouseButton::Left, pressed: true,
                    }),
                    CGEventType::LeftMouseUp => Some(InputEvent::MouseButton {
                        button: MouseButton::Left, pressed: false,
                    }),
                    CGEventType::RightMouseDown => Some(InputEvent::MouseButton {
                        button: MouseButton::Right, pressed: true,
                    }),
                    CGEventType::RightMouseUp => Some(InputEvent::MouseButton {
                        button: MouseButton::Right, pressed: false,
                    }),
                    CGEventType::KeyDown | CGEventType::KeyUp => {
                        let keycode = event.get_integer_value_field(
                            core_graphics::event::EventField::KEYBOARD_EVENT_KEYCODE,
                        ) as u32;
                        let flags = event.get_flags();
                        let pressed = matches!(event_type, CGEventType::KeyDown);
                        Some(InputEvent::KeyPress {
                            key_code: keycode,
                            pressed,
                            modifiers: Modifiers {
                                ctrl: flags.contains(CGEventFlags::CGEventFlagControl),
                                alt: flags.contains(CGEventFlags::CGEventFlagAlternate),
                                shift: flags.contains(CGEventFlags::CGEventFlagShift),
                                meta: flags.contains(CGEventFlags::CGEventFlagCommand),
                            },
                        })
                    }
                    _ => None,
                };

                if let Some(ev) = input_event {
                    let _ = state.tx.try_send(CapturedEvent::Forward(ev));
                }

                return None; // Bloquear: no llega a ninguna app de este Mac
            } else {
                // ── MODO LOCAL: detectar borde de pantalla ───────────────────
                if matches!(event_type, CGEventType::MouseMoved) {
                    let pos = event.location();
                    let at_edge = match state.edge_side.as_str() {
                        "right"  => pos.x >= state.screen_width - state.edge_threshold,
                        "left"   => pos.x <= state.edge_threshold,
                        "top"    => pos.y <= state.edge_threshold,
                        "bottom" => pos.y >= state.screen_height - state.edge_threshold,
                        _ => false,
                    };

                    if at_edge {
                        tracing::info!("🔀 Borde alcanzado — cambiando al PC remoto");
                        state.forwarding.set(true);
                        let _ = state.tx.try_send(CapturedEvent::EdgeReached);
                        return None; // Bloquear este movimiento de borde
                    }
                }
            }

            // Dejar pasar el evento al sistema normalmente
            Some(event.to_owned())
        },
    ).map_err(|_| anyhow::anyhow!(
        "No se pudo crear CGEventTap.\n\
         ¿Tienes permisos de Accesibilidad?\n\
         Sistema → Privacidad y Seguridad → Accesibilidad → apliarte-switch ✅"
    ))?;

    tracing::info!("✅ CGEventTap activo — capturando input del sistema");

    let loop_source = tap.mach_port
        .create_runloop_source(0)
        .map_err(|_| anyhow::anyhow!("No se pudo crear RunLoop source"))?;

    let run_loop = CFRunLoop::get_current();
    run_loop.add_source(&loop_source, unsafe { kCFRunLoopDefaultMode });
    tap.enable();

    // CFRunLoop::run_current() bloquea procesando eventos hasta que se señaliza.
    // Javier: es el event loop del sistema — nunca sale solo mientras corra el proceso.
    while running.load(Ordering::Relaxed) {
        CFRunLoop::run_current();
    }

    Ok(())
}

