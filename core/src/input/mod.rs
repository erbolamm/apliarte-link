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
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

use crate::config::AppConfig;
use crate::network::PeerList;
use anyhow::Result;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Receiver;
use protocol::InputEvent;

/// Arranca el motor de input.
/// Javier: Es como llamar a initState() — arranca todos los listeners de input
/// y mantiene el bucle activo escuchando al canal de eventos de red.
pub async fn serve(mut event_rx: Receiver<InputEvent>, cfg: AppConfig, peers: PeerList) -> Result<()> {
    tracing::info!("⌨️  Motor de input arrancado (modo: {})",
        if cfg.input.edge_switch { "borde de pantalla" } else { "hotkey" }
    );

    // Javier: Bucle infinito esperando eventos de la red
    // Es como el listen() de un Stream en Flutter
    // Socket UDP para enviar eventos a los peers descubiertos
    let send_socket = UdpSocket::bind("0.0.0.0:0").await?;

    while let Some(event) = event_rx.recv().await {
        tracing::debug!("Input recibido de red: {:?}", event);

        // Reenviar a todos los peers descubiertos (modo SENDER)
        {
            let list = peers.lock().unwrap();
            if !list.is_empty() {
                if let Ok(data) = serde_json::to_vec(&event) {
                    for peer in list.iter() {
                        let _ = send_socket.send_to(&data, peer).await;
                    }
                }
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
