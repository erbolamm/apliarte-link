// Javier: Este módulo agrupa toda la lógica de teclado y ratón.
// Analogía Flutter: es como un GestureDetector global que captura
// y puede "emitir" eventos de input al SO.
//
// Tiene dos modos:
//   - CAPTURA (SENDER): intercepta lo que tú haces (teclas, ratón) y lo envía por red
//     → en macOS: usa CGEventTap (capture_macos.rs)
//   - INYECCIÓN (RECEIVER): recibe eventos de otro PC y los simula en este sistema
//     → usa las APIs de cada SO (macos.rs, windows.rs, linux.rs)
//
// La implementación real está en submódulos por SO:
//   - macos.rs  → inyección con CGEvent (API de Apple)
//   - windows.rs → inyección con SendInput (API Win32)
//   - linux.rs  → inyección con evdev/uinput (API del kernel Linux)
//
// Nuevos submódulos:
//   - state.rs      → SwitchState, SharedState, CaptureEvent
//   - hotkey.rs     → parse_hotkey, HotkeyMatcher (puro, con tests)
//   - edge.rs       → is_at_edge (puro, con tests)
//   - capture_macos.rs → CGEventTap SENDER (solo macOS)

pub mod protocol;
pub mod state;
pub mod hotkey;
pub mod edge;

// Javier: En Rust se puede compilar código diferente según el SO
// Es como if (Platform.isIOS) en Flutter pero decidido en tiempo de compilación
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
pub mod capture_macos;

// Re-exportaciones para que el resto del crate acceda directamente
pub use state::{SharedState, SwitchState, new_shared_state};

use crate::config::AppConfig;
use anyhow::Result;
use protocol::InputEvent;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Receiver;

/// Arranca el motor de input.
/// Javier: Es como llamar a initState() — arranca todos los listeners de input
/// y mantiene el bucle activo escuchando al canal de eventos de red.
///
/// Nueva firma (Phase 1 / task 1.4):
///   - `socket`: socket UDP compartido (Arc) — necesario para el sender loop
///   - `state`: estado del switch KVM (Arc<RwLock<SwitchState>>) — compartido con network
pub async fn serve(
    event_rx: Receiver<InputEvent>,
    cfg: AppConfig,
    socket: Arc<UdpSocket>,
    state: SharedState,
) -> Result<()> {
    tracing::info!(
        "⌨️  Motor de input arrancado (modo: {})",
        if cfg.input.edge_switch { "borde de pantalla" } else { "hotkey" }
    );

    // Javier: El bucle de inyección (RECEIVER) corre siempre.
    // En macOS, además lanzamos el capture (SENDER) como tarea Tokio independiente.
    //
    // Diseño:
    //   - capture_macos::run corre como tokio::spawn → no bloquea inject_loop
    //   - Si el tap falla (sin permisos), capture_macos retorna Ok(()) y el RECEIVER sigue
    //   - Si el sender_loop falla con error fatal, loggea pero no mata el RECEIVER
    //   - inject_loop dirige el ciclo de vida del módulo

    #[cfg(target_os = "macos")]
    {
        // Lanzar el SENDER como tarea independiente — no esperamos su completado aquí
        let socket_for_cap = Arc::clone(&socket);
        let state_for_cap = state.clone();
        let cfg_for_cap = cfg.clone();
        tokio::spawn(async move {
            if let Err(e) = capture_macos::run(cfg_for_cap, socket_for_cap, state_for_cap).await {
                tracing::error!("Error en capture_macos SENDER: {}", e);
            }
        });
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Javier: suprimir "unused variable" warnings en targets no-macOS
        let _ = socket;
        let _ = state;
    }

    // El RECEIVER (inject_loop) dirige el ciclo de vida del módulo
    inject_loop(event_rx, cfg).await?;

    Ok(())
}

/// Bucle de inyección (RECEIVER): recibe InputEvents del canal y los inyecta en el SO.
/// Javier: Renombrado desde el bucle principal de serve() para coexistir con capture_macos.
async fn inject_loop(mut event_rx: Receiver<InputEvent>, cfg: AppConfig) -> Result<()> {
    // Javier: Bucle infinito esperando eventos de la red
    // Es como el listen() de un Stream en Flutter
    while let Some(event) = event_rx.recv().await {
        tracing::debug!("Input recibido de red: {:?}", event);

        // Javier: Según el SO, se llama a la implementación correcta
        #[cfg(target_os = "macos")]
        macos::inject(event).await?;

        #[cfg(target_os = "windows")]
        windows::inject(event).await?;

        #[cfg(target_os = "linux")]
        linux::inject(event).await?;

        // Suprimir "unused" en plataformas sin implementación
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            let _ = event;
        }

        let _ = &cfg; // Javier: cfg disponible para uso futuro en el bucle
    }

    Ok(())
}
