use crate::config::InputConfig;
use crate::input::protocol::{ControlMessage, InputEvent, MouseButton, NetworkPacket};
use crate::network::PeerList;
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, UdpSocket as StdUdpSocket};
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
            format!("Este ordenador está recibiendo control desde {}.", source_label),
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
    data_port: u16,
) -> bool {
    let previous = current_active_peer(active_peer);
    let next = next_connected_peer(peers, previous);

    if previous == next {
        return false;
    }

    set_active_peer_value(active_peer, next);
    apply_transfer(previous, next, peers, control_state, data_port);
    true
}

pub fn set_active_peer(
    next: SocketAddr,
    peers: &PeerList,
    active_peer: &Arc<Mutex<Option<SocketAddr>>>,
    control_state: &ControlState,
    data_port: u16,
) -> bool {
    let previous = current_active_peer(active_peer);
    if previous == Some(next) {
        return false;
    }

    set_active_peer_value(active_peer, Some(next));
    apply_transfer(previous, Some(next), peers, control_state, data_port);
    true
}

pub fn release_control(
    peers: &PeerList,
    active_peer: &Arc<Mutex<Option<SocketAddr>>>,
    control_state: &ControlState,
    data_port: u16,
) -> bool {
    let previous = current_active_peer(active_peer);
    if previous.is_none() {
        return false;
    }

    set_active_peer_value(active_peer, None);
    apply_transfer(previous, None, peers, control_state, data_port);
    true
}

pub fn current_active_peer(active_peer: &Arc<Mutex<Option<SocketAddr>>>) -> Option<SocketAddr> {
    let current_active = active_peer.lock().unwrap();
    *current_active
}

pub fn send_input_to_active_peer(
    socket: &StdUdpSocket,
    active_peer: &Arc<Mutex<Option<SocketAddr>>>,
    event: &InputEvent,
) -> bool {
    let Some(target) = current_active_peer(active_peer) else {
        return false;
    };

    let packet = NetworkPacket::Input {
        event: event.clone(),
    };
    let data = match serde_json::to_vec(&packet) {
        Ok(data) => data,
        Err(error) => {
            tracing::warn!("No se pudo serializar el evento local: {}", error);
            return false;
        }
    };

    match socket.send_to(&data, target) {
        Ok(_) => true,
        Err(error) => {
            tracing::warn!("No se pudo reenviar el input local a {}: {}", target, error);
            false
        }
    }
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

fn set_active_peer_value(
    active_peer: &Arc<Mutex<Option<SocketAddr>>>,
    value: Option<SocketAddr>,
) {
    let mut current_active = active_peer.lock().unwrap();
    *current_active = value;
}

fn apply_transfer(
    previous: Option<SocketAddr>,
    next: Option<SocketAddr>,
    peers: &PeerList,
    control_state: &ControlState,
    data_port: u16,
) {
    let source_name = control_state.lock().unwrap().local_name.clone();

    if let Some(previous_addr) = previous {
        let _ = send_control_message(
            previous_addr,
            &ControlMessage::Release {
                source_name: source_name.clone(),
            },
        );
    }

    if let Some(next_addr) = next {
        let target_label = peer_label(peers, next_addr);
        {
            let mut state = control_state.lock().unwrap();
            state.mark_source_transfer(&target_label);
        }

        let target_addr = SocketAddr::new(next_addr.ip(), data_port);
        let _ = send_control_message(
            target_addr,
            &ControlMessage::Assign {
                source_name: source_name,
            },
        );
    } else {
        let mut state = control_state.lock().unwrap();
        state.mark_local_control();
    }
}

fn send_control_message(target: SocketAddr, message: &ControlMessage) -> std::io::Result<()> {
    let socket = StdUdpSocket::bind("0.0.0.0:0")?;
    let packet = NetworkPacket::Control {
        message: message.clone(),
    };
    let data = serde_json::to_vec(&packet)?;
    socket.send_to(&data, target)?;
    Ok(())
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
        ("triple_right_click", MouseButton::Right)
            | ("triple_middle_click", MouseButton::Middle)
    )
}