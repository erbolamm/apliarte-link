// Javier: Este módulo gestiona la comunicación con otros PCs por red local (LAN).
// Protocolo MVP:
//   - Transporte: UDP (puerto 4242 por defecto)
//   - Formato: JSON (simple, legible, fácil de debugear)
//   - IP del peer: configurada manualmente en config.json (sin mDNS en MVP)
//   - Sin cifrado en MVP (DTLS llega en Fase 2)

use crate::config::AppConfig;
use crate::input::protocol::InputEvent;
use anyhow::Result;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;

/// Arranca el receptor UDP — escucha eventos del PC remoto y los envía al canal de input.
/// Javier: Esta función la usa el PC en MODO RECEIVER (el que recibe el input).
pub async fn serve(event_tx: Sender<InputEvent>, cfg: AppConfig) -> Result<()> {
    let addr = format!("0.0.0.0:{}", cfg.network.port);
    let socket = UdpSocket::bind(&addr).await?;
    tracing::info!("📡 Receptor UDP escuchando en {} (esperando eventos del PC remoto)", addr);

    let mut buf = vec![0u8; 4096];

    loop {
        let (len, peer_addr) = socket.recv_from(&mut buf).await?;
        tracing::debug!("Paquete recibido de {}: {} bytes", peer_addr, len);

        match serde_json::from_slice::<InputEvent>(&buf[..len]) {
            Ok(event) => {
                if event_tx.send(event).await.is_err() {
                    tracing::warn!("Canal de input cerrado — deteniendo receptor UDP");
                    break;
                }
            }
            Err(e) => {
                tracing::warn!("Paquete inválido de {}: {}", peer_addr, e);
            }
        }
    }

    Ok(())
}

/// Cliente UDP para enviar eventos al PC remoto.
/// Javier: Esta función la usa el PC en MODO SENDER (el que tiene el teclado/ratón).
pub struct UdpSender {
    socket: UdpSocket,
    peer_addr: String,
}

impl UdpSender {
    /// Crea un socket UDP listo para enviar al peer configurado.
    pub async fn new(cfg: &AppConfig) -> Result<Self> {
        let peer_ip = cfg.network.peer_ip.as_deref().ok_or_else(|| {
            anyhow::anyhow!(
                "No hay IP configurada para el PC remoto.\n\
                 Edita config.json y añade: \"peer_ip\": \"192.168.1.X\"\n\
                 (sustituye X por la IP del otro PC en tu red)"
            )
        })?;

        let peer_addr = format!("{}:{}", peer_ip, cfg.network.port);

        // Bind en puerto 0 = el SO elige un puerto libre para enviar
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        tracing::info!("📤 Sender UDP listo → {}", peer_addr);

        Ok(Self { socket, peer_addr })
    }

    /// Envía un InputEvent al PC remoto.
    pub async fn send(&self, event: &InputEvent) -> Result<()> {
        let data = serde_json::to_vec(event)?;
        self.socket.send_to(&data, &self.peer_addr).await?;
        Ok(())
    }
}

/// Función de conveniencia para envío puntual (sin mantener socket abierto)
#[allow(dead_code)]
pub async fn send_event(socket: &UdpSocket, event: &InputEvent, target: &str) -> Result<()> {
    let data = serde_json::to_vec(event)?;
    socket.send_to(&data, target).await?;
    Ok(())
}

