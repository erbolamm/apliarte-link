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
    allowed: &[String],
) -> bool {
    let previous = current_active_peer(active_peer);
    let next = next_connected_peer(peers, previous, allowed);

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
    allowed: &[String],
) -> bool {
    // Javier: "Dispositivos registrados" — si la allowlist no está vacía
    // y este peer no está en ella, no podemos tomar su control.
    if !is_addr_allowed(&next, allowed) {
        tracing::warn!(
            "🚫 No se puede conectar a {} — no está en la allowlist",
            next
        );
        return false;
    }

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

fn next_connected_peer(
    peers: &PeerList,
    current: Option<SocketAddr>,
    allowed: &[String],
) -> Option<SocketAddr> {
    let now = Instant::now();
    let candidates = {
        let list = peers.lock().unwrap();
        list.iter()
            .filter(|peer| peer.is_connected(now))
            .filter(|peer| is_addr_allowed(&peer.addr, allowed))
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

/// Javier: ¿Está esta IP en la lista de permitidos?
/// Allowlist vacía = todos permitidos (modo actual).
pub fn is_addr_allowed(addr: &SocketAddr, allowed: &[String]) -> bool {
    if allowed.is_empty() {
        return true;
    }
    allowed.iter().any(|ip| ip == &addr.ip().to_string())
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
        tokio::spawn(async move {
            mgr.send(previous_addr, Frame::Control(msg)).await;
        });
    }

    if let Some(next_addr) = next {
        let target_label = peer_label(peers, next_addr);
        {
            let mut state = control_state.lock().unwrap();
            state.mark_source_transfer(&target_label);
        }

        let msg = ControlMessage::Assign { source_name };
        tokio::spawn(async move {
            // Javier: Pieza que faltaba — antes de enviar el Assign hay que
            // ESTABLECER la conexión saliente (connect es idempotente: si ya
            // está en el pool, no hace nada). Sin esto, send() descartaba
            // el frame silenciosamente y el control nunca llegaba al peer.
            if let Err(error) = conn_mgr.connect(next_addr).await {
                tracing::warn!(
                    "⚠️ No se pudo conectar con {} para ceder el control: {}",
                    next_addr,
                    error
                );
                return;
            }
            conn_mgr.send(next_addr, Frame::Control(msg)).await;
        });
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
