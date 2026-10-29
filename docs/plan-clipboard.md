# Plan de Arquitectura e Implementación: Portapapeles Compartido (Clipboard Sync)

Este documento detalla el plan técnico para la sincronización segura y bidireccional del portapapeles entre macOS, Windows y Linux en ApliArte Link.

## 1. Principios de diseño

1. **Protocolo ligero y seguro**:
   - El portapapeles viaja por el mismo canal confiable TCP + TLS (puerto 4242).
   - Se serializa con MessagePack para máxima eficiencia y compatibilidad binaria.
2. **Sin sobrecargar la red (Hash / Deduplicación)**:
   - Solo se transmiten cambios reales. Cada actualización calcula un hash (ej: SHA-256 o CRC64) del contenido para ignorar eventos duplicados.
3. **Límites de tamaño para proteger la memoria**:
   - Límite de texto: 2 MB.
   - Si en el futuro se soportan imágenes (PNG): límite de 10 MB.
4. **Respeto a la privacidad**:
   - Solo se comparte el portapapeles cuando hay una sesión activa de ApliArte Link y los peers están mutuamente emparejados.
   - Opción en configuración (`clipboard_sync: true/false`).

---

## 2. Tipos de datos en el protocolo

En `native/daemon/src/input/protocol.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardFormat {
    PlainText(String),
    ImagePng(Vec<u8>), // Fase posterior
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardPayload {
    pub format: ClipboardFormat,
    pub timestamp_ms: u64,
    pub content_hash: u64,
}

// Extensión de Frame
pub enum Frame {
    Handshake(HandshakeMessage),
    Input(InputEvent),
    Control(ControlMessage),
    KeepAlive(KeepAlive),
    Clipboard(ClipboardPayload), // <- Nuevo frame
}
```

---

## 3. Integración en el Daemon (Rust)

Se utilizará una abstracción multiplataforma o librerías nativas probadas (ejemplo: `arboard` o APIs del SO):

- **macOS**: `NSPasteboard` (`generalPasteboard`) vía bindings o `arboard`.
- **Windows**: `OpenClipboard`, `GetClipboardData(CF_UNICODETEXT)` / `SetClipboardData`.
- **Linux**: Wayland data control protocol / X11 selection clipboard (`xclip` / `wl-clipboard` / API nativa).

### Bucle de detección y envío:
1. Un hilo en segundo plano sondea periódicamente el portapapeles local (o escucha eventos de cambio donde el SO lo soporte, ej. `WM_CLIPBOARDUPDATE` en Windows).
2. Si el hash del contenido actual difiere del último hash conocido y no proviene de una sincronización remota:
   - Se emite un `Frame::Clipboard` a los peers conectados.

### Recepción e inyección:
1. Al recibir un `Frame::Clipboard(payload)` en `transport.rs`:
   - Se verifica que el hash sea nuevo.
   - Se actualiza el portapapeles local sin disparar un reenvío en bucle (flag `is_syncing`).

---

## 4. Pruebas y Criterios de Aceptación

1. **Test unitario**: Serialización y deserialización de `Frame::Clipboard` con MessagePack.
2. **Test de deduplicación**: Evitar bucles de retroalimentación infinita (origen -> destino -> origen).
3. **Test de estrés de tamaño**: Descarte seguro de payloads que superen el límite configurado (2 MB).
