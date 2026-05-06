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
use tokio::sync::mpsc::Sender;

// Javier: Mensajes que Flutter puede enviar al servicio (Fase 2)
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum UiCommand {
    Status,
    AddPeer { ip: String, name: Option<String> },
    RemovePeer { ip: String },
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
    Error { message: String },
}

// Javier: Información de un PC conectado — es como un modelo en Flutter (Fase 2)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub ip: String,
    pub name: String,
    pub connected: bool,
}

use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Arranca el servidor IPC por TCP local (Loopback).
/// Javier: Es el backend de nuestra UI. Se levanta en el puerto cerrado 4243 (solo accesible desde este Mac).
pub async fn serve(_event_tx: Sender<InputEvent>, _cfg: AppConfig, peers: PeerList) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4243").await?;
    tracing::info!("🔌 Servidor IPC escuchando en 127.0.0.1:4243");

    loop {
        // Javier: accept() bloquea hasta que Flutter (nuestro front) se conecte
        let (mut socket, peer_addr) = listener.accept().await?;
        tracing::info!("📱 UI Flutter conectada desde {:?}", peer_addr);

        // Cada conexión de UI se maneja en un hilo independiente
        // Clonar el Arc antes del spawn — barato, solo incrementa el contador de referencias
        let peers = peers.clone();
        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut buf_reader = BufReader::new(reader);
            let mut line = String::new();

            // 1. Al conectar, enviamos a la UI nuestro status actual con peers reales
            let status = {
                let list = peers.lock().unwrap();
                let peer_infos = list.iter().map(|addr| PeerInfo {
                    ip: addr.ip().to_string(),
                    name: addr.to_string(),
                    connected: true,
                }).collect::<Vec<_>>();
                drop(list);
                UiResponse::Status {
                    running: true,
                    active_peer: peer_infos.first().map(|p| p.ip.clone()),
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
                        if let Ok(cmd) = serde_json::from_str::<UiCommand>(&line) {
                            tracing::info!("⚡ Comando recibido de la UI: {:?}", cmd);
                            
                            // Si manda status, le devolvemos el status con peers reales del discovery
                            if let UiCommand::Status = cmd {
                                let peer_infos = {
                                    let list = peers.lock().unwrap();
                                    list.iter().map(|addr| PeerInfo {
                                        ip: addr.ip().to_string(),
                                        name: addr.to_string(),
                                        connected: true,
                                    }).collect::<Vec<_>>()
                                };
                                let st = UiResponse::Status {
                                    running: true,
                                    active_peer: peer_infos.first().map(|p| p.ip.clone()),
                                    peers: peer_infos,
                                };
                                if let Ok(resp) = serde_json::to_string(&st) {
                                    let _ = writer.write_all(format!("{}\n", resp).as_bytes()).await;
                                }
                            } else {
                                // Para otros comandos, solo confirmamos "Ok" de momento
                                let ok = UiResponse::Ok;
                                if let Ok(resp) = serde_json::to_string(&ok) {
                                    let _ = writer.write_all(format!("{}\n", resp).as_bytes()).await;
                                }
                            }
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
