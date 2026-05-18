// Javier: Descubrimiento automático de peers en la red local.
// Analogía Flutter: como un Bonjour/mDNS pero implementado a mano con UDP broadcast.
//
// Cómo funciona:
//   1. Al arrancar, enviamos un broadcast "HELLO" al broadcast global y al de cada interfaz IPv4
//   2. Quien recibe el HELLO responde con "HELLO_REPLY" directamente a nuestra IP
//   3. Guardamos las IPs de los peers en una lista compartida
//   4. Cada 5 segundos repetimos el broadcast por si entra alguien nuevo
//
// Puerto 4244: solo para descubrimiento (no para datos de input)
// Puerto 4242: para los eventos de input reales (configurado en AppConfig)

use anyhow::Result;
use if_addrs::{get_if_addrs, IfAddr};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::time::{interval, Duration};

pub const DISCOVERY_PORT: u16 = 4244;
pub const PEER_TTL: Duration = Duration::from_secs(15);

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum DiscoveryMsg {
    Hello {
        port: u16,
        name: String,
        #[serde(default)]
        instance_id: String,
        #[serde(default = "default_transport")]
        transport: String,
        #[serde(default)]
        version: [u16; 2],
    },
    HelloReply {
        port: u16,
        name: String,
        #[serde(default)]
        instance_id: String,
        #[serde(default = "default_transport")]
        transport: String,
        #[serde(default)]
        version: [u16; 2],
    },
}

fn default_transport() -> String {
    "tcp".to_string()
}

/// Lista de peers descubiertos en la red local.
/// Javier: Arc<Mutex<...>> es como un ValueNotifier compartido entre threads en Flutter.
pub type PeerList = Arc<Mutex<Vec<PeerEntry>>>;

#[derive(Debug, Clone)]
pub struct PeerEntry {
    pub(crate) addr: SocketAddr,
    pub(crate) name: String,
    pub(crate) last_seen: Option<Instant>,
}

impl PeerEntry {
    pub(crate) fn discovered(addr: SocketAddr, name: String) -> Self {
        Self {
            addr,
            name,
            last_seen: Some(Instant::now()),
        }
    }

    pub(crate) fn manual(addr: SocketAddr, name: Option<String>) -> Self {
        Self {
            addr,
            name: name.unwrap_or_else(|| addr.ip().to_string()),
            last_seen: None,
        }
    }

    pub(crate) fn mark_seen(&mut self, name: String) {
        if !name.trim().is_empty() {
            self.name = name;
        }
        self.last_seen = Some(Instant::now());
    }

    pub(crate) fn is_connected(&self, now: Instant) -> bool {
        self.last_seen
            .map(|last_seen| now.duration_since(last_seen) <= PEER_TTL)
            .unwrap_or(false)
    }
}

/// Arranca el servicio de descubrimiento.
/// Devuelve la lista de peers para que otros módulos la consulten.
pub async fn start(data_port: u16, local_name: String) -> Result<PeerList> {
    let peers: PeerList = Arc::new(Mutex::new(Vec::new()));
    let peers_clone = peers.clone();
    let local_instance_id = build_instance_id();

    tokio::spawn(async move {
        if let Err(e) = run_discovery(data_port, local_name, local_instance_id, peers_clone).await {
            tracing::error!("Error en descubrimiento: {}", e);
        }
    });

    Ok(peers)
}

async fn run_discovery(
    data_port: u16,
    local_name: String,
    local_instance_id: String,
    peers: PeerList,
) -> Result<()> {
    // Abrimos socket en el puerto de discovery con SO_REUSEADDR para que
    // múltiples instancias en el mismo PC no colisionen
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT)).await?;
    socket.set_broadcast(true)?;

    tracing::info!(
        "🔍 Descubrimiento de peers activo en puerto {}",
        DISCOVERY_PORT
    );

    // Enviamos el primer HELLO inmediatamente
    send_hello(&socket, data_port, &local_name, &local_instance_id).await?;

    // Javier: Dos tareas en paralelo:
    //   - escuchar HELLOs entrantes y responder
    //   - broadcast periódico cada 5 segundos
    let socket = Arc::new(socket);
    let socket_recv = socket.clone();
    let peers_recv = peers.clone();
    let local_name_recv = local_name.clone();
    let local_instance_id_recv = local_instance_id.clone();

    // Tarea: escuchar mensajes entrantes
    let recv_task = tokio::spawn(async move {
        let mut buf = vec![0u8; 512];
        loop {
            match socket_recv.recv_from(&mut buf).await {
                Ok((len, from)) => {
                    handle_message(
                        &socket_recv,
                        &buf[..len],
                        from,
                        data_port,
                        &local_name_recv,
                        &local_instance_id_recv,
                        &peers_recv,
                    )
                    .await;
                }
                Err(e) => {
                    tracing::warn!("Error recibiendo discovery: {}", e);
                }
            }
        }
    });

    // Tarea: broadcast periódico cada 5 segundos
    let send_task = tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(5));
        ticker.tick().await; // primera tick es inmediata, la saltamos (ya hicimos el primer HELLO)
        loop {
            ticker.tick().await;
            if let Err(e) = send_hello(&socket, data_port, &local_name, &local_instance_id).await {
                tracing::warn!("Error enviando HELLO: {}", e);
            }
        }
    });

    tokio::select! {
        _ = recv_task => {}
        _ = send_task => {}
    }

    Ok(())
}

async fn handle_message(
    socket: &UdpSocket,
    data: &[u8],
    from: SocketAddr,
    data_port: u16,
    local_name: &str,
    local_instance_id: &str,
    peers: &PeerList,
) {
    let msg: DiscoveryMsg = match serde_json::from_slice(data) {
        Ok(m) => m,
        Err(_) => return, // paquete no reconocido, ignorar
    };

    match msg {
        DiscoveryMsg::Hello {
            port,
            name,
            instance_id,
            transport: _,
            version: _,
        } => {
            if should_ignore_self_discovery(&instance_id, local_instance_id) {
                tracing::debug!("Ignorando HELLO propio de {}", from.ip());
                return;
            }

            tracing::info!("👋 HELLO de {} (puerto datos: {})", from.ip(), port);

            // Guardar el peer con su puerto de datos real
            let peer_addr = SocketAddr::new(from.ip(), port);
            add_peer(peers, peer_addr, Some(name.clone()));

            // Responder directamente a quien preguntó
            let reply = DiscoveryMsg::HelloReply {
                port: data_port,
                name: local_name.to_string(),
                instance_id: local_instance_id.to_string(),
                transport: "tcp".to_string(),
                version: [1, 0],
            };
            if let Ok(data) = serde_json::to_vec(&reply) {
                let reply_addr = SocketAddr::new(from.ip(), DISCOVERY_PORT);
                let _ = socket.send_to(&data, reply_addr).await;
            }
        }
        DiscoveryMsg::HelloReply {
            port,
            name,
            instance_id,
            transport: _,
            version: _,
        } => {
            if should_ignore_self_discovery(&instance_id, local_instance_id) {
                tracing::debug!("Ignorando HELLO_REPLY propio de {}", from.ip());
                return;
            }

            tracing::info!("✅ HELLO_REPLY de {} (puerto datos: {})", from.ip(), port);
            let peer_addr = SocketAddr::new(from.ip(), port);
            add_peer(peers, peer_addr, Some(name));
        }
    }
}

async fn send_hello(
    socket: &UdpSocket,
    data_port: u16,
    local_name: &str,
    local_instance_id: &str,
) -> Result<()> {
    let msg = DiscoveryMsg::Hello {
        port: data_port,
        name: local_name.to_string(),
        instance_id: local_instance_id.to_string(),
        transport: "tcp".to_string(),
        version: [1, 0],
    };
    let data = serde_json::to_vec(&msg)?;
    let mut sent_any = false;

    for target in discovery_targets() {
        match socket.send_to(&data, target).await {
            Ok(_) => {
                sent_any = true;
                tracing::debug!("📢 HELLO enviado a {}", target);
            }
            Err(error) => {
                tracing::debug!("No se pudo enviar HELLO a {}: {}", target, error);
            }
        }
    }

    if !sent_any {
        anyhow::bail!("No se pudo enviar HELLO a ningún broadcast local");
    }

    Ok(())
}

fn discovery_targets() -> Vec<SocketAddr> {
    let mut targets = HashSet::from([SocketAddr::new(
        IpAddr::V4(Ipv4Addr::BROADCAST),
        DISCOVERY_PORT,
    )]);

    match get_if_addrs() {
        Ok(interfaces) => {
            for iface in interfaces {
                let IfAddr::V4(addr) = iface.addr else {
                    continue;
                };

                if addr.ip.is_loopback() {
                    continue;
                }

                let broadcast = ipv4_broadcast(addr.ip, addr.netmask);
                targets.insert(SocketAddr::new(IpAddr::V4(broadcast), DISCOVERY_PORT));
            }
        }
        Err(error) => {
            tracing::warn!(
                "No se pudieron enumerar interfaces de red para discovery: {}",
                error
            );
        }
    }

    targets.into_iter().collect()
}

fn ipv4_broadcast(ip: Ipv4Addr, netmask: Ipv4Addr) -> Ipv4Addr {
    Ipv4Addr::from(u32::from(ip) | !u32::from(netmask))
}

fn build_instance_id() -> String {
    let started_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}-{}", std::process::id(), started_at)
}

fn should_ignore_self_discovery(instance_id: &str, local_instance_id: &str) -> bool {
    !instance_id.is_empty() && instance_id == local_instance_id
}

fn add_peer(peers: &PeerList, addr: SocketAddr, name: Option<String>) {
    let mut list = peers.lock().unwrap();
    let peer_name = name.unwrap_or_else(|| addr.ip().to_string());
    if let Some(existing) = list.iter_mut().find(|peer| peer.addr == addr) {
        existing.mark_seen(peer_name);
    } else {
        tracing::info!("🖥️  Peer nuevo descubierto: {} ({})", addr, peer_name);
        list.push(PeerEntry::discovered(addr, peer_name));
    }
}

#[cfg(test)]
mod tests {
    use super::{ipv4_broadcast, should_ignore_self_discovery};
    use std::net::Ipv4Addr;

    #[test]
    fn ignores_messages_from_same_instance_id() {
        assert!(should_ignore_self_discovery("daemon-123", "daemon-123"));
        assert!(!should_ignore_self_discovery("daemon-123", "daemon-456"));
        assert!(!should_ignore_self_discovery("", "daemon-123"));
    }

    #[test]
    fn calculates_ipv4_broadcast_from_netmask() {
        assert_eq!(
            ipv4_broadcast(
                Ipv4Addr::new(192, 168, 1, 42),
                Ipv4Addr::new(255, 255, 255, 0)
            ),
            Ipv4Addr::new(192, 168, 1, 255)
        );
    }
}
