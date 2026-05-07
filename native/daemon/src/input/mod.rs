// Javier: Este módulo agrupa toda la lógica de teclado y ratón.
// Analogía Flutter: es como un GestureDetector global que captura
// y puede "emitir" eventos de input al SO.
//
// Tiene dos modos:
//   - CAPTURA: intercepta lo que tú haces (teclas, ratón) y lo envía por red
//   - INYECCIÓN: recibe eventos de otro PC y los simula en este sistema como si
//                fueran de un teclado/ratón físico conectado aquí
//
// La implementación real está en submódulos por SO:
//   - macos.rs  → usa CGEvent (API de Apple)
//   - windows.rs → usa SendInput (API Win32)
//   - linux.rs  → usa evdev/uinput (API del kernel Linux)

pub mod protocol;

// Javier: En Rust se puede compilar código diferente según el SO
// Es como if (Platform.isIOS) en Flutter pero decidido en tiempo de compilación
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use crate::config::AppConfig;
use crate::control::ControlState;
use crate::network::PeerList;
use anyhow::Result;
use protocol::InputEvent;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Receiver;

/// Arranca el motor de input.
/// Javier: Es como llamar a initState() — arranca todos los listeners de input
/// y mantiene el bucle activo escuchando al canal de eventos de red.
pub async fn serve(
    mut event_rx: Receiver<InputEvent>,
    cfg: Arc<Mutex<AppConfig>>,
    peers: PeerList,
    active_peer: Arc<Mutex<Option<SocketAddr>>>,
    control_state: ControlState,
) -> Result<()> {
    let input_cfg = cfg.lock().unwrap().input.clone();
    let data_port = cfg.lock().unwrap().network.port;
    tracing::info!(
        "⌨️  Motor de input arrancado (modo: {})",
        if input_cfg.edge_switch {
            "borde de pantalla"
        } else {
            "hotkey"
        }
    );

    #[cfg(target_os = "macos")]
    macos::start_switch_listener(
        cfg.clone(),
        peers.clone(),
        active_peer.clone(),
        control_state.clone(),
        data_port,
    )?;

    #[cfg(target_os = "windows")]
    windows::start_switch_listener(
        cfg.clone(),
        peers.clone(),
        active_peer.clone(),
        control_state.clone(),
        data_port,
    )?;

    // Javier: Bucle infinito esperando eventos recibidos por red.
    // La captura local real vive en los listeners de macOS/Windows y desde allí
    // se reenvían los InputEvent al peer activo.

    while let Some(event) = event_rx.recv().await {
        tracing::debug!("Input recibido de red: {:?}", event);

        // Javier: Según el SO, se llama a la implementación correcta
        #[cfg(target_os = "macos")]
        macos::inject(event).await?;

        #[cfg(target_os = "windows")]
        windows::inject(event).await?;

        #[cfg(target_os = "linux")]
        linux::inject(event).await?;
    }

    Ok(())
}
