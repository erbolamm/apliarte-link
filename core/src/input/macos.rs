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
    tokio::task::spawn_blocking(move || {
        inject_sync(event)
    }).await??;
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
            // Javier: Fix del bug MouseMove (Spec D1):
            // Antes: CGPoint::new(dx, dy) — trataba los deltas como coordenadas absolutas.
            //        Causaba teleportación del cursor al punto (dx, dy) en pantalla.
            // Ahora: Leemos la posición actual del cursor y le sumamos el delta.
            //        Es como currentOffset + Offset(dx, dy) en Flutter.
            //
            // MANUAL TEST:
            //   1. En macOS, arrancar apliarte-switch en modo RECEIVER
            //   2. Desde otro PC, enviar MouseMove { dx: 50.0, dy: 0.0 } via UDP
            //   3. Verificar que el cursor se mueve 50px a la derecha desde su posición actual
            //   4. (Comportamiento previo incorrecto: el cursor saltaba al punto (50, 0))
            //
            // Obtenemos la posición actual del cursor creando un evento de prueba.
            // CGEvent::new() crea un evento genérico vacío del que podemos leer location().
            let probe = CGEvent::new(source.clone())
                .map_err(|_| anyhow::anyhow!("No se pudo crear CGEvent probe para leer posición del cursor"))?;
            let current = probe.location(); // CGPoint absoluto actual del cursor

            // Calculamos la posición objetivo sumando el delta
            let target = CGPoint::new(current.x + dx, current.y + dy);

            let evt = CGEvent::new_mouse_event(
                source,
                CGEventType::MouseMoved,
                target,
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

            let event_type = match (button, pressed) {
                (MouseButton::Left, true) => CGEventType::LeftMouseDown,
                (MouseButton::Left, false) => CGEventType::LeftMouseUp,
                (MouseButton::Right, true) => CGEventType::RightMouseDown,
                (MouseButton::Right, false) => CGEventType::RightMouseUp,
                _ => CGEventType::OtherMouseDown,
            };

            // Javier: Para eventos de botón, usar la posición actual del cursor
            // En vez de (0,0) que manda clicks a la esquina superior-izquierda
            let probe = CGEvent::new(source.clone())
                .map_err(|_| anyhow::anyhow!("No se pudo leer posición del cursor para click"))?;
            let current_pos = probe.location();

            let evt = CGEvent::new_mouse_event(
                source,
                event_type,
                current_pos,
                btn,
            ).map_err(|_| anyhow::anyhow!("Error creando evento de click"))?;

            evt.post(CGEventTapLocation::HID);
        }

        InputEvent::MouseScroll { dx, dy } => {
            // TODO: implementar scroll con CGEventCreateScrollWheelEvent
            tracing::debug!("Scroll: dx={} dy={}", dx, dy);
        }

        InputEvent::KeyPress { key_code, pressed, modifiers: _ } => {
            // Javier: CGEvent para teclado — key_code es el código de tecla virtual
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
