// Javier: Implementación de input para macOS.
// Usamos CGEvent — la API oficial de Apple para simular teclado y ratón.
//
// Analogía Flutter: es como escribir código en Swift en un Platform Channel.
// Rust lo llama directamente a través de los bindings de core-graphics y core-foundation.
//
// Para que funcione, el usuario debe dar permiso en:
//   Sistema → Privacidad y Seguridad → Accesibilidad → apliarte-link ✅
//   Sistema → Privacidad y Seguridad → Monitor de Teclado → apliarte-link ✅

use crate::config::AppConfig;
use crate::control::{self, CommandDetector, ControlState};
use crate::input::protocol::{InputEvent, Modifiers, MouseButton};
use crate::network::ConnectionManager;
use crate::network::PeerList;
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, OnceLock};

/// Última posición conocida del cursor — para convertir deltas en posición absoluta
/// (CGEvent de macOS trabaja con coordenadas absolutas de pantalla).
static LAST_MOUSE_POS: OnceLock<Mutex<Option<(f64, f64)>>> = OnceLock::new();

fn last_mouse_pos() -> &'static Mutex<Option<(f64, f64)>> {
    LAST_MOUSE_POS.get_or_init(|| Mutex::new(None))
}

/// Inyecta un evento de input en macOS
/// Javier: "inyectar" = decirle a macOS que el ratón/teclado hizo algo,
/// aunque físicamente no se haya movido nada
pub async fn inject(event: InputEvent) -> Result<()> {
    // Javier: tokio::task::spawn_blocking es para código que bloquea el hilo
    // (las APIs de C de Apple no son async). Es como usar compute() en Flutter.
    tokio::task::spawn_blocking(move || inject_sync(event)).await??;
    Ok(())
}

fn inject_sync(event: InputEvent) -> Result<()> {
    use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
    use core_graphics::geometry::CGPoint;

    // Javier: CGEventSource es como el "origen" del evento. Le decimos al SO
    // que este evento viene de un dispositivo HID (hardware input device) real.
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| anyhow::anyhow!("No se pudo crear CGEventSource"))?;

    match event {
        InputEvent::MouseMove { dx, dy, .. } => {
            // CGEvent de macOS usa posición ABSOLUTA: hay que sumar el delta a la
            // posición actual conocida (fix del TODO original "calcular posición absoluta").
            let mut last = last_mouse_pos().lock().unwrap();
            let (px, py) = last.unwrap_or((0.0, 0.0));
            let nx = px + dx;
            let ny = py + dy;
            *last = Some((nx, ny));
            drop(last);

            let current = CGEvent::new_mouse_event(
                source.clone(),
                CGEventType::MouseMoved,
                CGPoint::new(nx, ny),
                CGMouseButton::Left,
            )
            .map_err(|_| anyhow::anyhow!("Error creando evento de ratón"))?;

            current.post(CGEventTapLocation::HID);
        }

        InputEvent::MouseButton {
            button,
            pressed,
            seq: _,
        } => {
            let btn = match button {
                MouseButton::Left => CGMouseButton::Left,
                MouseButton::Right => CGMouseButton::Right,
                MouseButton::Middle => CGMouseButton::Center,
                _ => CGMouseButton::Left,
            };

            let event_type = match (button, pressed) {
                (MouseButton::Left, true) => CGEventType::LeftMouseDown,
                (MouseButton::Left, false) => CGEventType::LeftMouseUp,
                (MouseButton::Right, true) => CGEventType::RightMouseDown,
                (MouseButton::Right, false) => CGEventType::RightMouseUp,
                (MouseButton::Middle, true) => CGEventType::OtherMouseDown,
                (MouseButton::Middle, false) => CGEventType::OtherMouseUp,
                (_, true) => CGEventType::OtherMouseDown,
                (_, false) => CGEventType::OtherMouseUp,
            };

            // Los clicks deben ir en la posición actual del cursor, no en (0,0)
            let (px, py) = last_mouse_pos()
                .lock()
                .unwrap()
                .unwrap_or((0.0, 0.0));
            let evt = CGEvent::new_mouse_event(
                source,
                event_type,
                CGPoint::new(px, py),
                btn,
            )
            .map_err(|_| anyhow::anyhow!("Error creando evento de click"))?;

            evt.post(CGEventTapLocation::HID);
        }

        InputEvent::MouseScroll { dx, dy, .. } => {
            // Implementado: CGEventCreateScrollWheelEvent (rueda con signo, línea/píxel)
            let evt = CGEvent::new_scroll_event(
                source,
                core_graphics::event::ScrollEventUnit::PIXEL,
                2,
                dy as i32,
                dx as i32,
                0,
            )
            .map_err(|_| anyhow::anyhow!("Error creando evento de scroll"))?;

            evt.post(CGEventTapLocation::HID);
        }

        InputEvent::KeyPress {
            key_code,
            pressed,
            modifiers,
            ..
        } => {
            // CGEvent para teclado — key_code es el código de tecla virtual
            let evt = CGEvent::new_keyboard_event(source, key_code as u16, pressed)
                .map_err(|_| anyhow::anyhow!("Error creando evento de teclado"))?;

            // Aplicar modificadores (Ctrl/Alt/Shift/Cmd)
            use core_graphics::event::CGEventFlags;
            let mut flags = CGEventFlags::empty();
            if modifiers.ctrl {
                flags.insert(CGEventFlags::CGEventFlagControl);
            }
            if modifiers.alt {
                flags.insert(CGEventFlags::CGEventFlagAlternate);
            }
            if modifiers.shift {
                flags.insert(CGEventFlags::CGEventFlagShift);
            }
            if modifiers.meta {
                flags.insert(CGEventFlags::CGEventFlagCommand);
            }
            evt.set_flags(flags);

            evt.post(CGEventTapLocation::HID);
        }
    }

    Ok(())
}

pub fn start_switch_listener(
    cfg: Arc<Mutex<AppConfig>>,
    peers: PeerList,
    active_peer: Arc<Mutex<Option<SocketAddr>>>,
    control_state: ControlState,
    conn_mgr: ConnectionManager,
) -> Result<()> {
    std::thread::Builder::new()
        .name("apliarte-link-switch-macos".to_string())
        .spawn(move || {
            use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
            use core_graphics::event::{
                CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
                CGEventType, EventField,
            };

            let detector = Arc::new(Mutex::new(CommandDetector::default()));
            let callback_detector = detector.clone();
            let callback_cfg = cfg.clone();
            let callback_peers = peers.clone();
            let callback_active_peer = active_peer.clone();
            let callback_control_state = control_state.clone();
            let callback_conn_mgr = conn_mgr.clone();

            let tap = CGEventTap::new(
                CGEventTapLocation::HID,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::Default,
                vec![
                    CGEventType::MouseMoved,
                    CGEventType::LeftMouseDown,
                    CGEventType::LeftMouseUp,
                    CGEventType::RightMouseDown,
                    CGEventType::RightMouseUp,
                    CGEventType::OtherMouseDown,
                    CGEventType::OtherMouseUp,
                    CGEventType::LeftMouseDragged,
                    CGEventType::RightMouseDragged,
                    CGEventType::OtherMouseDragged,
                    CGEventType::ScrollWheel,
                    CGEventType::KeyDown,
                    CGEventType::KeyUp,
                    CGEventType::FlagsChanged,
                ],
                move |_proxy, event_type, event| {
                    let input_cfg = callback_cfg.lock().unwrap().input.clone();

                    if !input_cfg.armed {
                        callback_detector.lock().unwrap().reset();
                        return Some(event.clone());
                    }

                    let should_cycle = match event_type {
                        CGEventType::RightMouseDown => callback_detector
                            .lock()
                            .unwrap()
                            .register_mouse(MouseButton::Right, &input_cfg),
                        CGEventType::OtherMouseDown => {
                            let button_number = event
                                .get_integer_value_field(EventField::MOUSE_EVENT_BUTTON_NUMBER);
                            if button_number == 2 {
                                callback_detector
                                    .lock()
                                    .unwrap()
                                    .register_mouse(MouseButton::Middle, &input_cfg)
                            } else {
                                false
                            }
                        }
                        CGEventType::KeyDown => {
                            let key_code = event
                                .get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE)
                                as u32;
                            let is_repeat = event
                                .get_integer_value_field(EventField::KEYBOARD_EVENT_AUTOREPEAT)
                                != 0;

                            !is_repeat
                                && callback_detector.lock().unwrap().register_keyboard_match(
                                    matches_macos_keyboard_command(
                                        &input_cfg.keyboard_switch_command,
                                        key_code,
                                    ),
                                )
                        }
                        CGEventType::FlagsChanged => {
                            let key_code = event
                                .get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE)
                                as u32;
                            flags_changed_is_pressed(key_code, event.get_flags())
                                && callback_detector.lock().unwrap().register_keyboard_match(
                                    matches_macos_keyboard_command(
                                        &input_cfg.keyboard_switch_command,
                                        key_code,
                                    ),
                                )
                        }
                        _ => false,
                    };

                    if should_cycle {
                        control::cycle_active_peer(
                            &callback_peers,
                            &callback_active_peer,
                            &callback_control_state,
                            &callback_conn_mgr,
                        );
                        None
                    } else if control::current_active_peer(&callback_active_peer).is_some()
                        && is_reserved_command_event(&input_cfg, event_type, event)
                    {
                        None
                    } else if let Some(input_event) = capture_event_for_remote(event_type, event) {
                        let mgr = callback_conn_mgr.clone();
                        let peer = callback_active_peer.clone();
                        let event_clone = input_event.clone();
                        tokio::spawn(async move {
                            control::send_input_to_active_peer(&mgr, &peer, &event_clone).await;
                        });
                        None
                    } else {
                        Some(event.clone())
                    }
                },
            );

            match tap {
                Ok(tap) => match tap.mach_port.create_runloop_source(0) {
                    Ok(source) => {
                        let run_loop = CFRunLoop::get_current();
                        unsafe {
                            run_loop.add_source(&source, kCFRunLoopCommonModes);
                        }
                        tap.enable();
                        CFRunLoop::run_current();
                    }
                    Err(_) => {
                        tracing::error!("No se pudo crear el runloop source del switch en macOS");
                    }
                },
                Err(_) => {
                    tracing::error!("No se pudo activar el event tap global en macOS");
                }
            }
        })
        .map_err(|error| {
            anyhow::anyhow!("No se pudo arrancar el listener global de macOS: {}", error)
        })?;

    Ok(())
}

fn matches_macos_keyboard_command(command: &str, key_code: u32) -> bool {
    match command {
        "triple_right_shift" => key_code == core_graphics::event::KeyCode::RIGHT_SHIFT as u32,
        "triple_escape" => key_code == core_graphics::event::KeyCode::ESCAPE as u32,
        _ => false,
    }
}

fn is_reserved_command_event(
    input_cfg: &crate::config::InputConfig,
    event_type: core_graphics::event::CGEventType,
    event: &core_graphics::event::CGEvent,
) -> bool {
    match event_type {
        core_graphics::event::CGEventType::KeyDown
        | core_graphics::event::CGEventType::KeyUp
        | core_graphics::event::CGEventType::FlagsChanged => {
            let key_code = event
                .get_integer_value_field(core_graphics::event::EventField::KEYBOARD_EVENT_KEYCODE)
                as u32;
            matches_macos_keyboard_command(&input_cfg.keyboard_switch_command, key_code)
        }
        core_graphics::event::CGEventType::RightMouseDown
        | core_graphics::event::CGEventType::RightMouseUp => {
            input_cfg.mouse_switch_command == "triple_right_click"
        }
        core_graphics::event::CGEventType::OtherMouseDown
        | core_graphics::event::CGEventType::OtherMouseUp => {
            input_cfg.mouse_switch_command == "triple_middle_click"
                && event.get_integer_value_field(
                    core_graphics::event::EventField::MOUSE_EVENT_BUTTON_NUMBER,
                ) == 2
        }
        _ => false,
    }
}

fn capture_event_for_remote(
    event_type: core_graphics::event::CGEventType,
    event: &core_graphics::event::CGEvent,
) -> Option<InputEvent> {
    use core_graphics::event::{CGEventType, EventField};

    match event_type {
        CGEventType::MouseMoved
        | CGEventType::LeftMouseDragged
        | CGEventType::RightMouseDragged
        | CGEventType::OtherMouseDragged => {
            let dx = event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_X) as f64;
            let dy = event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_Y) as f64;
            if dx == 0.0 && dy == 0.0 {
                None
            } else {
                Some(InputEvent::MouseMove { dx, dy, seq: 0 })
            }
        }
        CGEventType::LeftMouseDown => Some(InputEvent::MouseButton {
            seq: 0,
            button: MouseButton::Left,
            pressed: true,
        }),
        CGEventType::LeftMouseUp => Some(InputEvent::MouseButton {
            seq: 0,
            button: MouseButton::Left,
            pressed: false,
        }),
        CGEventType::RightMouseDown => Some(InputEvent::MouseButton {
            seq: 0,
            button: MouseButton::Right,
            pressed: true,
        }),
        CGEventType::RightMouseUp => Some(InputEvent::MouseButton {
            seq: 0,
            button: MouseButton::Right,
            pressed: false,
        }),
        CGEventType::OtherMouseDown | CGEventType::OtherMouseUp => {
            let button_number =
                event.get_integer_value_field(EventField::MOUSE_EVENT_BUTTON_NUMBER);
            let button = match button_number {
                2 => MouseButton::Middle,
                _ => return None,
            };
            Some(InputEvent::MouseButton {
                seq: 0,
                button,
                pressed: matches!(event_type, CGEventType::OtherMouseDown),
            })
        }
        CGEventType::ScrollWheel => {
            let dx =
                event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_DELTA_AXIS_2) as f64;
            let dy =
                event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_DELTA_AXIS_1) as f64;
            if dx == 0.0 && dy == 0.0 {
                None
            } else {
                Some(InputEvent::MouseScroll { dx, dy, seq: 0 })
            }
        }
        CGEventType::KeyDown | CGEventType::KeyUp => {
            let key_code = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u32;
            Some(InputEvent::KeyPress {
                seq: 0,
                key_code,
                pressed: matches!(event_type, CGEventType::KeyDown),
                modifiers: modifiers_from_flags(event.get_flags()),
            })
        }
        CGEventType::FlagsChanged => {
            let key_code = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u32;
            Some(InputEvent::KeyPress {
                seq: 0,
                key_code,
                pressed: flags_changed_is_pressed(key_code, event.get_flags()),
                modifiers: modifiers_from_flags(event.get_flags()),
            })
        }
        _ => None,
    }
}

fn flags_changed_is_pressed(key_code: u32, flags: core_graphics::event::CGEventFlags) -> bool {
    use core_graphics::event::{CGEventFlags, KeyCode};

    match key_code {
        value if value == KeyCode::SHIFT as u32 || value == KeyCode::RIGHT_SHIFT as u32 => {
            flags.contains(CGEventFlags::CGEventFlagShift)
        }
        value if value == KeyCode::CONTROL as u32 || value == KeyCode::RIGHT_CONTROL as u32 => {
            flags.contains(CGEventFlags::CGEventFlagControl)
        }
        value if value == KeyCode::OPTION as u32 || value == KeyCode::RIGHT_OPTION as u32 => {
            flags.contains(CGEventFlags::CGEventFlagAlternate)
        }
        value if value == KeyCode::COMMAND as u32 || value == KeyCode::RIGHT_COMMAND as u32 => {
            flags.contains(CGEventFlags::CGEventFlagCommand)
        }
        _ => false,
    }
}

fn modifiers_from_flags(flags: core_graphics::event::CGEventFlags) -> Modifiers {
    Modifiers {
        ctrl: flags.contains(core_graphics::event::CGEventFlags::CGEventFlagControl),
        alt: flags.contains(core_graphics::event::CGEventFlags::CGEventFlagAlternate),
        shift: flags.contains(core_graphics::event::CGEventFlags::CGEventFlagShift),
        meta: flags.contains(core_graphics::event::CGEventFlags::CGEventFlagCommand),
    }
}
