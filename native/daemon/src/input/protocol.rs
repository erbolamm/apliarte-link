// Javier: Protocolo de comunicación — define el "lenguaje" que usan los PCs entre sí.
// Analogía Flutter: es como los modelos de datos (clases con fromJson/toJson)
// que defines para la API de tu backend.
//
// Cada MovimientoRatón o TeclaPulsada se convierte en un InputEvent
// que se serializa a bytes (MessagePack) y se envía por TCP+TLS.

use serde::{Deserialize, Serialize};

/// Un evento de input que viaja por la red.
/// Cada evento lleva un número de secuencia para ordenamiento y deduplicación.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    /// Movimiento del ratón
    MouseMove {
        dx: f64,
        dy: f64,
        /// Número de secuencia monotónico
        seq: u32,
    },
    /// Botón del ratón pulsado o soltado
    MouseButton {
        button: MouseButton,
        pressed: bool,
        seq: u32,
    },
    /// Rueda del ratón
    MouseScroll { dx: f64, dy: f64, seq: u32 },
    /// Tecla del teclado pulsada o soltada
    KeyPress {
        /// Código de la tecla (independiente del SO)
        key_code: u32,
        pressed: bool,
        /// Teclas modificadoras activas (Ctrl, Alt, Shift, Meta)
        modifiers: Modifiers,
        seq: u32,
    },
}

/// Mensajes de handshake de protocolo (post-TLS).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandshakeMessage {
    Hello { version: (u16, u16), name: String },
    HelloAck { version: (u16, u16), name: String },
}

/// Keep-alive para detectar peers caídos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeepAlive {
    Ping(u32),
    Pong(u32),
}

/// Unidad de transmisión en el canal TCP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Frame {
    Handshake(HandshakeMessage),
    Input(InputEvent),
    Control(ControlMessage),
    KeepAlive(KeepAlive),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlMessage {
    Assign { source_name: String },
    Release { source_name: String },
}

/// Qué botón del ratón se pulsó
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

/// Estado de las teclas modificadoras
/// Javier: Como un Map<String, bool> pero tipado con Rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool, // Cmd en Mac, Windows key en Win, Super en Linux
}
