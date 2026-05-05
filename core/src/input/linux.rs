// Javier: Implementación de input para Linux.
// Usamos uinput — un módulo del kernel Linux que permite crear dispositivos de input virtuales.
//
// Analogía Flutter: es como escribir código nativo en C para un Platform Channel de Linux.
//
// Requisito: el usuario necesita tener acceso a /dev/uinput
// Se puede dar con: sudo usermod -a -G input $USER (y reiniciar sesión)
//
// NOTA: También hay diferencia entre X11 y Wayland:
//   - X11: más simple, funciona con XTest
//   - Wayland: más restrictivo, necesita libei o wlroots protocols
//   Esta implementación usa uinput (funciona en ambos)

use crate::input::protocol::{InputEvent, MouseButton};
use anyhow::Result;

/// Inyecta un evento de input en Linux vía uinput
pub async fn inject(event: InputEvent) -> Result<()> {
    tokio::task::spawn_blocking(move || inject_sync(event)).await??;
    Ok(())
}

fn inject_sync(event: InputEvent) -> Result<()> {
    // Javier: Por ahora usamos el crate evdev para escribir en /dev/uinput
    // Es un TODO que se completará en Fase 3 del plan de desarrollo

    match event {
        InputEvent::MouseMove { dx, dy } => {
            tracing::debug!("[Linux] MouseMove dx={} dy={}", dx, dy);
            // TODO Fase 3: Implementar con evdev REL_X / REL_Y
        }
        InputEvent::MouseButton { button, pressed } => {
            tracing::debug!("[Linux] MouseButton {:?} pressed={}", button, pressed);
            // TODO Fase 3: Implementar con evdev BTN_LEFT, BTN_RIGHT
        }
        InputEvent::MouseScroll { dx, dy } => {
            tracing::debug!("[Linux] Scroll dx={} dy={}", dx, dy);
            // TODO Fase 3: Implementar con evdev REL_WHEEL
        }
        InputEvent::KeyPress {
            key_code, pressed, ..
        } => {
            tracing::debug!("[Linux] KeyPress code={} pressed={}", key_code, pressed);
            // TODO Fase 3: Implementar con evdev EV_KEY
        }
    }

    Ok(())
}
