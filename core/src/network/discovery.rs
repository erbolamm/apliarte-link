// Javier: Descubrimiento automático de peers en la red local.
// Analogía Flutter: como un Bonjour/mDNS pero implementado a mano con UDP broadcast.
//
// Cómo funciona:
//   1. Al arrancar, enviamos un broadcast "HELLO" a toda la red (255.255.255.255:4244)
//   2. Quien recibe el HELLO responde con "HELLO_REPLY" directamente a nuestra IP
//   3. Guardamos las IPs de los peers en una lista compartida
//   4. Cada 5 segundos repetimos el broadcast por si entra alguien nuevo
//
// Puerto 4244: solo para descubrimiento (no para datos de input)
// Puerto 4242: para los eventos de input reales (configurado en AppConfig)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;
use tokio::time::{interval, Duration};

pub const DISCOVERY_PORT: u16 = 4244;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum DiscoveryMsg {
    Hello { port: u16 },
    HelloReply { port: u16 },
}

/// Lista de peers descubiertos en la red local.
/// Javier: Arc<Mutex<...>> es como un ValueNotifier compartido entre threads en Flutter.
pub type PeerList = Arc<Mutex<Vec<SocketAddr>>>;

/// Arranca el servicio de descubrimiento.
/// Devuelve la lista de peers para que otros módulos la consulten.
pub async fn start(data_port: u16) -> Result<PeerList> {
    let peers: PeerList = Arc::new(Mutex::new(Vec::new()));
    let peers_clone = peers.clone();

    tokio::spawn(async move {
        if let Err(e) = run_discovery(data_port, peers_clone).await {
            tracing::error!("Error en descubrimiento: {}", e);
        }
    });

    Ok(peers)
}

async fn run_discovery(data_port: u16, peers: PeerList) -> Result<()> {
    // Abrimos socket en el puerto de discovery con SO_REUSEADDR para que
    // múltiples instancias en el mismo PC no colisionen
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT)).await?;
    socket.set_broadcast(true)?;

    tracing::info!("🔍 Descubrimiento de peers activo en puerto {}", DISCOVERY_PORT);

    // Enviamos el primer HELLO inmediatamente
    send_hello(&socket, data_port).await?;

    // Javier: Dos tareas en paralelo:
    //   - escuchar HELLOs entrantes y responder
    //   - broadcast periódico cada 5 segundos
    let socket = Arc::new(socket);
    let socket_recv = socket.clone();
    let peers_recv = peers.clone();

    // Tarea: escuchar mensajes entrantes
    let recv_task = tokio::spawn(async move {
        let mut buf = vec![0u8; 512];
        loop {
            match socket_recv.recv_from(&mut buf).await {
                Ok((len, from)) => {
                    handle_message(&socket_recv, &buf[..len], from, data_port, &peers_recv).await;
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
            if let Err(e) = send_hello(&socket, data_port).await {
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
    peers: &PeerList,
) {
    let msg: DiscoveryMsg = match serde_json::from_slice(data) {
        Ok(m) => m,
        Err(_) => return, // paquete no reconocido, ignorar
    };

    match msg {
        DiscoveryMsg::Hello { port } => {
            tracing::info!("👋 HELLO de {} (puerto datos: {})", from.ip(), port);

            // Guardar el peer con su puerto de datos real
            let peer_addr = SocketAddr::new(from.ip(), port);
            add_peer(peers, peer_addr);

            // Responder directamente a quien preguntó
            let reply = DiscoveryMsg::HelloReply { port: data_port };
            if let Ok(data) = serde_json::to_vec(&reply) {
                let reply_addr = SocketAddr::new(from.ip(), DISCOVERY_PORT);
                let _ = socket.send_to(&data, reply_addr).await;
            }
        }
        DiscoveryMsg::HelloReply { port } => {
            tracing::info!("✅ HELLO_REPLY de {} (puerto datos: {})", from.ip(), port);
            let peer_addr = SocketAddr::new(from.ip(), port);
            add_peer(peers, peer_addr);
        }
    }
}

async fn send_hello(socket: &UdpSocket, data_port: u16) -> Result<()> {
    let msg = DiscoveryMsg::Hello { port: data_port };
    let data = serde_json::to_vec(&msg)?;
    let broadcast_addr: SocketAddr = format!("255.255.255.255:{}", DISCOVERY_PORT).parse()?;
    socket.send_to(&data, broadcast_addr).await?;
    tracing::debug!("📢 HELLO broadcast enviado");
    Ok(())
}

fn add_peer(peers: &PeerList, addr: SocketAddr) {
    let mut list = peers.lock().unwrap();
    if !list.contains(&addr) {
        tracing::info!("🖥️  Peer nuevo descubierto: {}", addr);
        list.push(addr);
    }
}
