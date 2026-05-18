# Spec — Fase 1: Protocolo Confiable

## 1. Transport Layer

### 1.1 TCP Listener
- El daemon bindea `TcpListener` en `0.0.0.0:4242`
- Acepta conexiones entrantes de peers
- Cada conexión aceptada inicia handshake TLS + versión

### 1.2 TCP Outbound
- Al descubrir un peer, el daemon inicia `TcpStream::connect` a `peer_addr:4242`
- Tras conectar, inicia handshake TLS + versión
- Se mantiene la conexión abierta (persistente)

### 1.3 TLS
- Certificados autofirmados generados por sesión (en memoria, no en disco)
- `tokio-rustls` con `ring` como crypto backend
- Sin verificación de CA (confiamos en el handshake de versión + LAN)

### 1.4 Framing
- Length-prefixed: 4 bytes BE (u32) + payload MessagePack
- Max frame size: 1MB

## 2. Protocol Messages

### 2.1 Handshake (post-TLS)
```
Client → Server:  Hello { version: (1, 0), name: "agenmac" }
Server → Client:  HelloAck { version: (1, 0), name: "agenpc" }
```
Ambos lados usan `min(client_version, server_version)`.

### 2.2 Input Events (MessagePack)
```
MouseMove    { dx: f64, dy: f64, seq: u32 }
MouseButton  { button: u8, pressed: bool, seq: u32 }
MouseScroll  { dx: f64, dy: f64, seq: u32 }
KeyPress     { key_code: u32, pressed: bool, modifiers: u8, seq: u32 }
```

### 2.3 Control Messages
```
Assign  { source_name: String }
Release { source_name: String }
```

### 2.4 Keep-Alive
```
Ping { seq: u32 }
Pong { seq: u32 }
```
Enviar Ping cada 3s. Timeout tras 3 Pings sin respuesta (9s).

## 3. Connection Manager

### 3.1 Pool
- `HashMap<SocketAddr, ConnectionHandle>` con todas las conexiones activas
- `ConnectionHandle` = `tokio::sync::mpsc::Sender<Frame>` para enviar
- Cada conexión tiene su propia task de receive + keep-alive

### 3.2 Active Peer
- `active_peer: Arc<Mutex<Option<SocketAddr>>>` sigue igual
- Al cambiar, se usa la conexión del pool para el nuevo peer
- Si no hay conexión, se crea una nueva (connect + handshake)

### 3.3 Receive
- Cada conexión tiene un `mpsc::Receiver<Frame>` por donde llegan los eventos
- Los eventos se forwardean al `event_tx` central (igual que antes)

## 4. Discovery (sin cambios mayores)
- Sigue en UDP 4244 con JSON
- El mensaje HELLO ahora declara `transport: "tcp"` y `version: (1, 0)`
- Retrocompatible: si un peer viejo manda HELLO sin `transport`, se asume UDP

## 5. Config
- `network.port`: 4242 (sin cambios)
- `network.encrypted`: true → TLS activo (sin cambios, ahora se implementa)
- Nuevo: `network.transport`: "tcp" (default, inmutable por ahora)
