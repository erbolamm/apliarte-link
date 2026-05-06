// Javier: Implementación de input para macOS.
// Usamos CGEvent — la API oficial de Apple para simular teclado y ratón.
//
// Analogía Flutter: es como escribir código en Swift en un Platform Channel.
// Rust lo llama directamente a través de los bindings de core-graphics y core-foundation.
//
// Para que funcione, el usuario debe dar permiso en:
//   Sistema → Privacidad y Seguridad → Accesibilidad → apliarte-switch ✅
//   Sistema → Privacidad y Seguridad → Monitor de Teclado → apliarte-switch ✅

use crate::input::protocol::{InputEvent, MouseButton};
use anyhow::Result;

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
        InputEvent::MouseMove { dx, dy } => {
            // Javier: Obtenemos la posición actual del ratón y le sumamos el delta
            // Es como currentOffset + Offset(dx, dy) en Flutter
            let current = CGEvent::new_mouse_event(
                source.clone(),
                CGEventType::MouseMoved,
                CGPoint::new(dx, dy), // TODO: calcular posición absoluta
                CGMouseButton::Left,
            )
            .map_err(|_| anyhow::anyhow!("Error creando evento de ratón"))?;

            current.post(CGEventTapLocation::HID);
        }

        InputEvent::MouseButton { button, pressed } => {
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
                _ => CGEventType::OtherMouseDown,
            };

            let evt = CGEvent::new_mouse_event(source, event_type, CGPoint::new(0.0, 0.0), btn)
                .map_err(|_| anyhow::anyhow!("Error creando evento de click"))?;

            evt.post(CGEventTapLocation::HID);
        }

        InputEvent::MouseScroll { dx, dy } => {
            // TODO: implementar scroll con CGEventCreateScrollWheelEvent
            tracing::debug!("Scroll: dx={} dy={}", dx, dy);
        }

        InputEvent::KeyPress {
            key_code,
            pressed,
            modifiers: _,
        } => {
            // Javier: CGEvent para teclado — key_code es el código de tecla virtual
            let evt = CGEvent::new_keyboard_event(source, key_code as u16, pressed)
                .map_err(|_| anyhow::anyhow!("Error creando evento de teclado"))?;

            evt.post(CGEventTapLocation::HID);
        }
    }

    Ok(())
}
