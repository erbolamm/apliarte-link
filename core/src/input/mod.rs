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
use anyhow::Result;
use protocol::InputEvent;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Receiver;

/// Arranca el motor de input.
/// Javier: Es como llamar a initState() — arranca todos los listeners de input
/// y mantiene el bucle activo escuchando al canal de eventos de red.
pub async fn serve(
    mut event_rx: Receiver<InputEvent>,
    cfg: AppConfig,
    active_peer: Arc<Mutex<Option<SocketAddr>>>,
) -> Result<()> {
    tracing::info!(
        "⌨️  Motor de input arrancado (modo: {})",
        if cfg.input.edge_switch {
            "borde de pantalla"
        } else {
            "hotkey"
        }
    );

    // Javier: Bucle infinito esperando eventos de la red
    // Es como el listen() de un Stream en Flutter
    // Socket UDP para enviar eventos a los peers descubiertos
    let send_socket = UdpSocket::bind("0.0.0.0:0").await?;

    while let Some(event) = event_rx.recv().await {
        tracing::debug!("Input recibido de red: {:?}", event);

        // Reenviar solo al peer activo. Un KVM controla un ordenador cada vez.
        let target = {
            let current_active = active_peer.lock().unwrap();
            *current_active
        };
        if let Some(peer) = target {
            if let Ok(data) = serde_json::to_vec(&event) {
                let _ = send_socket.send_to(&data, peer).await;
            }
        }

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
