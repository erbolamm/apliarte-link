// Javier: Este módulo define el estado compartido del switch KVM.
// Analogía Flutter: es como un ChangeNotifier o un Bloc state — contiene
// todo lo que necesita saber el sistema para decidir si capturar o inyectar.
//
// El estado es leído y escrito desde varios contextos:
//   - El callback del CGEventTap (thread síncrono de CFRunLoop)
//   - La tarea Tokio del sender_loop
//   - El módulo de red (network::serve) al recibir paquetes entrantes
//
// Usamos parking_lot::RwLock porque:
//   - El callback CGEventTap NO puede hacer .await (no está en un contexto async)
//   - parking_lot es más rápido que std::sync::RwLock y no tiene poisoning
//   - La tarea Tokio también puede hacer read/write síncrono (el lock es trivial en tiempo)

use std::net::SocketAddr;
use std::sync::Arc;

/// Estado del switch KVM compartido entre el tap, el sender y el receiver.
/// Javier: Piensa en esto como el "model" central del sistema — una sola fuente de verdad.
#[derive(Debug, Default)]
pub struct SwitchState {
    /// Javier: Cuando true, el SENDER está activo — interceptamos input local y lo mandamos por UDP.
    /// Cuando false, somos solo RECEIVER.
    pub active: bool,

    /// Javier: Dirección del último peer que nos mandó un paquete UDP.
    /// Se usa como destino de envío si cfg.network.peer no está configurado (diseño D1).
    pub last_peer: Option<SocketAddr>,

    /// Javier: Contador de fallos UDP consecutivos.
    /// Si llega a 10, el sender se auto-desactiva (diseño D4).
    /// Se resetea a 0 cuando un envío tiene éxito o cuando recibimos un paquete entrante.
    pub consecutive_send_failures: u32,
}

/// Javier: Tipo alias para el estado compartido — un Arc<RwLock<SwitchState>>.
/// El Arc permite compartirlo entre threads/tasks sin copiar.
/// El RwLock permite múltiples lectores concurrentes o un único escritor.
pub type SharedState = Arc<parking_lot::RwLock<SwitchState>>;

/// Javier: Tipo alias para los mensajes que pasan del callback CGEventTap a la tarea Tokio.
/// Lleva el evento de input y la dirección destino ya resuelta (evita resolver en la tarea).
pub type CaptureEvent = (crate::input::protocol::InputEvent, SocketAddr);

/// Crea un nuevo SharedState con valores por defecto.
/// Javier: Como Provider.of<SwitchState>(context) — acceso al estado compartido inicial.
pub fn new_shared_state() -> SharedState {
    Arc::new(parking_lot::RwLock::new(SwitchState::default()))
}

/// Resuelve el peer destino a partir de la configuración y el estado actual.
///
/// Javier: Implementa la lógica D1 del diseño:
///   1. Si hay un peer configurado explícitamente en cfg → parsearlo y usarlo
///   2. Si no → usar el último peer que nos mandó un paquete (last_peer)
///   3. Si ninguno → None (no enviar nada)
///
/// Esta función es pura y testable — no toca el estado, solo lo lee.
pub fn resolve_target_peer(
    cfg_peer: Option<&str>,
    last_peer: Option<SocketAddr>,
) -> Option<SocketAddr> {
    if let Some(peer_str) = cfg_peer {
        // Javier: Si hay peer configurado, lo intentamos parsear como SocketAddr
        // Si falla el parseo, caemos al last_peer como fallback
        match peer_str.parse::<SocketAddr>() {
            Ok(addr) => Some(addr),
            Err(_) => {
                tracing::warn!(
                    "cfg.network.peer '{}' no es una SocketAddr válida, usando last_peer",
                    peer_str
                );
                last_peer
            }
        }
    } else {
        last_peer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Javier: Tests para resolve_target_peer — pura función sin I/O.
    // Spec A1, A5, F1, F2 + diseño D1.

    #[test]
    fn test_resolve_cfg_peer_overrides_last_peer() {
        // D1: si hay peer en config, se usa ese aunque exista last_peer
        let last = "10.0.0.1:4242".parse().ok();
        let result = resolve_target_peer(Some("192.168.1.50:4242"), last);
        assert_eq!(
            result,
            Some("192.168.1.50:4242".parse().unwrap()),
            "cfg.peer debe dominar sobre last_peer"
        );
    }

    #[test]
    fn test_resolve_falls_back_to_last_peer_when_no_cfg() {
        // D1: sin peer en config, se usa last_peer
        let last: Option<SocketAddr> = "10.0.0.2:4242".parse().ok();
        let result = resolve_target_peer(None, last);
        assert_eq!(
            result,
            Some("10.0.0.2:4242".parse().unwrap()),
            "debe caer al last_peer cuando no hay cfg.peer"
        );
    }

    #[test]
    fn test_resolve_returns_none_when_neither() {
        // A5: sin peer configurado y sin last_peer → None (no enviar)
        let result = resolve_target_peer(None, None);
        assert_eq!(result, None, "debe devolver None cuando no hay ningún peer conocido");
    }

    #[test]
    fn test_resolve_invalid_cfg_peer_falls_back_to_last_peer() {
        // Robustez: peer mal formado en config cae al last_peer
        let last: Option<SocketAddr> = "10.0.0.3:4242".parse().ok();
        let result = resolve_target_peer(Some("not-an-address"), last);
        assert_eq!(
            result,
            Some("10.0.0.3:4242".parse().unwrap()),
            "debe caer a last_peer cuando cfg.peer no es parseable"
        );
    }

    #[test]
    fn test_switch_state_default_is_inactive() {
        // El estado inicial debe ser inactivo — el sistema arranca como RECEIVER puro
        let state = SwitchState::default();
        assert!(!state.active, "estado inicial debe ser inactivo (RECEIVER only)");
        assert!(state.last_peer.is_none(), "last_peer debe ser None al iniciar");
        assert_eq!(state.consecutive_send_failures, 0);
    }

    #[test]
    fn test_new_shared_state_is_sendable() {
        // Verifica que SharedState es Send+Sync (compilación — si compila, pasa)
        let state = new_shared_state();
        let state_clone = Arc::clone(&state);
        let _ = std::thread::spawn(move || {
            let _guard = state_clone.read();
        });
        let _guard = state.read();
    }
}
