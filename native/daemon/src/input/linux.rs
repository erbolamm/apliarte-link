// Javier: Implementación de input para Linux.
// Usamos uinput — un módulo del kernel Linux que permite crear dispositivos de input virtuales.
//
// Requisito: acceso a /dev/uinput
//   sudo usermod -a -G input $USER   (y reiniciar sesión)
//   sudo modprobe uinput
//
// NOTA: uinput funciona en X11 y Wayland (inyección a nivel de kernel).

use std::sync::{Mutex, OnceLock};

use crate::input::protocol::{InputEvent, MouseButton};
use anyhow::{Context, Result};
use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{
    AttributeSet, EventType, InputEvent as EvInputEvent, Key, RelativeAxisType, Synchronization,
};

/// Dispositivo virtual uinput compartido (Mutex porque emit() necesita &mut).
fn virtual_device() -> Result<&'static Mutex<VirtualDevice>> {
    static DEV: OnceLock<Result<Mutex<VirtualDevice>>> = OnceLock::new();
    DEV.get_or_init(|| {
        // Botones de ratón + rango completo de teclas
        let mut keys = AttributeSet::<Key>::new();
        keys.insert(Key::BTN_LEFT);
        keys.insert(Key::BTN_RIGHT);
        keys.insert(Key::BTN_MIDDLE);
        keys.insert(Key::BTN_SIDE);
        keys.insert(Key::BTN_EXTRA);
        for code in 0..=255u16 {
            keys.insert(Key::new(code));
        }

        // Ejes relativos del ratón
        let mut rel = AttributeSet::<RelativeAxisType>::new();
        rel.insert(RelativeAxisType::REL_X);
        rel.insert(RelativeAxisType::REL_Y);
        rel.insert(RelativeAxisType::REL_WHEEL);
        rel.insert(RelativeAxisType::REL_HWHEEL);

        VirtualDeviceBuilder::new()
            .context(
                "abrir /dev/uinput (¿permisos? sudo usermod -a -G input $USER + sudo modprobe uinput)",
            )?
            .name("ApliArte Link virtual input")
            .with_keys(&keys)
            .context("configurar teclas")?
            .with_relative_axes(&rel)
            .context("configurar ejes del ratón")?
            .build()
            .context("crear dispositivo virtual uinput")
            .map(Mutex::new)
    })
    .as_ref()
    .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Envía una ráfaga de eventos al dispositivo virtual y sincroniza.
fn emit(events: &[EvInputEvent]) -> Result<()> {
    let dev = virtual_device()?;
    let mut guard = dev.lock().map_err(|_| anyhow::anyhow!("lock uinput envenenado"))?;
    guard.emit(events).context("emitir eventos a uinput")?;
    Ok(())
}

fn syn() -> EvInputEvent {
    EvInputEvent::new(EventType::SYNCHRONIZATION, Synchronization::SYN_REPORT.0, 0)
}

/// Inyecta un evento de input en Linux vía uinput
pub async fn inject(event: InputEvent) -> Result<()> {
    tokio::task::spawn_blocking(move || inject_sync(event))
        .await
        .context("tarea de inyección cancelada")??;
    Ok(())
}

fn inject_sync(event: InputEvent) -> Result<()> {
    match event {
        InputEvent::MouseMove { dx, dy, .. } => {
            tracing::debug!("[Linux] MouseMove dx={} dy={}", dx, dy);
            emit(&[
                EvInputEvent::new(
                    EventType::RELATIVE,
                    RelativeAxisType::REL_X.0,
                    dx as i32,
                ),
                EvInputEvent::new(
                    EventType::RELATIVE,
                    RelativeAxisType::REL_Y.0,
                    dy as i32,
                ),
                syn(),
            ])?;
        }
        InputEvent::MouseButton { button, pressed, .. } => {
            let code = match button {
                MouseButton::Left => Key::BTN_LEFT,
                MouseButton::Right => Key::BTN_RIGHT,
                MouseButton::Middle => Key::BTN_MIDDLE,
                MouseButton::Back => Key::BTN_SIDE,
                MouseButton::Forward => Key::BTN_EXTRA,
            };
            tracing::debug!("[Linux] MouseButton {:?} pressed={}", button, pressed);
            emit(&[
                EvInputEvent::new(EventType::KEY, code.0, if pressed { 1 } else { 0 }),
                syn(),
            ])?;
        }
        InputEvent::MouseScroll { dx, dy, .. } => {
            tracing::debug!("[Linux] Scroll dx={} dy={}", dx, dy);
            let mut events = Vec::with_capacity(3);
            if dy != 0.0 {
                events.push(EvInputEvent::new(
                    EventType::RELATIVE,
                    RelativeAxisType::REL_WHEEL.0,
                    dy as i32,
                ));
            }
            if dx != 0.0 {
                events.push(EvInputEvent::new(
                    EventType::RELATIVE,
                    RelativeAxisType::REL_HWHEEL.0,
                    dx as i32,
                ));
            }
            events.push(syn());
            emit(&events)?;
        }
        InputEvent::KeyPress {
            key_code, pressed, ..
        } => {
            tracing::debug!("[Linux] KeyPress code={} pressed={}", key_code, pressed);
            emit(&[
                EvInputEvent::new(
                    EventType::KEY,
                    Key::new(key_code as u16).0,
                    if pressed { 1 } else { 0 },
                ),
                syn(),
            ])?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::protocol::{Modifiers, MouseButton};

    /// Prueba de humo real: crea el dispositivo virtual uinput y emite eventos.
    /// Requiere permisos sobre /dev/uinput (sudo modprobe uinput + chmod 666).
    #[tokio::test]
    async fn uinput_smoke_inject_mouse_and_key() {
        inject(InputEvent::MouseMove {
            dx: 10.0,
            dy: 5.0,
            seq: 1,
        })
        .await
        .expect("inyectar MouseMove");

        inject(InputEvent::MouseButton {
            button: MouseButton::Left,
            pressed: true,
            seq: 2,
        })
        .await
        .expect("inyectar MouseButton press");
        inject(InputEvent::MouseButton {
            button: MouseButton::Left,
            pressed: false,
            seq: 3,
        })
        .await
        .expect("inyectar MouseButton release");

        inject(InputEvent::MouseScroll {
            dx: 0.0,
            dy: -3.0,
            seq: 4,
        })
        .await
        .expect("inyectar MouseScroll");

        // Tecla 'a' (KEY_A = 30)
        inject(InputEvent::KeyPress {
            key_code: 30,
            pressed: true,
            modifiers: Modifiers::default(),
            seq: 5,
        })
        .await
        .expect("inyectar KeyPress");
        inject(InputEvent::KeyPress {
            key_code: 30,
            pressed: false,
            modifiers: Modifiers::default(),
            seq: 6,
        })
        .await
        .expect("inyectar KeyPress release");

        tracing::info!("[TEST] uinput smoke OK: dispositivo virtual creado y eventos emitidos");
    }
}

// ═══════════════════════════════════════════════════════════════
// Listener de captura para Linux — evdev (/dev/input/event*)
// Detecta el comando de cambio (triple clic derecho / triple shift)
// y reenvía los eventos locales al peer activo (como macOS/Windows).
// ═══════════════════════════════════════════════════════════════

use crate::config::AppConfig;
use crate::control::{self, CommandDetector, ControlState};
use crate::network::{ConnectionManager, PeerList};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use evdev::raw_stream::RawDevice;
use evdev::InputEventKind;

/// Listener global de captura para Linux.
/// Abre los dispositivos de teclado y ratón de /dev/input, detecta el
/// comando de switch y reenvía eventos al peer activo.
pub fn start_switch_listener(
    cfg: Arc<Mutex<AppConfig>>,
    peers: PeerList,
    active_peer: Arc<Mutex<Option<SocketAddr>>>,
    control_state: ControlState,
    conn_mgr: ConnectionManager,
) -> Result<()> {
    std::thread::Builder::new()
        .name("apliarte-link-switch-linux".to_string())
        .spawn(move || {
            // Enumerar dispositivos de input (teclados y ratones)
            let mut devices: Vec<_> = evdev::enumerate()
                .filter_map(|(path, dev)| {
                    let keys = dev.supported_keys();
                    let is_mouse = keys.map(|k| k.contains(Key::BTN_LEFT)).unwrap_or(false);
                    let is_keyboard = keys.map(|k| k.contains(Key::KEY_A)).unwrap_or(false);
                    if is_mouse || is_keyboard {
                        tracing::debug!(
                            "[Linux] Capturando dispositivo {:?} (mouse={}, teclado={})",
                            path,
                            is_mouse,
                            is_keyboard
                        );
                        Some(dev)
                    } else {
                        None
                    }
                })
                .collect();

            if devices.is_empty() {
                tracing::warn!("[Linux] No se encontraron dispositivos de input en /dev/input");
                return;
            }
            tracing::info!("[Linux] Listener de captura activo ({} dispositivos)", devices.len());

            let mut detector = CommandDetector::default();

            loop {
                for device in devices.iter_mut() {
                    if let Ok(events) = device.fetch_events() {
                        for ev in events {
                            let input_cfg = cfg.lock().unwrap().input.clone();
                            let allowed = cfg.lock().unwrap().sharing.allowed_peers.clone();
                            handle_evdev_event(
                                &ev,
                                &mut detector,
                                &input_cfg,
                                &allowed,
                                &peers,
                                &active_peer,
                                &control_state,
                                &conn_mgr,
                            );
                        }
                    }
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        })
        .map_err(|error| {
            anyhow::anyhow!("No se pudo arrancar el listener global de Linux: {}", error)
        })?;

    Ok(())
}

/// Procesa un evento evdev: detecta comandos de switch y reenvía input.
#[allow(clippy::too_many_arguments)]
fn handle_evdev_event(
    ev: &evdev::InputEvent,
    detector: &mut CommandDetector,
    input_cfg: &crate::config::InputConfig,
    allowed: &[String],
    peers: &PeerList,
    active_peer: &Arc<Mutex<Option<SocketAddr>>>,
    control_state: &ControlState,
    conn_mgr: &ConnectionManager,
) {
    let pressed = ev.value() != 0;

    // 1) Detección del comando de cambio (ratón y teclado)
    let should_cycle = match ev.kind() {
        InputEventKind::Key(key) if pressed => {
            let code = key.0;
            if code == Key::BTN_RIGHT.0 {
                detector.register_mouse(MouseButton::Right, input_cfg)
            } else if code == Key::BTN_MIDDLE.0 {
                detector.register_mouse(MouseButton::Middle, input_cfg)
            } else {
                let matched =
                    matches_linux_keyboard_command(&input_cfg.keyboard_switch_command, code);
                detector.register_keyboard_match(matched)
            }
        }
        _ => false,
    };

    if should_cycle {
        control::cycle_active_peer(peers, active_peer, control_state, conn_mgr, allowed);
        return;
    }

    // 2) Si hay peer activo y es evento reservado, no reenviar (se traga)
    if control::current_active_peer(active_peer).is_some()
        && is_reserved_evdev_event(&input_cfg, ev)
    {
        return;
    }

    // 3) Capturar eventos para el peer remoto
    if let Some(proto) = evdev_to_protocol(ev) {
        let mgr = conn_mgr.clone();
        let peer = active_peer.clone();
        tokio::spawn(async move {
            control::send_input_to_active_peer(&mgr, &peer, &proto).await;
        });
    }
}

/// Comando de teclado para Linux: triple_right_shift / triple_escape.
fn matches_linux_keyboard_command(command: &str, key_code: u16) -> bool {
    match command {
        "triple_right_shift" => {
            key_code == Key::KEY_RIGHTSHIFT.0 || key_code == Key::KEY_LEFTSHIFT.0
        }
        "triple_escape" => key_code == Key::KEY_ESC.0,
        _ => false,
    }
}

/// ¿El evento es parte del comando de switch y debe tragarse?
fn is_reserved_evdev_event(input_cfg: &crate::config::InputConfig, ev: &evdev::InputEvent) -> bool {
    match ev.kind() {
        InputEventKind::Key(key) => {
            let code = key.0;
            matches_linux_keyboard_command(&input_cfg.keyboard_switch_command, code)
                || (input_cfg.mouse_switch_command == "triple_right_click"
                    && code == Key::BTN_RIGHT.0)
                || (input_cfg.mouse_switch_command == "triple_middle_click"
                    && code == Key::BTN_MIDDLE.0)
        }
        _ => false,
    }
}

/// Convierte un evento evdev en el protocolo de ApliArte Link.
fn evdev_to_protocol(ev: &evdev::InputEvent) -> Option<InputEvent> {
    let value = ev.value();
    match ev.kind() {
        InputEventKind::RelAxis(axis) => {
            let a = axis.0;
            if a == RelativeAxisType::REL_X.0 {
                Some(InputEvent::MouseMove { dx: value as f64, dy: 0.0, seq: 0 })
            } else if a == RelativeAxisType::REL_Y.0 {
                Some(InputEvent::MouseMove { dx: 0.0, dy: value as f64, seq: 0 })
            } else if a == RelativeAxisType::REL_WHEEL.0 {
                Some(InputEvent::MouseScroll { dx: 0.0, dy: value as f64, seq: 0 })
            } else if a == RelativeAxisType::REL_HWHEEL.0 {
                Some(InputEvent::MouseScroll { dx: value as f64, dy: 0.0, seq: 0 })
            } else {
                None
            }
        }
        InputEventKind::Key(key) => {
            let code = key.0;
            let pressed = value != 0;
            if code == Key::BTN_LEFT.0 {
                Some(InputEvent::MouseButton { button: MouseButton::Left, pressed, seq: 0 })
            } else if code == Key::BTN_RIGHT.0 {
                Some(InputEvent::MouseButton { button: MouseButton::Right, pressed, seq: 0 })
            } else if code == Key::BTN_MIDDLE.0 {
                Some(InputEvent::MouseButton { button: MouseButton::Middle, pressed, seq: 0 })
            } else if code == Key::BTN_SIDE.0 {
                Some(InputEvent::MouseButton { button: MouseButton::Back, pressed, seq: 0 })
            } else if code == Key::BTN_EXTRA.0 {
                Some(InputEvent::MouseButton { button: MouseButton::Forward, pressed, seq: 0 })
            } else {
                // Teclas normales: se envían como KeyPress
                Some(InputEvent::KeyPress {
                    key_code: code as u32,
                    pressed,
                    modifiers: crate::input::protocol::Modifiers::default(),
                    seq: 0,
                })
            }
        }
        _ => None,
    }
}
