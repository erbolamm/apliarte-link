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
