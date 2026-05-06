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
//   - Socket Unix en macOS/Linux: /tmp/apliarte-switch.sock
//   - Named Pipe en Windows: \\.\pipe\apliarte-switch
//   - Formato mensajes: JSON (un objeto por línea = JSON Lines)
//
// Comandos que recibe de Flutter:
//   - {"cmd": "status"}               → responde con el estado actual
//   - {"cmd": "add_peer", "ip": "..."}  → añade un PC a la lista
//   - {"cmd": "remove_peer", "ip": "..."}→ elimina un PC
//   - {"cmd": "update_config", ...}    → actualiza la configuración
//   - {"cmd": "shutdown"}              → apaga el servicio

use crate::config::AppConfig;
use crate::input::protocol::InputEvent;
use crate::network::PeerList;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;

// Javier: Mensajes que Flutter puede enviar al servicio (Fase 2)
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum UiCommand {
    Status,
    AddPeer { ip: String, name: Option<String> },
    RemovePeer { ip: String },
    SetActivePeer { ip: String },
    UpdateConfig { config: serde_json::Value },
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
    },
    Ok,
    Error {
        message: String,
    },
}

// Javier: Información de un PC conectado — es como un modelo en Flutter (Fase 2)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub ip: String,
    pub name: String,
    pub connected: bool,
}

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

/// Arranca el servidor IPC por TCP local (Loopback).
/// Javier: Es el backend de nuestra UI. Se levanta en el puerto cerrado 4243 (solo accesible desde este Mac).
pub async fn serve(
    _event_tx: Sender<InputEvent>,
    cfg: AppConfig,
    peers: PeerList,
    active_peer: Arc<Mutex<Option<SocketAddr>>>,
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
        let data_port = cfg.network.port;
        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut buf_reader = BufReader::new(reader);
            let mut line = String::new();

            // 1. Al conectar, enviamos a la UI nuestro status actual con peers reales
            let status = {
                let peer_infos = build_peer_infos(&peers);
                let current_active = active_peer.lock().unwrap();
                let active_ip = current_active.as_ref().map(|addr| addr.ip().to_string());
                drop(current_active);
                UiResponse::Status {
                    running: true,
                    active_peer: active_ip,
                    peers: peer_infos,
                }
            };

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
                                handle_command(cmd, &peers, &active_peer, data_port)
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
    data_port: u16,
) -> UiResponse {
    match cmd {
        UiCommand::Status => build_status(peers, active_peer),
        UiCommand::AddPeer { ip, name: _ } => match resolve_peer_addr(&ip, data_port, peers) {
            Ok(addr) => {
                add_peer(peers, addr);
                UiResponse::Ok
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
            UiResponse::Ok
        }
        UiCommand::SetActivePeer { ip } => match resolve_peer_addr(&ip, data_port, peers) {
            Ok(addr) => {
                add_peer(peers, addr);
                let mut current_active = active_peer.lock().unwrap();
                *current_active = Some(addr);
                UiResponse::Status {
                    running: true,
                    active_peer: Some(addr.ip().to_string()),
                    peers: build_peer_infos(peers),
                }
            }
            Err(message) => UiResponse::Error { message },
        },
        UiCommand::UpdateConfig { config: _ } => UiResponse::Ok,
        UiCommand::Shutdown => UiResponse::Ok,
    }
}

fn build_status(peers: &PeerList, active_peer: &Arc<Mutex<Option<SocketAddr>>>) -> UiResponse {
    let peer_infos = build_peer_infos(peers);
    let current_active = active_peer.lock().unwrap();
    UiResponse::Status {
        running: true,
        active_peer: current_active.as_ref().map(|addr| addr.ip().to_string()),
        peers: peer_infos,
    }
}

fn build_peer_infos(peers: &PeerList) -> Vec<PeerInfo> {
    let list = peers.lock().unwrap();
    list.iter()
        .map(|addr| PeerInfo {
            ip: addr.ip().to_string(),
            name: addr.to_string(),
            connected: true,
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
    if let Some(existing) = list.iter().find(|addr| addr.ip() == parsed_ip) {
        return Ok(*existing);
    }

    Ok(SocketAddr::new(parsed_ip, data_port))
}

fn add_peer(peers: &PeerList, addr: SocketAddr) {
    let mut list = peers.lock().unwrap();
    if !list.contains(&addr) {
        tracing::info!("🖥️  Peer añadido manualmente desde UI: {}", addr);
        list.push(addr);
    }
}

fn remove_peer(peers: &PeerList, ip: &str) -> Option<SocketAddr> {
    let mut list = peers.lock().unwrap();
    let index = list
        .iter()
        .position(|addr| addr.ip().to_string() == ip || addr.to_string() == ip)?;
    Some(list.remove(index))
}
