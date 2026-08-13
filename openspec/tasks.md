# Tasks — Fase 1: Protocolo Confiable ✅ COMPLETADO

## T1: Agregar dependencias ✅
- [x] `Cargo.toml`: agregar `rmp-serde`, `tokio-rustls`, `rcgen`
- [x] Verificar build con `cargo check`

## T2: Crear `network/transport.rs` ✅
- [x] `TransportConfig`, `ConnectionManager`, `Connection`
- [x] TLS config generation (`rcgen`)
- [x] TCP listener + accept loop
- [x] TCP outbound connect + handshake
- [x] MessagePack framing (length-prefixed)
- [x] Keep-alive (Ping cada 3s / timeout 9s — activado en `connect()` y `handle_inbound()`)
- [x] Connection pool (`HashMap<SocketAddr, Sender<Frame>>`)
- [x] `send()` y `recv_stream()`

## T3: Actualizar `input/protocol.rs` ✅
- [x] Agregar `Frame` enum (Handshake, Input, Control, KeepAlive)
- [x] Agregar `HandshakeMessage` enum
- [x] Agregar `KeepAlive` enum
- [x] Agregar `seq: u32` a `InputEvent`

## T4: Actualizar `network/discovery.rs` ✅
- [x] Agregar campo `transport` y `version` al mensaje HELLO

## T5: Actualizar `network/mod.rs` ✅
- [x] Reemplazar UDP serve con `ConnectionManager`
- [x] Agregar `mod_handle_control()` para frames de control entrantes

## T6: Actualizar `control.rs` ✅
- [x] `send_input_to_active_peer()` usa `ConnectionManager`
- [x] `send_control_message()` usa `ConnectionManager`
- [x] `apply_transfer()` despacha vía tokio::spawn

## T7: Actualizar `input/macos.rs` ✅
- [x] `start_switch_listener()` recibe `ConnectionManager`
- [x] Callback de captura usa `ConnectionManager::send()` vía tokio::spawn

## T8: Actualizar `input/mod.rs` ✅
- [x] `serve()` recibe `ConnectionManager` y `(SocketAddr, Frame)`
- [x] Dispatch de Frame::Input, Frame::Control, Frame::KeepAlive, Frame::Handshake

## T9: Actualizar `main.rs` ✅
- [x] Instanciar `ConnectionManager`
- [x] Inicializar rustls crypto provider

## T10: Actualizar `ipc/mod.rs` ✅
- [x] `serve()` y `handle_command()` reciben `ConnectionManager`

## T11: Verificar build y tests ✅
- [x] `cargo build` compila sin errores
- [x] `cargo test` 5/5 tests pasan
- [x] Warnings conocidos: 3 métodos unused (se usarán cuando se active keep-alive)
