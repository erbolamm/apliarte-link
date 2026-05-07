// Javier: Implementación de input para Windows.
// Usamos SendInput — la API oficial de Windows para simular teclado y ratón.
//
// Analogía Flutter: es como escribir código nativo en Kotlin/Java
// dentro de un Platform Channel para Android/Windows.
//
// En Windows generalmente no hace falta dar permisos especiales,
// aunque en algunos casos puede pedir ejecutar como Administrador.

use crate::config::AppConfig;
use crate::control::{self, CommandDetector, ControlState};
use crate::input::protocol::{InputEvent, Modifiers, MouseButton};
use crate::network::PeerList;
use anyhow::Result;
use std::net::{SocketAddr, UdpSocket as StdUdpSocket};
use std::sync::{Arc, Mutex, OnceLock};

/// Inyecta un evento de input en Windows
pub async fn inject(event: InputEvent) -> Result<()> {
    tokio::task::spawn_blocking(move || inject_sync(event)).await??;
    Ok(())
}

fn inject_sync(event: InputEvent) -> Result<()> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP,
        MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN,
        MOUSEEVENTF_RIGHTUP, MOUSEINPUT,
    };

    let input = match event {
        InputEvent::MouseMove { dx, dy } => INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                mi: MOUSEINPUT {
                    dx: dx as i32,
                    dy: dy as i32,
                    dwFlags: MOUSEEVENTF_MOVE,
                    ..Default::default()
                },
            },
        },

        InputEvent::MouseButton { button, pressed } => {
            let flags = match (button, pressed) {
                (MouseButton::Left, true) => MOUSEEVENTF_LEFTDOWN,
                (MouseButton::Left, false) => MOUSEEVENTF_LEFTUP,
                (MouseButton::Right, true) => MOUSEEVENTF_RIGHTDOWN,
                (MouseButton::Right, false) => MOUSEEVENTF_RIGHTUP,
                _ => MOUSEEVENTF_LEFTDOWN,
            };

            INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    mi: MOUSEINPUT {
                        dwFlags: flags,
                        ..Default::default()
                    },
                },
            }
        }

        InputEvent::KeyPress {
            key_code, pressed, ..
        } => INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(key_code as u16),
                    dwFlags: if pressed {
                        Default::default()
                    } else {
                        KEYEVENTF_KEYUP
                    },
                    ..Default::default()
                },
            },
        },

        InputEvent::MouseScroll { dy, .. } => INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                mi: MOUSEINPUT {
                    dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_WHEEL,
                    mouseData: (dy * 120.0) as u32,
                    ..Default::default()
                },
            },
        },
    };

    // Javier: SendInput envía el array de inputs al SO
    unsafe {
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }

    Ok(())
}

struct WindowsSwitchContext {
    cfg: Arc<Mutex<AppConfig>>,
    peers: PeerList,
    active_peer: Arc<Mutex<Option<SocketAddr>>>,
    control_state: ControlState,
    detector: Mutex<CommandDetector>,
    last_mouse_pos: Mutex<Option<(i32, i32)>>,
    send_socket: Arc<StdUdpSocket>,
    data_port: u16,
}

static WINDOWS_SWITCH_CONTEXT: OnceLock<WindowsSwitchContext> = OnceLock::new();

pub fn start_switch_listener(
    cfg: Arc<Mutex<AppConfig>>,
    peers: PeerList,
    active_peer: Arc<Mutex<Option<SocketAddr>>>,
    control_state: ControlState,
    data_port: u16,
) -> Result<()> {
    let send_socket = StdUdpSocket::bind("0.0.0.0:0")
        .map_err(|error| anyhow::anyhow!("No se pudo abrir el socket de captura en Windows: {}", error))?;

    let _ = WINDOWS_SWITCH_CONTEXT.set(WindowsSwitchContext {
        cfg,
        peers,
        active_peer,
        control_state,
        detector: Mutex::new(CommandDetector::default()),
        last_mouse_pos: Mutex::new(None),
        send_socket: Arc::new(send_socket),
        data_port,
    });

    std::thread::Builder::new()
        .name("apliarte-link-switch-windows".to_string())
        .spawn(move || unsafe {
            use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
            use windows::Win32::System::LibraryLoader::GetModuleHandleW;
            use windows::Win32::UI::Input::KeyboardAndMouse::{VK_ESCAPE, VK_RSHIFT};
            use windows::Win32::UI::WindowsAndMessaging::{KBDLLHOOKSTRUCT, MSLLHOOKSTRUCT};
            use windows::Win32::UI::WindowsAndMessaging::{
                CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW,
                TranslateMessage, UnhookWindowsHookEx, HC_ACTION, HHOOK, MSG,
                WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN,
                WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE,
                WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN,
                WM_SYSKEYUP,
            };

            extern "system" fn keyboard_hook(
                code: i32,
                wparam: WPARAM,
                lparam: LPARAM,
            ) -> LRESULT {
                use windows::Win32::Foundation::LRESULT;
                use windows::Win32::UI::Input::KeyboardAndMouse::{VK_ESCAPE, VK_RSHIFT};
                use windows::Win32::UI::WindowsAndMessaging::{CallNextHookEx, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP};

                if code == HC_ACTION as i32 {
                    if let Some(context) = WINDOWS_SWITCH_CONTEXT.get() {
                        let msg = wparam.0 as u32;
                        if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN || msg == WM_KEYUP || msg == WM_SYSKEYUP {
                            let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
                            let input_cfg = context.cfg.lock().unwrap().input.clone();

                            if !input_cfg.armed {
                                context.detector.lock().unwrap().reset();
                                return unsafe {
                                    CallNextHookEx(HHOOK::default(), code, wparam, lparam)
                                };
                            }

                            let matched = matches!(
                                (input_cfg.keyboard_switch_command.as_str(), info.vkCode),
                                ("triple_right_shift", value)
                                    if value == VK_RSHIFT.0 as u32
                            ) || matches!(
                                (input_cfg.keyboard_switch_command.as_str(), info.vkCode),
                                ("triple_escape", value)
                                    if value == VK_ESCAPE.0 as u32
                            );

                            if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
                                if context
                                    .detector
                                    .lock()
                                    .unwrap()
                                    .register_keyboard_match(matched)
                                {
                                    control::cycle_active_peer(
                                        &context.peers,
                                        &context.active_peer,
                                        &context.control_state,
                                        context.data_port,
                                    );
                                    return LRESULT(1);
                                }
                            }

                            let pressed = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
                            if control::current_active_peer(&context.active_peer).is_some() && matched {
                                return LRESULT(1);
                            }

                            let input_event = InputEvent::KeyPress {
                                key_code: info.vkCode,
                                pressed,
                                modifiers: Modifiers::default(),
                            };
                            if control::send_input_to_active_peer(
                                context.send_socket.as_ref(),
                                &context.active_peer,
                                &input_event,
                            ) {
                                return LRESULT(1);
                            }
                        }
                    }
                }

                unsafe { CallNextHookEx(HHOOK::default(), code, wparam, lparam) }
            }

            extern "system" fn mouse_hook(
                code: i32,
                wparam: WPARAM,
                lparam: LPARAM,
            ) -> LRESULT {
                use windows::Win32::Foundation::LRESULT;
                use windows::Win32::UI::WindowsAndMessaging::{
                    CallNextHookEx, HC_ACTION, HHOOK, WM_LBUTTONDOWN, WM_LBUTTONUP,
                    WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL,
                    WM_RBUTTONDOWN, WM_RBUTTONUP,
                };

                if code == HC_ACTION as i32 {
                    if let Some(context) = WINDOWS_SWITCH_CONTEXT.get() {
                        let info = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
                        let input_cfg = context.cfg.lock().unwrap().input.clone();

                        if !input_cfg.armed {
                            context.detector.lock().unwrap().reset();
                            if wparam.0 as u32 == WM_MOUSEMOVE {
                                *context.last_mouse_pos.lock().unwrap() = Some((info.pt.x, info.pt.y));
                            }
                            return unsafe {
                                CallNextHookEx(HHOOK::default(), code, wparam, lparam)
                            };
                        }

                        let should_cycle = match wparam.0 as u32 {
                            WM_RBUTTONDOWN => context
                                .detector
                                .lock()
                                .unwrap()
                                .register_mouse(MouseButton::Right, &input_cfg),
                            WM_MBUTTONDOWN => context
                                .detector
                                .lock()
                                .unwrap()
                                .register_mouse(MouseButton::Middle, &input_cfg),
                            _ => false,
                        };

                        if should_cycle {
                            control::cycle_active_peer(
                                &context.peers,
                                &context.active_peer,
                                &context.control_state,
                                context.data_port,
                            );
                            return LRESULT(1);
                        }

                        if control::current_active_peer(&context.active_peer).is_some()
                            && is_reserved_mouse_command(&input_cfg.mouse_switch_command, wparam.0 as u32)
                        {
                            return LRESULT(1);
                        }

                        if let Some(input_event) = mouse_input_event(
                            wparam.0 as u32,
                            info,
                            &context.last_mouse_pos,
                        ) {
                            if control::send_input_to_active_peer(
                                context.send_socket.as_ref(),
                                &context.active_peer,
                                &input_event,
                            ) {
                                return LRESULT(1);
                            }
                        }
                    }
                }

                unsafe { CallNextHookEx(HHOOK::default(), code, wparam, lparam) }
            }

            let module = GetModuleHandleW(None).unwrap_or_default();
            let mouse_hook_handle = SetWindowsHookExW(
                WH_MOUSE_LL,
                Some(mouse_hook),
                HINSTANCE(module.0),
                0,
            ).unwrap_or_default();
            let keyboard_hook_handle = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard_hook),
                HINSTANCE(module.0),
                0,
            ).unwrap_or_default();

            if mouse_hook_handle.is_invalid() || keyboard_hook_handle.is_invalid() {
                tracing::error!("No se pudieron instalar los hooks globales de Windows");
                return;
            }

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).into() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let _ = UnhookWindowsHookEx(mouse_hook_handle);
            let _ = UnhookWindowsHookEx(keyboard_hook_handle);
        })
        .map_err(|error| anyhow::anyhow!("No se pudo arrancar el listener global de Windows: {}", error))?;

    Ok(())
}

fn is_reserved_mouse_command(command: &str, message: u32) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{
        WM_MBUTTONDOWN, WM_MBUTTONUP, WM_RBUTTONDOWN, WM_RBUTTONUP,
    };
    (command == "triple_right_click"
        && (message == WM_RBUTTONDOWN || message == WM_RBUTTONUP))
        || (command == "triple_middle_click"
            && (message == WM_MBUTTONDOWN || message == WM_MBUTTONUP))
}

fn mouse_input_event(
    message: u32,
    info: &windows::Win32::UI::WindowsAndMessaging::MSLLHOOKSTRUCT,
    last_mouse_pos: &Mutex<Option<(i32, i32)>>,
) -> Option<InputEvent> {
    use windows::Win32::UI::WindowsAndMessaging::{
        WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE,
        WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP,
    };

    match message {
        WM_MOUSEMOVE => {
            let current = (info.pt.x, info.pt.y);
            let mut previous = last_mouse_pos.lock().unwrap();
            let delta = previous
                .map(|last| (current.0 - last.0, current.1 - last.1))
                .unwrap_or((0, 0));
            *previous = Some(current);

            if delta == (0, 0) {
                None
            } else {
                Some(InputEvent::MouseMove {
                    dx: delta.0 as f64,
                    dy: delta.1 as f64,
                })
            }
        }
        WM_LBUTTONDOWN => Some(InputEvent::MouseButton {
            button: MouseButton::Left,
            pressed: true,
        }),
        WM_LBUTTONUP => Some(InputEvent::MouseButton {
            button: MouseButton::Left,
            pressed: false,
        }),
        WM_RBUTTONDOWN => Some(InputEvent::MouseButton {
            button: MouseButton::Right,
            pressed: true,
        }),
        WM_RBUTTONUP => Some(InputEvent::MouseButton {
            button: MouseButton::Right,
            pressed: false,
        }),
        WM_MBUTTONDOWN => Some(InputEvent::MouseButton {
            button: MouseButton::Middle,
            pressed: true,
        }),
        WM_MBUTTONUP => Some(InputEvent::MouseButton {
            button: MouseButton::Middle,
            pressed: false,
        }),
        WM_MOUSEWHEEL => {
            let wheel_delta = ((info.mouseData >> 16) as i16) as f64 / 120.0;
            if wheel_delta == 0.0 {
                None
            } else {
                Some(InputEvent::MouseScroll {
                    dx: 0.0,
                    dy: wheel_delta,
                })
            }
        }
        _ => None,
    }
}
