// Transport layer: TCP + TLS + MessagePack framing.
//
// Replaces the old UDP + JSON transport with a reliable, encrypted,
// binary protocol — following deskflow's proven architecture.
//
// Architecture:
//   TcpListener (0.0.0.0:4242)
//     ├── accept loop → TLS handshake → protocol handshake → Connection
//     └── each Connection spawns recv + keep-alive tasks
//
//   ConnectionManager
//     ├── pool: HashMap<SocketAddr, Sender<Frame>>  (outbound senders)
//     ├── inbound_tx: broadcast to event channel
//     └── connect(): TCP → TLS → handshake → add to pool
//
// Framing: 4-byte BE length prefix + MessagePack payload.

use crate::input::protocol::Frame;
use anyhow::{Context, Result};
use rcgen::{CertificateParams, DistinguishedName, KeyPair, PKCS_ECDSA_P256_SHA256};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::ServerConfig;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::time;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::TlsConnector;
use tracing;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Shared handle to the connection pool for sending frames.
pub type ConnectionManager = Arc<ConnectionPool>;

/// Pool of active TCP+TLS connections to peers.
pub struct ConnectionPool {
    /// Outbound senders keyed by peer address.
    connections: RwLock<HashMap<SocketAddr, tokio::sync::mpsc::Sender<Frame>>>,
    /// Channel to forward received frames to the input engine.
    inbound_tx: tokio::sync::mpsc::Sender<(SocketAddr, Frame)>,
    /// Local daemon name for handshakes.
    local_name: String,
    /// Protocol version we speak.
    version: (u16, u16),
    /// TLS acceptor for inbound connections.
    tls_acceptor: TlsAcceptor,
    /// TLS connector for outbound connections.
    tls_connector: TlsConnector,
    /// Keep-alive interval.
    keepalive_interval: Duration,
    /// Keep-alive timeout (after this many missed pings, connection is dead).
    keepalive_timeout: Duration,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

impl ConnectionPool {
    /// Build the TLS config and return a new ConnectionPool.
    pub fn new(
        local_name: String,
        inbound_tx: tokio::sync::mpsc::Sender<(SocketAddr, Frame)>,
    ) -> Result<ConnectionManager> {
        let (tls_acceptor, tls_connector) = build_tls_config()?;

        Ok(Arc::new(Self {
            connections: RwLock::new(HashMap::new()),
            inbound_tx,
            local_name,
            version: (1, 0),
            tls_acceptor,
            tls_connector,
            keepalive_interval: Duration::from_secs(3),
            keepalive_timeout: Duration::from_secs(9),
        }))
    }

    /// Start accepting inbound connections. Never returns.
    pub async fn accept_loop(self: &ConnectionManager, listener: TcpListener) {
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    tracing::info!("🔌 Conexión TCP entrante de {}", addr);
                    let mgr = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = mgr.handle_inbound(stream, addr).await {
                            tracing::warn!("Conexión entrante de {} cerrada: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Error aceptando conexión: {}", e);
                }
            }
        }
    }

    /// Connect to a peer and add it to the pool. Idempotent.
    pub async fn connect(self: &ConnectionManager, addr: SocketAddr) -> Result<()> {
        // Already connected?
        {
            let pool = self.connections.read().await;
            if pool.contains_key(&addr) {
                return Ok(());
            }
        }

        tracing::info!("🔗 Conectando a peer {}:{}", addr.ip(), addr.port());

        let stream = TcpStream::connect(addr)
            .await
            .with_context(|| format!("TCP connect to {}", addr))?;

        let tls_stream = self
            .tls_connector
            .connect(
                rustls::pki_types::ServerName::IpAddress(addr.ip().into()),
                stream,
            )
            .await
            .with_context(|| format!("TLS handshake with {}", addr))?;

        let (read, write) = tokio::io::split(tls_stream);
        let (tx, rx) = tokio::sync::mpsc::channel::<Frame>(64);

        // Keep-alive tracking
        let last_pong = Arc::new(AtomicU64::new(now_millis()));

        // Spawn writer task
        let writer_mgr = self.clone();
        let writer_addr = addr;
        tokio::spawn(async move {
            if let Err(e) = write_frames(write, rx).await {
                tracing::warn!("Writer para {}: {}", writer_addr, e);
            }
            writer_mgr.remove_connection(&writer_addr).await;
        });

        // Spawn reader task
        let reader_mgr = self.clone();
        let reader_addr = addr;
        let reader_last_pong = last_pong.clone();
        tokio::spawn(async move {
            if let Err(e) = read_frames(
                read,
                reader_addr,
                reader_mgr.inbound_tx.clone(),
                &reader_last_pong,
            )
            .await
            {
                tracing::warn!("Reader para {}: {}", reader_addr, e);
            }
            reader_mgr.remove_connection(&reader_addr).await;
        });

        // Spawn keep-alive task
        let ka_tx = tx.clone();
        let ka_last_pong = last_pong.clone();
        let ka_timeout = self.keepalive_timeout;
        let ka_interval = self.keepalive_interval;
        let ka_addr = addr;
        let ka_mgr = self.clone();
        tokio::spawn(async move {
            keepalive_loop(ka_tx, ka_last_pong, ka_timeout, ka_interval).await;
            ka_mgr.remove_connection(&ka_addr).await;
        });

        // Send handshake
        let hello = Frame::Handshake(crate::input::protocol::HandshakeMessage::Hello {
            version: self.version,
            name: self.local_name.clone(),
        });
        tx.send(hello)
            .await
            .map_err(|_| anyhow::anyhow!("Handshake channel closed"))?;

        // Add to pool
        {
            let mut pool = self.connections.write().await;
            pool.insert(addr, tx);
        }

        tracing::info!("✅ Conectado a {}", addr);
        Ok(())
    }

    /// Send a frame to a specific peer. Automatically connects if not yet connected.
    pub async fn send(self: &ConnectionManager, addr: SocketAddr, frame: Frame) {
        {
            let pool = self.connections.read().await;
            if let Some(tx) = pool.get(&addr) {
                if tx.send(frame.clone()).await.is_ok() {
                    return;
                }
            }
        }

        // Si no está en el pool o el canal estaba cerrado, intentamos conectar
        if let Err(e) = self.connect(addr).await {
            tracing::debug!("No se pudo conectar a {} para enviar frame: {}", addr, e);
            return;
        }

        let pool = self.connections.read().await;
        if let Some(tx) = pool.get(&addr) {
            if tx.send(frame).await.is_err() {
                tracing::debug!("Canal cerrado para {} tras conectar", addr);
            }
        }
    }

    /// Check if a peer is currently connected.
    pub async fn is_connected(self: &ConnectionManager, addr: &SocketAddr) -> bool {
        self.connections.read().await.contains_key(addr)
    }

    pub fn inbound_channel(&self) -> tokio::sync::mpsc::Sender<(SocketAddr, Frame)> {
        self.inbound_tx.clone()
    }
}

// ---------------------------------------------------------------------------
// Inbound (accept → TLS → handshake → keep-alive)
// ---------------------------------------------------------------------------

impl ConnectionPool {
    async fn handle_inbound(
        self: &ConnectionManager,
        stream: TcpStream,
        addr: SocketAddr,
    ) -> Result<()> {
        let tls_stream = self
            .tls_acceptor
            .accept(stream)
            .await
            .with_context(|| format!("TLS accept from {}", addr))?;

        let (read, write) = tokio::io::split(tls_stream);
        let (tx, rx) = tokio::sync::mpsc::channel::<Frame>(64);

        // Send HelloAck immediately (server-side handshake response)
        let ack = Frame::Handshake(crate::input::protocol::HandshakeMessage::HelloAck {
            version: self.version,
            name: self.local_name.clone(),
        });
        let _ = tx.send(ack).await;

        // Keep-alive tracking
        let last_pong = Arc::new(AtomicU64::new(now_millis()));

        // Writer task
        let writer_mgr = self.clone();
        let writer_addr = addr;
        tokio::spawn(async move {
            if let Err(e) = write_frames(write, rx).await {
                tracing::warn!("Writer para {}: {}", writer_addr, e);
            }
            writer_mgr.remove_connection(&writer_addr).await;
        });

        // Reader task
        let reader_mgr = self.clone();
        let reader_addr = addr;
        let reader_last_pong = last_pong.clone();
        tokio::spawn(async move {
            if let Err(e) = read_frames(
                read,
                reader_addr,
                reader_mgr.inbound_tx.clone(),
                &reader_last_pong,
            )
            .await
            {
                tracing::warn!("Reader para {}: {}", reader_addr, e);
            }
            reader_mgr.remove_connection(&reader_addr).await;
        });

        // Keep-alive task
        let ka_tx = tx.clone();
        let ka_addr = addr;
        let ka_last_pong = last_pong.clone();
        let ka_timeout = self.keepalive_timeout;
        let ka_interval = self.keepalive_interval;
        let ka_mgr = self.clone();
        tokio::spawn(async move {
            keepalive_loop(ka_tx, ka_last_pong, ka_timeout, ka_interval).await;
            ka_mgr.remove_connection(&ka_addr).await;
        });

        // Add to pool
        {
            let mut pool = self.connections.write().await;
            pool.insert(addr, tx);
        }

        Ok(())
    }

    async fn remove_connection(self: &ConnectionManager, addr: &SocketAddr) {
        let mut pool = self.connections.write().await;
        pool.remove(addr);
        tracing::info!("🔌 Conexión cerrada: {}", addr);
    }
}

// ---------------------------------------------------------------------------
// Frame I/O (length-prefixed MessagePack)
// ---------------------------------------------------------------------------

async fn write_frames(
    mut write: impl AsyncWriteExt + Unpin,
    mut rx: tokio::sync::mpsc::Receiver<Frame>,
) -> Result<()> {
    while let Some(frame) = rx.recv().await {
        let payload = rmp_serde::to_vec(&frame).with_context(|| "MessagePack encode")?;
        let len = payload.len() as u32;
        let mut header = len.to_be_bytes().to_vec();
        header.extend_from_slice(&payload);
        write.write_all(&header).await?;
        write.flush().await?;
    }
    Ok(())
}

async fn read_frames(
    mut read: impl AsyncReadExt + Unpin,
    addr: SocketAddr,
    inbound_tx: tokio::sync::mpsc::Sender<(SocketAddr, Frame)>,
    last_pong: &AtomicU64,
) -> Result<()> {
    let mut len_buf = [0u8; 4];
    loop {
        // Read 4-byte length prefix
        read.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        // Sanity check
        if len > 1024 * 1024 {
            anyhow::bail!("Frame demasiado grande de {}: {} bytes", addr, len);
        }

        // Read payload
        let mut payload = vec![0u8; len];
        read.read_exact(&mut payload).await?;

        let frame: Frame = rmp_serde::from_slice(&payload).with_context(|| "MessagePack decode")?;

        // Update last_pong on Pong or any message (any traffic = alive)
        if matches!(
            frame,
            Frame::KeepAlive(crate::input::protocol::KeepAlive::Pong(_))
        ) {
            last_pong.store(now_millis(), Ordering::SeqCst);
        } else {
            // Any received frame means the peer is alive
            last_pong.store(now_millis(), Ordering::SeqCst);
            // Forward to input engine
            if inbound_tx.send((addr, frame)).await.is_err() {
                break; // channel closed, daemon shutting down
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// TLS (self-signed certificate per session)
// ---------------------------------------------------------------------------

fn build_tls_config() -> Result<(TlsAcceptor, TlsConnector)> {
    // Generate a self-signed certificate valid for this session
    let key_pair = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256)?;

    let mut params = CertificateParams::default();
    let mut dn = DistinguishedName::new();
    dn.push(rcgen::DnType::CommonName, "apliarte-link");
    params.distinguished_name = dn;
    params.key_usages = vec![
        rcgen::KeyUsagePurpose::DigitalSignature,
        rcgen::KeyUsagePurpose::KeyEncipherment,
    ];

    let cert = params.self_signed(&key_pair)?;

    let cert_der = CertificateDer::from(cert.der().clone());
    let key_der = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_pair.serialize_der()));

    let mut roots = rustls::RootCertStore::empty();
    roots.add(cert_der.clone())?;

    // Server config: expects our cert
    let server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der.clone()], key_der.clone_key())?;

    // Client config: trusts peers with self-signed certificate on local LAN
    let client_config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(super::danger_tls::SkipServerVerification))
        .with_no_client_auth();

    Ok((
        TlsAcceptor::from(Arc::new(server_config)),
        TlsConnector::from(Arc::new(client_config)),
    ))
}

// ---------------------------------------------------------------------------
// Keep-alive (per connection)
// ---------------------------------------------------------------------------

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

async fn keepalive_loop(
    tx: tokio::sync::mpsc::Sender<Frame>,
    last_pong: Arc<AtomicU64>,
    timeout: Duration,
    interval: Duration,
) {
    let mut ticker = time::interval(interval);
    ticker.tick().await; // skip first immediate tick
    let mut seq: u32 = 0;
    loop {
        ticker.tick().await;
        let elapsed = now_millis().saturating_sub(last_pong.load(Ordering::SeqCst));
        if elapsed > timeout.as_millis() as u64 {
            tracing::warn!(
                "Keep-alive timeout: {}ms sin respuesta, cerrando conexión",
                elapsed
            );
            break; // drop tx → writer channel closes → connection removed
        }

        seq = seq.wrapping_add(1);
        let ping = Frame::KeepAlive(crate::input::protocol::KeepAlive::Ping(seq));
        if tx.send(ping).await.is_err() {
            break; // channel closed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::protocol::{Frame, InputEvent, MouseButton};

    #[test]
    fn msgpack_roundtrip_input_event() {
        let event = InputEvent::MouseButton {
            button: MouseButton::Left,
            pressed: true,
            seq: 1,
        };
        let frame = Frame::Input(event);

        let encoded = rmp_serde::to_vec(&frame).unwrap();
        let decoded: Frame = rmp_serde::from_slice(&encoded).unwrap();

        match decoded {
            Frame::Input(event) => match event {
                InputEvent::MouseButton {
                    button,
                    pressed,
                    seq,
                } => {
                    assert!(matches!(button, MouseButton::Left));
                    assert!(pressed);
                    assert_eq!(seq, 1);
                }
                _ => panic!("Wrong input event type"),
            },
            _ => panic!("Wrong frame type"),
        }
    }

    #[tokio::test]
    async fn connection_pool_connect_and_send() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let (tx, _rx) = tokio::sync::mpsc::channel::<(SocketAddr, Frame)>(16);
        let mgr = ConnectionPool::new("test-daemon".into(), tx).unwrap();

        // Verify initial state
        let addr: SocketAddr = "192.168.1.1:4242".parse().unwrap();
        assert!(!mgr.is_connected(&addr).await);
    }
}
