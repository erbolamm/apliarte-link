// Javier: Este es el punto de entrada del servicio — como main() en Flutter.
// El servicio corre en segundo plano sin interfaz gráfica.
// La app Flutter se conecta a él por un socket local (IPC).
//
// Flujo general:
//   1. main() arranca
//   2. Se carga la configuración (config.rs)
//   3. Se crea el socket UDP compartido — todos los módulos lo comparten (Arc<UdpSocket>)
//   4. Se crea el SwitchState compartido — estado del KVM (SENDER/RECEIVER, last_peer, etc.)
//   5. Se inicia el servidor IPC (ipc.rs) — para recibir órdenes de la UI Flutter
//   6. Se inicia el motor de red (network.rs) — para hablar con otros PCs
//   7. Se inicia el motor de input (input.rs) — para capturar e inyectar teclado/ratón

mod config;
mod input;
mod ipc;
mod network;

use anyhow::Result;
use input::state::new_shared_state;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Javier: Esto activa los logs — como debugPrint() pero con niveles (info, warn, error)
    tracing_subscriber::fmt()
        .with_env_filter("apliarte_switch=debug")
        .init();

    info!("🚀 ApliArteSwitch Core arrancando...");

    // Javier: Cargamos la configuración del archivo JSON/TOML
    // Es como leer SharedPreferences en Flutter, pero desde un archivo
    let cfg = config::load()?;
    info!("⚙️  Configuración cargada: puerto UDP {}", cfg.network.port);

    // Javier: Creamos el socket UDP aquí — un único socket compartido por red e input.
    // Arc permite compartirlo entre tareas sin copiar — como un GlobalKey en Flutter.
    let addr = format!("0.0.0.0:{}", cfg.network.port);
    let socket = Arc::new(UdpSocket::bind(&addr).await?);
    info!("📡 Socket UDP enlazado en {}", addr);

    // Javier: Estado compartido del switch KVM — contiene si estamos en modo SENDER,
    // el último peer conocido, y el contador de fallos de envío.
    // Compartido entre network::serve y input::serve mediante Arc<RwLock<...>>.
    let state = new_shared_state();

    // Javier: Creamos un "canal de mensajes" entre módulos
    // En Flutter esto sería como un Stream o un ChangeNotifier compartido
    let (event_tx, event_rx) = tokio::sync::mpsc::channel(256);

    // Javier: Arrancamos los 3 módulos de forma concurrente
    // Es como lanzar múltiples Isolates en Flutter
    tokio::select! {
        result = ipc::serve(event_tx.clone(), cfg.clone()) => {
            if let Err(e) = result {
                tracing::error!("Error en IPC: {}", e);
            }
        },
        result = network::serve(event_tx.clone(), Arc::clone(&socket), state.clone()) => {
            if let Err(e) = result {
                tracing::error!("Error en red: {}", e);
            }
        },
        result = input::serve(event_rx, cfg.clone(), Arc::clone(&socket), state.clone()) => {
            if let Err(e) = result {
                tracing::error!("Error en input: {}", e);
            }
        },
    }

    Ok(())
}
