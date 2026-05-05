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
//
// Cambios en Phase 4 (task 4.1/4.2):
//   - serve() acepta Arc<UdpSocket> y SharedState (ya no crea su propio socket)
//   - Cada recv_from exitoso actualiza state.last_peer y resetea consecutive_send_failures
//   - send_event acepta &Arc<UdpSocket> en vez de &UdpSocket
//   - Se quita #[allow(dead_code)] de send_event

use crate::input::protocol::InputEvent;
use crate::input::state::SharedState;
use anyhow::Result;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;

/// Arranca el servidor de red.
/// Javier: Es como initState() del módulo de red — recibe el socket UDP compartido
/// creado en main.rs y se queda escuchando paquetes de otros PCs.
///
/// Cambio respecto a la versión anterior:
///   - Ya NO crea su propio socket (main.rs lo crea como Arc<UdpSocket> compartido)
///   - Actualiza state.last_peer con la IP del último PC que nos mandó datos
///   - Resetea consecutive_send_failures al recibir un paquete válido (señal de vida del peer)
pub async fn serve(
    event_tx: Sender<InputEvent>,
    socket: Arc<UdpSocket>,
    state: SharedState,
) -> Result<()> {
    tracing::info!("📡 Servidor UDP escuchando en {}", socket.local_addr()?);

    let mut buf = vec![0u8; 4096];

    loop {
        // Javier: recv_from espera un paquete UDP — es como await en Flutter
        // Se bloquea hasta que llega algo de cualquier PC de la red
        let (len, peer_addr) = socket.recv_from(&mut buf).await?;
        tracing::debug!("Paquete recibido de {}: {} bytes", peer_addr, len);

        // Javier: Intentamos deserializar el paquete como un InputEvent
        // Si falla (paquete corrupto, versión incompatible), lo ignoramos
        match serde_json::from_slice::<InputEvent>(&buf[..len]) {
            Ok(event) => {
                // Javier: Actualizamos last_peer — este PC nos mandó datos, es un peer válido.
                // También reseteamos el contador de fallos porque el peer sigue vivo.
                // Diseño D1 (last_peer como fallback) y D4 (reset en recepción).
                {
                    let mut st = state.write();
                    st.last_peer = Some(peer_addr);
                    st.consecutive_send_failures = 0;
                }

                // Javier: Mandamos el evento al módulo de input por el canal
                // Es como emit() en un Stream — el módulo de input lo recibirá
                if event_tx.send(event).await.is_err() {
                    tracing::warn!("Canal de input cerrado — el servicio se está apagando");
                    break;
                }
            }
            Err(e) => {
                tracing::warn!("Paquete ignorado (no es un InputEvent válido): {}", e);
            }
        }
    }

    Ok(())
}

/// Envía un InputEvent a un PC remoto por UDP.
/// Javier: Esta función la llama el sender_loop de capture_macos cuando el SENDER está activo.
///
/// Cambio respecto a la versión anterior:
///   - Acepta &Arc<UdpSocket> en vez de &UdpSocket (el socket es compartido)
///   - Se quitó #[allow(dead_code)] — ahora capture_macos la usa
pub async fn send_event(socket: &Arc<UdpSocket>, event: &InputEvent, target: &str) -> Result<()> {
    // Serializar el evento a JSON
    let data = serde_json::to_vec(event)?;
    socket.send_to(&data, target).await?;
    Ok(())
}
