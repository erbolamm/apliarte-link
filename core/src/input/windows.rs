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
    tokio::task::spawn_blocking(move || {
        inject_sync(event)
    }).await??;
    Ok(())
}

fn inject_sync(event: InputEvent) -> Result<()> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_MOUSE, INPUT_KEYBOARD,
        MOUSEEVENTF_MOVE, MOUSEEVENTF_ABSOLUTE,
        MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
        MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
        KEYEVENTF_KEYUP, MOUSEINPUT, KEYBDINPUT,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

    let input = match event {
        InputEvent::MouseMove { dx, dy } => {
            // dx/dy son coordenadas absolutas en píxeles del SO emisor.
            // Windows SendInput con MOUSEEVENTF_ABSOLUTE espera coordenadas
            // normalizadas de 0 a 65535. Mapeamos usando el tamaño de nuestra pantalla.
            let sw = unsafe { GetSystemMetrics(SM_CXSCREEN) }.max(1) as f64;
            let sh = unsafe { GetSystemMetrics(SM_CYSCREEN) }.max(1) as f64;
            let nx = ((dx / sw) * 65535.0).clamp(0.0, 65535.0) as i32;
            let ny = ((dy / sh) * 65535.0).clamp(0.0, 65535.0) as i32;
            INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: nx,
                        dy: ny,
                        dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
                        ..Default::default()
                    },
                },
            }
        }

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

        InputEvent::KeyPress { key_code, pressed, .. } => {
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(key_code as u16),
                        dwFlags: if pressed { Default::default() } else { KEYEVENTF_KEYUP },
                        ..Default::default()
                    },
                },
            }
        }

        InputEvent::MouseScroll { dy, .. } => {
            INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    mi: MOUSEINPUT {
                        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_WHEEL,
                        mouseData: (dy * 120.0) as u32,
                        ..Default::default()
                    },
                },
            }
        }
    };

    // Javier: SendInput envía el array de inputs al SO
    unsafe {
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }

    Ok(())
}
