// Javier: Este módulo gestiona la comunicación con otros PCs por red local (LAN).
// Analogía Flutter: es como el http.dart o Dio — pero en vez de HTTP usa UDP,
// que es más rápido (sin confirmaciones de entrega) para input en tiempo real.
//
// Protocolo:
//   - Transporte: UDP (puerto 4242 por defecto)
//   - Formato mensajes: JSON (en el futuro: binario con MessagePack para menos latencia)
//   - Descubrimiento: mDNS (los PCs se encuentran solos en la red local, sin configurar IPs)
//   - Encriptación: DTLS (UDP cifrado) — habilitado por defecto
//
// Roles (como cliente-servidor pero simétrico):
//   - SENDER: el PC que tiene el foco del teclado/ratón en este momento
//              Captura input y envía paquetes UDP
//   - RECEIVER: los otros PCs
//                Recibe paquetes UDP e inyecta el input en su SO

pub mod discovery;
pub use discovery::{PeerEntry, PeerList};

use crate::control::ControlState;
use crate::config::AppConfig;
use crate::input::protocol::{ControlMessage, InputEvent, NetworkPacket};
use anyhow::Result;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;

/// Arranca el servidor de red y el descubrimiento de peers.
/// Devuelve la lista de peers para que otros módulos la usen.
pub async fn start(
    event_tx: Sender<InputEvent>,
    cfg: AppConfig,
    control_state: ControlState,
) -> Result<PeerList> {
    let local_name = control_state.lock().unwrap().local_name.clone();
    let peers = discovery::start(cfg.network.port, local_name).await?;
    tokio::spawn(serve(event_tx, cfg, control_state));
    Ok(peers)
}

async fn serve(event_tx: Sender<InputEvent>, cfg: AppConfig, control_state: ControlState) -> Result<()> {
    let addr = format!("0.0.0.0:{}", cfg.network.port);
    let socket = UdpSocket::bind(&addr).await?;
    tracing::info!("📡 Servidor UDP escuchando en {}", addr);

    let mut buf = vec![0u8; 4096];

    loop {
        // Javier: recv_from espera un paquete UDP — es como await en Flutter
        // Se bloquea hasta que llega algo de cualquier PC de la red
        let (len, peer_addr) = socket.recv_from(&mut buf).await?;
        tracing::debug!("Paquete recibido de {}: {} bytes", peer_addr, len);

        // Javier: Intentamos deserializar el paquete como un InputEvent
        // Si falla (paquete corrupto, versión incompatible), lo ignoramos
        match serde_json::from_slice::<NetworkPacket>(&buf[..len]) {
            Ok(NetworkPacket::Input { event }) => {
                if event_tx.send(event).await.is_err() {
                    tracing::warn!("Canal de input cerrado — el servicio se está apagando");
                    break;
                }
            }
            Ok(NetworkPacket::Control { message }) => {
                handle_control_message(&control_state, message);
            }
            Err(_) => match serde_json::from_slice::<InputEvent>(&buf[..len]) {
                Ok(event) => {
                    if event_tx.send(event).await.is_err() {
                        tracing::warn!("Canal de input cerrado — el servicio se está apagando");
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("Paquete ignorado (no es un paquete válido): {}", e);
                }
            },
        }
    }

    Ok(())
}

fn handle_control_message(control_state: &ControlState, message: ControlMessage) {
    let mut state = control_state.lock().unwrap();
    match message {
        ControlMessage::Assign { source_name } => {
            state.mark_destination_assigned(&source_name);
        }
        ControlMessage::Release { source_name } => {
            state.mark_destination_released(&source_name);
        }
    }
}

/// Envía un InputEvent a un PC remoto por UDP
/// Javier: Esta función la llama el módulo de captura cuando el ratón
/// llega al borde de la pantalla — es el "envío" del evento al otro PC
#[allow(dead_code)]
pub async fn send_event(socket: &UdpSocket, event: &InputEvent, target: &str) -> Result<()> {
    let packet = NetworkPacket::Input {
        event: event.clone(),
    };
    let data = serde_json::to_vec(&packet)?;
    socket.send_to(&data, target).await?;
    Ok(())
}
