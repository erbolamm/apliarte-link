// Javier: Protocolo de comunicación — define el "lenguaje" que usan los PCs entre sí.
// Analogía Flutter: es como los modelos de datos (clases con fromJson/toJson)
// que defines para la API de tu backend.
//
// Cada MovimientoRatón o TeclaPulsada se convierte en un InputEvent
// que se serializa a bytes y se envía por UDP.

use serde::{Deserialize, Serialize};

/// Un evento de input que viaja por la red.
/// Javier: Es como un sealed class en Dart — puede ser varios tipos
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputEvent {
    /// Movimiento del ratón
    MouseMove {
        /// Coordenada X relativa (cuántos píxeles se movió)
        dx: f64,
        /// Coordenada Y relativa
        dy: f64,
    },
    /// Botón del ratón pulsado o soltado
    MouseButton { button: MouseButton, pressed: bool },
    /// Rueda del ratón
    MouseScroll { dx: f64, dy: f64 },
    /// Tecla del teclado pulsada o soltada
    KeyPress {
        /// Código de la tecla (independiente del SO)
        key_code: u32,
        pressed: bool,
        /// Teclas modificadoras activas (Ctrl, Alt, Shift, Meta)
        modifiers: Modifiers,
    },
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
