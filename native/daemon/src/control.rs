use crate::config::InputConfig;
use crate::input::protocol::{ControlMessage, Frame, InputEvent, MouseButton};
use crate::network::ConnectionManager;
use crate::network::PeerList;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub type ControlState = Arc<Mutex<ControlStatus>>;
const COMMAND_WINDOW: Duration = Duration::from_millis(650);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferNotice {
    pub id: u64,
    pub role: String,
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct ControlStatus {
    pub local_name: String,
    pub assigned_here: bool,
    pub controller_name: Option<String>,
    pub transfer_notice: Option<TransferNotice>,
    next_notice_id: u64,
}

impl ControlStatus {
    pub fn new(local_name: String) -> Self {
        Self {
            local_name,
            assigned_here: false,
            controller_name: None,
            transfer_notice: None,
            next_notice_id: 0,
        }
    }

    pub fn mark_source_transfer(&mut self, target_label: &str) {
        self.assigned_here = false;
        self.controller_name = None;
        self.transfer_notice = Some(self.notice(
            "source",
            "Ratón y teclado asignados",
            format!("Ahora controlás {} desde este ordenador.", target_label),
        ));
    }

    pub fn mark_local_control(&mut self) {
        self.assigned_here = false;
        self.controller_name = None;
        self.transfer_notice = Some(self.notice(
            "local",
            "Ratón y teclado conectados aquí",
            "Volviste a controlar este ordenador.".to_string(),
        ));
    }

    pub fn mark_destination_assigned(&mut self, source_label: &str) {
        self.assigned_here = true;
        self.controller_name = Some(source_label.to_string());
        self.transfer_notice = Some(self.notice(
            "destination",
            "Ratón y teclado conectados aquí",
            format!(
                "Este ordenador está recibiendo control desde {}.",
                source_label
            ),
        ));
    }

    pub fn mark_destination_released(&mut self, source_label: &str) {
        self.assigned_here = false;
        self.controller_name = None;
        self.transfer_notice = Some(self.notice(
            "destination",
            "Control remoto liberado",
            format!("{} dejó de controlar este ordenador.", source_label),
        ));
    }

    fn notice(&mut self, role: &str, title: &str, detail: String) -> TransferNotice {
        self.next_notice_id += 1;
        TransferNotice {
            id: self.next_notice_id,
            role: role.to_string(),
            title: title.to_string(),
            detail,
        }
    }
}

pub fn detect_local_name() -> String {
    hostname::get()
        .ok()
        .and_then(|name| name.into_string().ok())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| "Este ordenador".to_string())
}

#[derive(Debug, Default)]
pub struct CommandDetector {
    mouse_hits: Vec<Instant>,
    keyboard_hits: Vec<Instant>,
}

impl CommandDetector {
    pub fn reset(&mut self) {
        self.mouse_hits.clear();
        self.keyboard_hits.clear();
    }

    pub fn register_mouse(&mut self, button: MouseButton, cfg: &InputConfig) -> bool {
        if !matches_mouse_command(&cfg.mouse_switch_command, &button) {
            return false;
        }

        record_hits(&mut self.mouse_hits)
    }

    pub fn register_keyboard_match(&mut self, matched: bool) -> bool {
        if !matched {
            return false;
        }

        record_hits(&mut self.keyboard_hits)
    }
}

pub fn cycle_active_peer(
    peers: &PeerList,
    active_peer: &Arc<Mutex<Option<SocketAddr>>>,
    control_state: &ControlState,
    conn_mgr: &ConnectionManager,
) -> bool {
    let previous = current_active_peer(active_peer);
    let next = next_connected_peer(peers, previous);

    if previous == next {
        return false;
    }

    set_active_peer_value(active_peer, next);
    apply_transfer(previous, next, peers, control_state, conn_mgr);
    true
}

pub fn set_active_peer(
    next: SocketAddr,
    peers: &PeerList,
    active_peer: &Arc<Mutex<Option<SocketAddr>>>,
    control_state: &ControlState,
    conn_mgr: &ConnectionManager,
) -> bool {
    let previous = current_active_peer(active_peer);
    if previous == Some(next) {
        return false;
    }

    set_active_peer_value(active_peer, Some(next));
    apply_transfer(previous, Some(next), peers, control_state, conn_mgr);
    true
}

pub fn release_control(
    peers: &PeerList,
    active_peer: &Arc<Mutex<Option<SocketAddr>>>,
    control_state: &ControlState,
    conn_mgr: &ConnectionManager,
) -> bool {
    let previous = current_active_peer(active_peer);
    if previous.is_none() {
        return false;
    }

    set_active_peer_value(active_peer, None);
    apply_transfer(previous, None, peers, control_state, conn_mgr);
    true
}

/// Maneja la desconexión abrupta o caída de enlace (TCP drop o timeout keep-alive) de un peer.
/// Si el peer caído era el que estábamos controlando de forma remota, libera inmediatamente
/// el control y restaura el control de teclado y ratón a la pantalla local.
pub fn handle_peer_disconnected(
    disconnected_addr: SocketAddr,
    active_peer: &Arc<Mutex<Option<SocketAddr>>>,
    control_state: &ControlState,
) -> bool {
    let current = current_active_peer(active_peer);
    if current == Some(disconnected_addr) {
        set_active_peer_value(active_peer, None);
        let mut state = control_state.lock().unwrap();
        state.mark_local_control();
        tracing::warn!(
            "⚠️ Enlace perdido con peer activo {}. Control devuelto a pantalla local.",
            disconnected_addr
        );
        true
    } else {
        false
    }
}


pub fn current_active_peer(active_peer: &Arc<Mutex<Option<SocketAddr>>>) -> Option<SocketAddr> {
    let current_active = active_peer.lock().unwrap();
    *current_active
}

pub async fn send_input_to_active_peer(
    conn_mgr: &ConnectionManager,
    active_peer: &Arc<Mutex<Option<SocketAddr>>>,
    event: &InputEvent,
) -> bool {
    let Some(target) = current_active_peer(active_peer) else {
        return false;
    };

    conn_mgr.send(target, Frame::Input(event.clone())).await;
    true
}

fn next_connected_peer(peers: &PeerList, current: Option<SocketAddr>) -> Option<SocketAddr> {
    let now = Instant::now();
    let candidates = {
        let list = peers.lock().unwrap();
        list.iter()
            .filter(|peer| peer.is_connected(now))
            .map(|peer| peer.addr)
            .collect::<Vec<_>>()
    };

    if candidates.is_empty() {
        return None;
    }

    match current {
        None => Some(candidates[0]),
        Some(current_addr) => match candidates.iter().position(|addr| *addr == current_addr) {
            Some(index) if index + 1 < candidates.len() => Some(candidates[index + 1]),
            _ => None,
        },
    }
}

fn set_active_peer_value(active_peer: &Arc<Mutex<Option<SocketAddr>>>, value: Option<SocketAddr>) {
    let mut current_active = active_peer.lock().unwrap();
    *current_active = value;
}

fn apply_transfer(
    previous: Option<SocketAddr>,
    next: Option<SocketAddr>,
    peers: &PeerList,
    control_state: &ControlState,
    conn_mgr: &ConnectionManager,
) {
    let source_name = control_state.lock().unwrap().local_name.clone();
    let conn_mgr = conn_mgr.clone();

    if let Some(previous_addr) = previous {
        let mgr = conn_mgr.clone();
        let msg = ControlMessage::Release {
            source_name: source_name.clone(),
        };
        let release_events = create_release_modifier_events();
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                for event in release_events {
                    mgr.send(previous_addr, Frame::Input(event)).await;
                }
                mgr.send(previous_addr, Frame::Control(msg)).await;
            });
        }
    }

    if let Some(next_addr) = next {
        let target_label = peer_label(peers, next_addr);
        {
            let mut state = control_state.lock().unwrap();
            state.mark_source_transfer(&target_label);
        }

        let msg = ControlMessage::Assign { source_name };
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                conn_mgr.send(next_addr, Frame::Control(msg)).await;
            });
        }
    } else {
        let mut state = control_state.lock().unwrap();
        state.mark_local_control();
    }
}

fn peer_label(peers: &PeerList, addr: SocketAddr) -> String {
    let list = peers.lock().unwrap();
    list.iter()
        .find(|peer| peer.addr.ip() == addr.ip())
        .map(|peer| peer.name.clone())
        .unwrap_or_else(|| addr.ip().to_string())
}

fn record_hits(hits: &mut Vec<Instant>) -> bool {
    let now = Instant::now();
    hits.push(now);
    hits.retain(|instant| now.duration_since(*instant) <= COMMAND_WINDOW);
    if hits.len() >= 3 {
        hits.clear();
        return true;
    }
    false
}

fn matches_mouse_command(command: &str, button: &MouseButton) -> bool {
    matches!(
        (command, button),
        ("triple_right_click", MouseButton::Right) | ("triple_middle_click", MouseButton::Middle)
    )
}

/// Dead corners: evita cambios involuntarios de pantalla cuando el ratón se acerca a las esquinas.
/// corner_size define el tamaño en píxeles del área muerta (por defecto 32.0 px).
pub fn is_in_dead_corner(x: f64, y: f64, screen_width: f64, screen_height: f64, corner_size: f64) -> bool {
    if corner_size <= 0.0 || screen_width <= 0.0 || screen_height <= 0.0 {
        return false;
    }
    let in_left = x < corner_size;
    let in_right = x >= (screen_width - corner_size);
    let in_top = y < corner_size;
    let in_bottom = y >= (screen_height - corner_size);

    (in_left && in_top) || (in_right && in_top) || (in_left && in_bottom) || (in_right && in_bottom)
}

/// Keycodes estándar para teclas modificadoras (Shift, Ctrl, Alt, Meta).
pub const MODIFIER_KEY_CODES: [u32; 4] = [
    0x38, // Shift (ej: Shift_L en macOS/X11/Win virtual key)
    0x3B, // Control
    0x3A, // Alt / Option
    0x37, // Meta / Command / Super
];

/// Genera eventos KeyPress(pressed: false) para soltar todas las teclas modificadoras
/// (Shift, Ctrl, Alt, Meta) al cambiar de pantalla o perder foco, evitando teclas pegadas.
pub fn create_release_modifier_events() -> Vec<InputEvent> {
    MODIFIER_KEY_CODES
        .iter()
        .map(|&code| InputEvent::KeyPress {
            key_code: code,
            pressed: false,
            modifiers: Default::default(),
            seq: 0,
        })
        .collect()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dead_corners() {
        let w = 1920.0;
        let h = 1080.0;
        let corner = 32.0;

        // Top-left corner
        assert!(is_in_dead_corner(5.0, 5.0, w, h, corner));
        assert!(is_in_dead_corner(0.0, 0.0, w, h, corner));
        assert!(is_in_dead_corner(31.9, 31.9, w, h, corner));

        // Top-right corner
        assert!(is_in_dead_corner(1900.0, 5.0, w, h, corner));
        assert!(is_in_dead_corner(1919.0, 0.0, w, h, corner));

        // Bottom-left corner
        assert!(is_in_dead_corner(5.0, 1070.0, w, h, corner));

        // Bottom-right corner
        assert!(is_in_dead_corner(1915.0, 1075.0, w, h, corner));

        // Edge middle (NOT a dead corner - switch should be allowed)
        assert!(!is_in_dead_corner(1919.0, 540.0, w, h, corner));
        assert!(!is_in_dead_corner(0.0, 540.0, w, h, corner));
        assert!(!is_in_dead_corner(960.0, 0.0, w, h, corner));
        assert!(!is_in_dead_corner(960.0, 1079.0, w, h, corner));

        // Screen center
        assert!(!is_in_dead_corner(960.0, 540.0, w, h, corner));
    }

    #[test]
    fn test_release_modifier_events() {
        let events = create_release_modifier_events();
        assert_eq!(events.len(), 4);
    }

    #[test]
    fn test_handle_peer_disconnected_restores_local_control() {
        let control_state = Arc::new(Mutex::new(ControlStatus::new("Mac".to_string())));
        let addr: SocketAddr = "192.168.1.50:4242".parse().unwrap();
        let active_peer = Arc::new(Mutex::new(Some(addr)));

        {
            let mut state = control_state.lock().unwrap();
            state.mark_source_transfer("Windows");
        }

        handle_peer_disconnected(addr, &active_peer, &control_state);

        assert_eq!(*active_peer.lock().unwrap(), None);
        let state = control_state.lock().unwrap();
        assert_eq!(state.controller_name, None);
        assert!(!state.assigned_here);
        assert_eq!(state.transfer_notice.as_ref().unwrap().role, "local");
    }
}


