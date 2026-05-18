// Network module: peer discovery + TCP+TLS transport.
//
// Architecture:
//   discovery (UDP 4244) — finds peers on the LAN
//   transport (TCP 4242) — reliable encrypted input/control channel
//
// The old UDP+JSON input channel has been replaced by transport.rs.

pub mod discovery;
pub mod transport;

pub use discovery::{PeerEntry, PeerList};
pub use transport::ConnectionManager;

use crate::config::AppConfig;
use crate::control::ControlState;
use crate::input::protocol::Frame;
use anyhow::Result;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;

/// Start the network layer: discovery + TCP listener + connection manager.
/// Returns the peer list and connection manager for other modules to use.
pub async fn start(
    event_tx: Sender<(SocketAddr, Frame)>,
    cfg: AppConfig,
    control_state: ControlState,
) -> Result<(PeerList, ConnectionManager)> {
    let local_name = control_state.lock().unwrap().local_name.clone();

    // Discovery (UDP 4244) — unchanged
    let peers = discovery::start(cfg.network.port, local_name.clone()).await?;

    // Transport (TCP 4242)
    let listener = TcpListener::bind(format!("0.0.0.0:{}", cfg.network.port)).await?;
    tracing::info!("📡 Servidor TCP escuchando en 0.0.0.0:{}", cfg.network.port);

    let conn_mgr = transport::ConnectionPool::new(local_name, event_tx)?;

    // Spawn accept loop
    let accept_mgr = conn_mgr.clone();
    tokio::spawn(async move {
        accept_mgr.accept_loop(listener).await;
    });

    Ok((peers, conn_mgr))
}

/// Handle an inbound control frame from the transport layer.
pub fn mod_handle_control(
    control_state: &ControlState,
    message: crate::input::protocol::ControlMessage,
) {
    let mut state = control_state.lock().unwrap();
    match message {
        crate::input::protocol::ControlMessage::Assign { source_name } => {
            state.mark_destination_assigned(&source_name);
        }
        crate::input::protocol::ControlMessage::Release { source_name } => {
            state.mark_destination_released(&source_name);
        }
    }
}
