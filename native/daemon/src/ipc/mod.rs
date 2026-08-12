// Javier: Este módulo gestiona la comunicación entre el servicio Rust y la app Flutter.
// Analogía Flutter: es como el servidor de un backend local.
//                   Flutter se conecta a él como si fuera una API REST,
//                   pero usando sockets locales (IPC) en vez de HTTP.
//
// ¿Por qué sockets locales y no HTTP?
//   - El servicio y Flutter están en el mismo PC
//   - Los sockets locales son mucho más rápidos que HTTP
//   - No abren puertos de red (más seguro)
//
// Protocolo IPC:
//   - Socket Unix en macOS/Linux: /tmp/apliarte-link.sock
//   - Named Pipe en Windows: \\.\pipe\apliarte-link
//   - Formato mensajes: JSON (un objeto por línea = JSON Lines)
//
// Comandos que recibe de Flutter:
//   - {"cmd": "status"}               → responde con el estado actual
//   - {"cmd": "add_peer", "ip": "..."}  → añade un PC a la lista
//   - {"cmd": "remove_peer", "ip": "..."}→ elimina un PC
//   - {"cmd": "update_config", ...}    → actualiza la configuración
//   - {"cmd": "shutdown"}              → apaga el servicio
//   - {"cmd": "set_visible", "visible": true} → compartir/ocultar este equipo
//   - {"cmd": "set_peer_allowed", "ip": "...", "allowed": true} → registrar dispositivo

use crate::config::AppConfig;
use crate::control::{self, ControlState, TransferNotice};
use crate::devices::{self, PeripheralInfo};
use crate::network::ConnectionManager;
use crate::network::{PeerEntry, PeerList};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Instant;

// Javier: Mensajes que Flutter puede enviar al servicio (Fase 2)
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum UiCommand {
    Status,
    AddPeer { ip: String, name: Option<String> },
    RemovePeer { ip: String },
    SetActivePeer { ip: String },
    ReleaseControl,
    CycleActivePeer,
    UpdateInputConfig { input: InputConfigPayload },
    UpdateConfig { config: serde_json::Value },
    SetVisible { visible: bool },
    SetPeerAllowed { ip: String, allowed: bool },
    Shutdown,
}

// Javier: Respuestas que el servicio envía a Flutter (Fase 2)
#[allow(dead_code)]
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UiResponse {
    Status {
        running: bool,
        active_peer: Option<String>,
        peers: Vec<PeerInfo>,
        peripherals: Vec<PeripheralInfo>,
        input_config: InputConfigPayload,
        local_name: String,
        assigned_here: bool,
        controller_name: Option<String>,
        transfer_notice: Option<TransferNotice>,
        visible: bool,
        allowed_peers: Vec<String>,
        /// macOS: ¿tiene el proceso permiso de Accesibilidad? None en otros SO.
        mac_accessibility_granted: Option<bool>,
    },
    Ok,
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfigPayload {
    #[serde(default)]
    pub armed: bool,
    pub edge_switch: bool,
    pub edge_side: String,
    pub mouse_switch_command: String,
    pub keyboard_switch_command: String,
}

impl From<&crate::config::InputConfig> for InputConfigPayload {
    fn from(value: &crate::config::InputConfig) -> Self {
        Self {
            armed: value.armed,
            edge_switch: value.edge_switch,
            edge_side: value.edge_side.clone(),
            mouse_switch_command: value.mouse_switch_command.clone(),
            keyboard_switch_command: value.keyboard_switch_command.clone(),
        }
    }
}

// Javier: Información de un PC conectado — es como un modelo en Flutter (Fase 2)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub ip: String,
    pub name: String,
    pub connected: bool,
    /// true si este peer está en nuestra allowlist (dispositivo registrado)
    pub allowed: bool,
}

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

/// Arranca el servidor IPC por TCP local (Loopback).
/// Javier: Es el backend de nuestra UI. Se levanta en el puerto cerrado 4243 (solo accesible desde este Mac).
pub async fn serve(
    cfg: Arc<Mutex<AppConfig>>,
    peers: PeerList,
    active_peer: Arc<Mutex<Option<SocketAddr>>>,
    control_state: ControlState,
    conn_mgr: ConnectionManager,
) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4243").await?;
    tracing::info!("🔌 Servidor IPC escuchando en 127.0.0.1:4243");

    loop {
        // Javier: accept() bloquea hasta que Flutter (nuestro front) se conecte
        let (mut socket, peer_addr) = listener.accept().await?;
        tracing::info!("📱 UI Flutter conectada desde {:?}", peer_addr);

        // Cada conexión de UI se maneja en un hilo independiente
        // Clonar el Arc antes del spawn — barato, solo incrementa el contador de referencias
        let peers = peers.clone();
        let active_peer = active_peer.clone();
        let cfg = cfg.clone();
        let control_state = control_state.clone();
        let conn_mgr = conn_mgr.clone();
        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut buf_reader = BufReader::new(reader);
            let mut line = String::new();

            // 1. Al conectar, enviamos a la UI nuestro status actual con peers reales
            let status = build_status(&peers, &active_peer, &cfg, &control_state);

            if let Ok(json) = serde_json::to_string(&status) {
                let _ = writer.write_all(format!("{}\n", json).as_bytes()).await;
            }

            // 2. Loop para escuchar comandos de la UI
            loop {
                line.clear();
                match buf_reader.read_line(&mut line).await {
                    Ok(0) => {
                        // EOF - La app de Flutter se cerró
                        break;
                    }
                    Ok(_) => {
                        // Javier: Parsear el JSON que mandó Flutter
                        let response = match serde_json::from_str::<UiCommand>(&line) {
                            Ok(cmd) => {
                                tracing::info!("⚡ Comando recibido de la UI: {:?}", cmd);
                                handle_command(
                                    cmd,
                                    &peers,
                                    &active_peer,
                                    &cfg,
                                    &control_state,
                                    &conn_mgr,
                                )
                            }
                            Err(e) => UiResponse::Error {
                                message: format!("Comando IPC inválido: {}", e),
                            },
                        };

                        if let Ok(resp) = serde_json::to_string(&response) {
                            let _ = writer.write_all(format!("{}\n", resp).as_bytes()).await;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Error leyendo del socket IPC: {}", e);
                        break;
                    }
                }
            }
            tracing::info!("📱 UI Flutter desconectada");
        });
    }
}

fn handle_command(
    cmd: UiCommand,
    peers: &PeerList,
    active_peer: &Arc<Mutex<Option<SocketAddr>>>,
    cfg: &Arc<Mutex<AppConfig>>,
    control_state: &ControlState,
    conn_mgr: &ConnectionManager,
) -> UiResponse {
    let data_port = cfg.lock().unwrap().network.port;
    match cmd {
        UiCommand::Status => build_status(peers, active_peer, cfg, control_state),
        UiCommand::AddPeer { ip, name } => match resolve_peer_addr(&ip, data_port, peers) {
            Ok(addr) => {
                add_peer(peers, addr, name);
                build_status(peers, active_peer, cfg, control_state)
            }
            Err(message) => UiResponse::Error { message },
        },
        UiCommand::RemovePeer { ip } => {
            let removed_addr = remove_peer(peers, &ip);
            if let Some(addr) = removed_addr {
                let mut current_active = active_peer.lock().unwrap();
                if current_active.as_ref() == Some(&addr) {
                    *current_active = None;
                }
            }
            build_status(peers, active_peer, cfg, control_state)
        }
        UiCommand::SetActivePeer { ip } => match resolve_peer_addr(&ip, data_port, peers) {
            Ok(addr) => {
                add_peer(peers, addr, None);
                let allowed = cfg.lock().unwrap().sharing.allowed_peers.clone();
                control::set_active_peer(
                    addr,
                    peers,
                    active_peer,
                    control_state,
                    conn_mgr,
                    &allowed,
                );
                build_status(peers, active_peer, cfg, control_state)
            }
            Err(message) => UiResponse::Error { message },
        },
        UiCommand::ReleaseControl => {
            control::release_control(peers, active_peer, control_state, conn_mgr);
            build_status(peers, active_peer, cfg, control_state)
        }
        UiCommand::CycleActivePeer => {
            let allowed = cfg.lock().unwrap().sharing.allowed_peers.clone();
            control::cycle_active_peer(
                peers,
                active_peer,
                control_state,
                conn_mgr,
                &allowed,
            );
            build_status(peers, active_peer, cfg, control_state)
        }
        UiCommand::UpdateInputConfig { input } => {
            let updated_cfg = {
                let mut current_cfg = cfg.lock().unwrap();
                current_cfg.input.armed = input.armed;
                current_cfg.input.edge_switch = input.edge_switch;
                current_cfg.input.edge_side = input.edge_side;
                current_cfg.input.mouse_switch_command = input.mouse_switch_command;
                current_cfg.input.keyboard_switch_command = input.keyboard_switch_command;
                current_cfg.input.hotkey = Some(current_cfg.input.keyboard_switch_command.clone());
                current_cfg.clone()
            };

            if !updated_cfg.input.armed {
                control::release_control(peers, active_peer, control_state, conn_mgr);
            }

            match crate::config::save(&updated_cfg) {
                Ok(()) => build_status(peers, active_peer, cfg, control_state),
                Err(error) => UiResponse::Error {
                    message: format!("No se pudo guardar la configuración: {}", error),
                },
            }
        }
        UiCommand::UpdateConfig { config: _ } => UiResponse::Ok,
        UiCommand::SetVisible { visible } => {
            let updated_cfg = {
                let mut current_cfg = cfg.lock().unwrap();
                current_cfg.sharing.visible = visible;
                current_cfg.clone()
            };

            match crate::config::save(&updated_cfg) {
                Ok(()) => {
                    tracing::info!(
                        "👁️  Visibilidad del equipo cambiada: visible={}",
                        visible
                    );
                    build_status(peers, active_peer, cfg, control_state)
                }
                Err(error) => UiResponse::Error {
                    message: format!("No se pudo guardar la configuración: {}", error),
                },
            }
        }
        UiCommand::SetPeerAllowed { ip, allowed } => {
            let updated_cfg = {
                let mut current_cfg = cfg.lock().unwrap();
                if allowed {
                    if !current_cfg
                        .sharing
                        .allowed_peers
                        .iter()
                        .any(|entry| entry == &ip)
                    {
                        current_cfg.sharing.allowed_peers.push(ip.clone());
                    }
                } else {
                    current_cfg.sharing.allowed_peers.retain(|entry| entry != &ip);
                }
                current_cfg.clone()
            };

            match crate::config::save(&updated_cfg) {
                Ok(()) => {
                    tracing::info!(
                        "🔐 Registro de dispositivo {}: allowed={}",
                        ip,
                        allowed
                    );
                    build_status(peers, active_peer, cfg, control_state)
                }
                Err(error) => UiResponse::Error {
                    message: format!("No se pudo guardar la configuración: {}", error),
                },
            }
        }
        UiCommand::Shutdown => UiResponse::Ok,
    }
}

fn build_status(
    peers: &PeerList,
    active_peer: &Arc<Mutex<Option<SocketAddr>>>,
    cfg: &Arc<Mutex<AppConfig>>,
    control_state: &ControlState,
) -> UiResponse {
    let current_cfg = cfg.lock().unwrap();
    let peer_infos = build_peer_infos(peers, &current_cfg.sharing.allowed_peers);
    let peripherals = match devices::list_input_peripherals() {
        Ok(peripherals) => peripherals,
        Err(error) => {
            tracing::warn!("No se pudo construir el inventario HID local: {}", error);
            Vec::new()
        }
    };
    let current_active = active_peer.lock().unwrap();
    let control_snapshot = control_state.lock().unwrap().clone();
    #[cfg(target_os = "macos")]
    let mac_accessibility_granted = Some(crate::input::accessibility_granted());
    #[cfg(not(target_os = "macos"))]
    let mac_accessibility_granted = None;
    UiResponse::Status {
        running: true,
        active_peer: current_active.as_ref().map(|addr| addr.ip().to_string()),
        peers: peer_infos,
        peripherals,
        input_config: InputConfigPayload::from(&current_cfg.input),
        local_name: control_snapshot.local_name,
        assigned_here: control_snapshot.assigned_here,
        controller_name: control_snapshot.controller_name,
        transfer_notice: control_snapshot.transfer_notice,
        visible: current_cfg.sharing.visible,
        allowed_peers: current_cfg.sharing.allowed_peers.clone(),
        mac_accessibility_granted,
    }
}

fn build_peer_infos(peers: &PeerList, allowed_peers: &[String]) -> Vec<PeerInfo> {
    let list = peers.lock().unwrap();
    let now = Instant::now();
    list.iter()
        .map(|peer| {
            let ip = peer.addr.ip().to_string();
            PeerInfo {
                allowed: allowed_peers.is_empty() || allowed_peers.contains(&ip),
                ip,
                name: peer.name.clone(),
                connected: peer.is_connected(now),
            }
        })
        .collect()
}

fn resolve_peer_addr(
    ip: &str,
    data_port: u16,
    peers: &PeerList,
) -> std::result::Result<SocketAddr, String> {
    if let Ok(addr) = ip.parse::<SocketAddr>() {
        return Ok(addr);
    }

    let parsed_ip = ip
        .parse::<IpAddr>()
        .map_err(|_| format!("IP de peer inválida: {}", ip))?;

    let list = peers.lock().unwrap();
    if let Some(existing) = list.iter().find(|peer| peer.addr.ip() == parsed_ip) {
        return Ok(existing.addr);
    }

    Ok(SocketAddr::new(parsed_ip, data_port))
}

fn add_peer(peers: &PeerList, addr: SocketAddr, name: Option<String>) {
    let mut list = peers.lock().unwrap();
    if !list.iter().any(|peer| peer.addr == addr) {
        tracing::info!("🖥️  Peer añadido manualmente desde UI: {}", addr);
        list.push(PeerEntry::manual(addr, name));
    }
}

fn remove_peer(peers: &PeerList, ip: &str) -> Option<SocketAddr> {
    let mut list = peers.lock().unwrap();
    let index = list
        .iter()
        .position(|peer| peer.addr.ip().to_string() == ip || peer.addr.to_string() == ip)?;
    Some(list.remove(index).addr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::discovery::PEER_TTL;
    use std::time::Duration;

    #[test]
    fn build_peer_infos_only_marks_recent_peers_connected() {
        let recent = SocketAddr::from(([192, 168, 1, 10], 4242));
        let stale = SocketAddr::from(([192, 168, 1, 11], 4242));
        let manual = SocketAddr::from(([192, 168, 1, 12], 4242));

        let peers: PeerList = Arc::new(Mutex::new(vec![
            PeerEntry {
                addr: recent,
                name: "Mac Studio".to_string(),
                last_seen: Some(Instant::now()),
            },
            PeerEntry {
                addr: stale,
                name: "Portátil".to_string(),
                last_seen: Some(Instant::now() - PEER_TTL - Duration::from_secs(1)),
            },
            PeerEntry::manual(manual, Some("Windows".to_string())),
        ]));

        let infos = build_peer_infos(&peers, &[]);

        assert!(infos
            .iter()
            .any(|peer| peer.ip == recent.ip().to_string() && peer.connected));
        assert!(infos
            .iter()
            .any(|peer| peer.ip == stale.ip().to_string() && !peer.connected));
        assert!(infos
            .iter()
            .any(|peer| peer.ip == manual.ip().to_string() && !peer.connected));
    }
}
