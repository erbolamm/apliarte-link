// Javier: Implementación de input para Windows.
// Usamos SendInput — la API oficial de Windows para simular teclado y ratón.
//
// Analogía Flutter: es como escribir código nativo en Kotlin/Java
// dentro de un Platform Channel para Android/Windows.
//
// En Windows generalmente no hace falta dar permisos especiales,
// aunque en algunos casos puede pedir ejecutar como Administrador.

use crate::input::protocol::{InputEvent, MouseButton};
use anyhow::Result;

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
