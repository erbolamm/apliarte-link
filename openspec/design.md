# Design — Fase 1: Protocolo Confiable

## Data Flow

```
┌─────────────────────────────────────────────────────────┐
│  daemon A (agenmac)                                     │
│                                                         │
│  main.rs                                                │
│  ├── TcpListener::bind("0.0.0.0:4242")                  │
│  ├── ConnectionManager::new()                           │
│  │   ├── inbound:  accept loop → handshake → pool       │
│  │   └── outbound: connect() → handshake → pool         │
│  └── event_tx (mpsc channel) ← recv de todas las conns  │
│                                                         │
│  input/macos.rs                                         │
│  └── CGEventTap captura → send_input_to_active_peer()   │
│      └── ConnectionManager::send(active, frame)          │
│                                                         │
│  ipc/mod.rs (sin cambios)                               │
└─────────────────────────────────────────────────────────┘
                         │
                    TCP 4242 + TLS + MessagePack
                         │
┌─────────────────────────────────────────────────────────┐
│  daemon B (agenpc)                                      │
│  (estructura idéntica, roles intercambiables)           │
└─────────────────────────────────────────────────────────┘
```

## File Changes

### Nuevo: `native/daemon/src/network/transport.rs`

```
pub struct TransportConfig { port, tls }
pub struct ConnectionManager { ... }
pub struct Connection { ... }

impl ConnectionManager {
    pub async fn new(config, local_name) -> Self
    pub async fn connect(&mut self, addr) -> Result<()>
    pub async fn accept_loop(&self, listener) -> !
    pub fn send(&self, addr, frame) -> Result<()>
    pub fn recv_stream(&self) -> Receiver<(SocketAddr, Frame)>
}
```

Internamente:
- `inbound_tx: mpsc::Sender<(SocketAddr, Frame)>` para forwardear eventos recibidos
- `connections: Arc<RwLock<HashMap<SocketAddr, Sender<Frame>>>>` pool de senders
- `tls_config: Arc<ClientConfig>` para outbound, `Arc<ServerConfig>` para inbound

### Modificado: `native/daemon/src/input/protocol.rs`

Agregar:
```rust
#[derive(Serialize, Deserialize)]
pub enum HandshakeMessage {
    Hello { version: (u16, u16), name: String },
    HelloAck { version: (u16, u16), name: String },
}

#[derive(Serialize, Deserialize)]
pub enum KeepAlive { Ping(u32), Pong(u32) }

#[derive(Serialize, Deserialize)]
pub enum Frame {
    Handshake(HandshakeMessage),
    Input(InputEvent),
    Control(ControlMessage),
    KeepAlive(KeepAlive),
}
```

Agregar `seq: u32` a `InputEvent` (para ordenamiento/dedup).

### Modificado: `native/daemon/src/network/mod.rs`

- Reemplazar `UdpSocket` + `recv_from` loop con `ConnectionManager`
- `start()` ahora devuelve `ConnectionManager` además del `PeerList`
- Eliminar `send_event()` (reemplazado por `ConnectionManager::send()`)

### Modificado: `native/daemon/src/network/discovery.rs`

- Mensaje HELLO agrega campo `transport: "tcp"` y `version: [1, 0]`
- Sin cambios en el transporte (sigue UDP+JSON)

### Modificado: `native/daemon/src/control.rs`

- `send_input_to_active_peer()` recibe `ConnectionManager` en vez de `UdpSocket`
- `send_control_message()` recibe `ConnectionManager` en vez de `UdpSocket`

### Modificado: `native/daemon/src/input/macos.rs`

- `start_switch_listener()` recibe `ConnectionManager` en vez de `UdpSocket`
- El callback de captura usa `ConnectionManager::send()`

### Modificado: `native/daemon/src/main.rs`

- Instanciar `ConnectionManager` en vez de `UdpSocket`
- Pasar `ConnectionManager` a `input::serve()` y `ipc::serve()`

### Modificado: `native/daemon/Cargo.toml`

```toml
rmp-serde = "1"          # MessagePack
tokio-rustls = "0.26"    # TLS sobre tokio
rustls = "0.23"          # ya está, verificar features
x509-cert = "0.2"        # generar certificados autofirmados
rcgen = "0.13"           # alternativa para cert generation
```

### Modificado: `native/daemon/src/config.rs`

```rust
pub struct NetworkConfig {
    pub port: u16,
    pub encrypted: bool,
    pub transport: String,   // "tcp" (nuevo)
}
```

## TLS Certificate Strategy

Usamos `rcgen` para generar un certificado autofirmado en memoria al iniciar el daemon. El mismo par de claves se usa para server y client (simplicidad para LAN de confianza).

```rust
fn generate_tls_config() -> (ServerConfig, ClientConfig) {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
    // configurar server y client con el mismo cert
}
```

En producción (Fase 2), esto se reemplaza con TOFU (Trust On First Use) o certificados firmados por el servidor de rendezvous.
