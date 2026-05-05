// Javier: Este módulo agrupa toda la lógica de teclado y ratón.
// Tiene DOS modos de operación que pueden cambiar en tiempo real:
//
//   MODO LOCAL (por defecto al arrancar):
//     - El input va a este PC normalmente
//     - Vigilamos el borde de pantalla
//     - Cuando el ratón llega al borde → pasamos a MODO SENDER
//
//   MODO SENDER (estamos "dentro" del PC remoto):
//     - Capturamos TODO el input y lo enviamos por UDP al peer
//     - El input NO llega a ninguna app de este Mac
//     - Hotkey Ctrl+Alt+Tab → volvemos a MODO LOCAL
//
//   MODO RECEIVER (pasivo, siempre activo en paralelo):
//     - El servidor UDP recibe eventos del PC remoto
//     - Los inyectamos en este SO como si fueran del teclado/ratón físico

pub mod protocol;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub mod capture_windows;
#[cfg(target_os = "linux")]
mod linux;

use crate::config::AppConfig;
use crate::network::UdpSender;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use protocol::InputEvent;

/// Arranca el motor de input completo (captura + inyección).
/// `event_rx` recibe los eventos que llegan de la RED (modo receiver).
pub async fn serve(mut event_rx: Receiver<InputEvent>, cfg: AppConfig) -> Result<()> {
    tracing::info!(
        "⌨️  Motor de input arrancando (borde: {}, peer: {})",
        cfg.input.edge_side,
        cfg.network.peer_ip.as_deref().unwrap_or("no configurado"),
    );

    // ── Sender UDP ────────────────────────────────────────────────────────────
    let sender: Option<Arc<UdpSender>> = match UdpSender::new(&cfg).await {
        Ok(s) => {
            tracing::info!("✅ Sender UDP listo — modo SENDER disponible");
            Some(Arc::new(s))
        }
        Err(e) => {
            tracing::warn!("⚠️  Sin sender UDP: {}", e);
            None
        }
    };

    // ── macOS: arrancar captura CGEventTap ────────────────────────────────────
    #[cfg(target_os = "macos")]
    {
        use std::sync::atomic::AtomicBool;
        let (capture_tx, mut capture_rx) =
            tokio::sync::mpsc::channel::<macos::CapturedEvent>(256);
        let running = Arc::new(AtomicBool::new(true));
        macos::start_capture(capture_tx, cfg.input.edge_side.clone(), running);

        loop {
            tokio::select! {
                // Eventos de la RED → inyectar localmente (modo RECEIVER)
                maybe = event_rx.recv() => {
                    match maybe {
                        Some(event) => {
                            if let Err(e) = macos::inject(event).await {
                                tracing::error!("Inyección macOS fallida: {}", e);
                            }
                        }
                        None => {
                            tracing::info!("Canal de red cerrado — deteniendo input");
                            break;
                        }
                    }
                }

                // Eventos capturados localmente → enviar al peer (modo SENDER)
                maybe = capture_rx.recv() => {
                    match maybe {
                        Some(macos::CapturedEvent::Forward(event)) => {
                            if let Some(ref s) = sender {
                                if let Err(e) = s.send(&event).await {
                                    tracing::error!("Error enviando UDP: {}", e);
                                }
                            }
                        }
                        Some(macos::CapturedEvent::EdgeReached) => {
                            tracing::info!("🔀 Borde alcanzado — input al PC remoto");
                        }
                        Some(macos::CapturedEvent::HotkeyPressed) => {
                            tracing::info!("⌨️  Hotkey → volviendo al PC local");
                        }
                        None => break,
                    }
                }
            }
        }
        return Ok(());
    }

    // ── Windows: sender + receiver ─────────────────────────────────────────
    #[cfg(target_os = "windows")]
    {
        use std::sync::atomic::AtomicBool;
        let (capture_tx, mut capture_rx) =
            tokio::sync::mpsc::channel::<capture_windows::CapturedEvent>(256);
        let running = Arc::new(AtomicBool::new(true));
        capture_windows::start_capture(capture_tx, cfg.input.edge_side.clone(), running);

        loop {
            tokio::select! {
                // Eventos de la RED → inyectar localmente (modo RECEIVER)
                maybe = event_rx.recv() => {
                    match maybe {
                        Some(event) => {
                            if let Err(e) = windows::inject(event).await {
                                tracing::error!("Inyección Windows fallida: {}", e);
                            }
                        }
                        None => {
                            tracing::info!("Canal de red cerrado — deteniendo input");
                            break;
                        }
                    }
                }

                // Eventos capturados localmente → enviar al peer (modo SENDER)
                maybe = capture_rx.recv() => {
                    match maybe {
                        Some(capture_windows::CapturedEvent::Forward(event)) => {
                            if let Some(ref s) = sender {
                                if let Err(e) = s.send(&event).await {
                                    tracing::error!("Error enviando UDP: {}", e);
                                }
                            }
                        }
                        Some(capture_windows::CapturedEvent::EdgeReached) => {
                            tracing::info!("🔀 Borde alcanzado — input al PC remoto");
                        }
                        Some(capture_windows::CapturedEvent::HotkeyPressed) => {
                            tracing::info!("⌨️  Hotkey → volviendo al PC local");
                        }
                        None => break,
                    }
                }
            }
        }
        return Ok(());
    }

    // ── Linux: solo modo RECEIVER ──────────────────────────────────────────
    while let Some(event) = event_rx.recv().await {
        tracing::debug!("← Red: inyectando {:?}", event);

        #[cfg(target_os = "linux")]
        if let Err(e) = linux::inject(event).await {
            tracing::error!("Inyección Linux fallida: {}", e);
        }

        #[cfg(not(target_os = "linux"))]
        let _ = event;
    }

    Ok(())
}

