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
mod input;
mod ipc;
mod network;

use anyhow::Result;
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

    // Javier: Creamos un "canal de mensajes" entre módulos
    // En Flutter esto sería como un Stream o un ChangeNotifier compartido
    let (event_tx, event_rx) = tokio::sync::mpsc::channel(256);

    // Javier: Arrancamos la red primero para tener los peers disponibles desde el inicio
    // Tanto IPC como input comparten el mismo Arc<Mutex<Vec>> — si un peer aparece, ambos lo ven
    let peers = network::start(event_tx.clone(), cfg.clone()).await?;

    tokio::select! {
        result = ipc::serve(event_tx.clone(), cfg.clone(), peers.clone()) => {
            if let Err(e) = result {
                tracing::error!("Error en IPC: {}", e);
            }
        },
        result = input::serve(event_rx, cfg.clone(), peers) => {
            if let Err(e) = result {
                tracing::error!("Error en input: {}", e);
            }
        },
    }

    Ok(())
}
