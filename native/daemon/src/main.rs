#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

// Javier: Este es el punto de entrada del servicio — como main() en Flutter.
// El servicio corre en segundo plano sin interfaz gráfica.
// La app Flutter se conecta a él por un socket local (IPC).
//
// Flujo general:
//   1. main() arranca
//   2. Se carga la configuración (config.rs)
//   3. Se inicia el servidor IPC (ipc.rs) — para recibir órdenes de la UI Flutter
//   4. Se inicia el motor de red (network.rs) — para hablar con otros PCs
//   5. Se inicia el motor de input (input.rs) — para capturar e inyectar teclado/ratón

mod config;
mod control;
mod devices;
mod input;
mod ipc;
mod network;

use anyhow::Result;
use control::{ControlState, ControlStatus};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("apliarte_link=debug")
        .init();

    // Initialize rustls crypto provider (required for TLS)
    let _ = rustls::crypto::ring::default_provider().install_default();

    info!("🚀 ApliArte Link Core arrancando...");

    let loaded_cfg = config::load()?;
    info!(
        "⚙️  Configuración cargada: puerto TCP {}",
        loaded_cfg.network.port
    );
    let cfg = Arc::new(Mutex::new(loaded_cfg));

    // Channel: (peer_addr, frame) from transport → input engine
    let (event_tx, event_rx) =
        tokio::sync::mpsc::channel::<(SocketAddr, crate::input::protocol::Frame)>(256);

    let local_name = control::detect_local_name();
    let control_state: ControlState = Arc::new(Mutex::new(ControlStatus::new(local_name)));
    let network_cfg = cfg.lock().unwrap().clone();
    let (peers, conn_mgr) =
        network::start(event_tx.clone(), network_cfg, control_state.clone()).await?;
    let active_peer: Arc<Mutex<Option<SocketAddr>>> = Arc::new(Mutex::new(None));

    tokio::select! {
        result = ipc::serve(cfg.clone(), peers.clone(), active_peer.clone(), control_state.clone(), conn_mgr.clone()) => {
            if let Err(e) = result {
                tracing::error!("Error en IPC: {}", e);
            }
        },
        result = input::serve(event_rx, cfg.clone(), peers.clone(), active_peer, control_state.clone(), conn_mgr) => {
            if let Err(e) = result {
                tracing::error!("Error en input: {}", e);
            }
        },
    }

    Ok(())
}
