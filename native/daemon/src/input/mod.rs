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
use crate::network::ConnectionManager;
use crate::network::PeerList;
use anyhow::Result;
use protocol::Frame;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Receiver;

/// ¿Tiene el proceso permiso de Accesibilidad (solo relevante en macOS)?
/// En Linux/Windows siempre se considera concedido.
pub fn accessibility_granted() -> bool {
    #[cfg(target_os = "macos")]
    {
        macos::accessibility_granted()
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// Arranca el motor de input.
pub async fn serve(
    mut event_rx: Receiver<(SocketAddr, Frame)>,
    cfg: Arc<Mutex<AppConfig>>,
    peers: PeerList,
    active_peer: Arc<Mutex<Option<SocketAddr>>>,
    control_state: ControlState,
    conn_mgr: ConnectionManager,
) -> Result<()> {
    let input_cfg = cfg.lock().unwrap().input.clone();
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
        conn_mgr.clone(),
    )?;

    #[cfg(target_os = "windows")]
    windows::start_switch_listener(
        cfg.clone(),
        peers.clone(),
        active_peer.clone(),
        control_state.clone(),
        conn_mgr.clone(),
    )?;

    #[cfg(target_os = "linux")]
    linux::start_switch_listener(
        cfg.clone(),
        peers.clone(),
        active_peer.clone(),
        control_state.clone(),
        conn_mgr.clone(),
    )?;

    // Receive frames from the transport layer and inject them locally.
    while let Some((_peer_addr, frame)) = event_rx.recv().await {
        match frame {
            Frame::Input(event) => {
                tracing::debug!("Input recibido de red: {:?}", event);

                #[cfg(target_os = "macos")]
                macos::inject(event).await?;

                #[cfg(target_os = "windows")]
                windows::inject(event).await?;

                #[cfg(target_os = "linux")]
                linux::inject(event).await?;
            }
            Frame::Control(msg) => {
                crate::network::mod_handle_control(&control_state, msg);
            }
            Frame::KeepAlive(ping) => {
                tracing::trace!("Keep-alive: {:?}", ping);
            }
            Frame::Handshake(hs) => {
                tracing::debug!("Handshake: {:?}", hs);
            }
        }
    }

    Ok(())
}
