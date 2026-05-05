// Javier: Detección de borde de pantalla — función pura que decide si el cursor
// ha llegado al borde configurado y debe activar el SENDER.
//
// Analogía Flutter: como un Alignment check — pura aritmética de coordenadas.
// Sin I/O, sin estado, 100% testable con unit tests.
//
// Spec: A1, A2, A5 (activación por borde de pantalla)
// Diseño: parte del callback CGEventTap que decide activar/desactivar SwitchState.active

/// Punto en pantalla — coordenadas absolutas de pantalla macOS.
/// Javier: En macOS el origen (0,0) es la esquina superior-izquierda del display principal.
/// x crece hacia la derecha, y crece hacia abajo.
///
/// Nota: usamos nuestro propio tipo para no depender de core-graphics fuera del módulo macOS.
/// En capture_macos.rs se convierte el CGPoint de Apple a este tipo antes de llamar is_at_edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenPoint {
    pub x: f64,
    pub y: f64,
}

/// Verifica si el punto está en el borde indicado de la pantalla.
///
/// Javier: `screen_dims` es (ancho, alto) en píxeles del display principal.
/// `side` es el nombre del borde: "left", "right", "top", "bottom".
///
/// Un cursor está "en el borde" cuando:
///   - right:  x >= screen_width - 1
///   - left:   x <= 0
///   - top:    y <= 0
///   - bottom: y >= screen_height - 1
///
/// Por qué -1: macOS reporta el cursor en screen_width-1 como máximo alcanzable
/// (igual que los índices de array — el último píxel válido es width-1).
///
/// Spec A1 (right edge), A2 (disabled), A5 (no peer)
pub fn is_at_edge(point: ScreenPoint, screen_dims: (f64, f64), side: &str) -> bool {
    let (width, height) = screen_dims;

    // Javier: Coordenadas fuera de pantalla (negativas o más allá) también cuentan como borde
    match side {
        "right" => point.x >= width - 1.0,
        "left" => point.x <= 0.0,
        "top" => point.y <= 0.0,
        "bottom" => point.y >= height - 1.0,
        _ => {
            tracing::warn!("is_at_edge: lado '{}' no reconocido (válidos: left, right, top, bottom)", side);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: (f64, f64) = (1920.0, 1080.0);

    // ----------------------------------------------------------------
    // Javier: Tests escritos PRIMERO (TDD RED phase) — Spec A1, A2, A5.
    // ----------------------------------------------------------------

    // --- Borde derecho (right) ---

    #[test]
    fn test_at_right_edge_exact() {
        // x == width-1 → está en el borde derecho
        let p = ScreenPoint { x: 1919.0, y: 540.0 };
        assert!(is_at_edge(p, SCREEN, "right"), "x == width-1 debe ser borde derecho");
    }

    #[test]
    fn test_at_right_edge_beyond() {
        // x > width-1 (overflow) → también cuenta como borde
        let p = ScreenPoint { x: 1925.0, y: 540.0 };
        assert!(is_at_edge(p, SCREEN, "right"), "x > width-1 también debe ser borde derecho");
    }

    #[test]
    fn test_not_at_right_edge() {
        // x < width-1 → dentro de la pantalla
        let p = ScreenPoint { x: 960.0, y: 540.0 };
        assert!(!is_at_edge(p, SCREEN, "right"), "centro no debe ser borde derecho");
    }

    // --- Borde izquierdo (left) ---

    #[test]
    fn test_at_left_edge_exact() {
        let p = ScreenPoint { x: 0.0, y: 540.0 };
        assert!(is_at_edge(p, SCREEN, "left"), "x == 0 debe ser borde izquierdo");
    }

    #[test]
    fn test_at_left_edge_negative() {
        let p = ScreenPoint { x: -5.0, y: 540.0 };
        assert!(is_at_edge(p, SCREEN, "left"), "x negativa también debe ser borde izquierdo");
    }

    #[test]
    fn test_not_at_left_edge() {
        let p = ScreenPoint { x: 100.0, y: 540.0 };
        assert!(!is_at_edge(p, SCREEN, "left"), "x > 0 no debe ser borde izquierdo");
    }

    // --- Borde superior (top) ---

    #[test]
    fn test_at_top_edge_exact() {
        let p = ScreenPoint { x: 960.0, y: 0.0 };
        assert!(is_at_edge(p, SCREEN, "top"), "y == 0 debe ser borde superior");
    }

    #[test]
    fn test_at_top_edge_negative() {
        let p = ScreenPoint { x: 960.0, y: -3.0 };
        assert!(is_at_edge(p, SCREEN, "top"), "y negativa también debe ser borde superior");
    }

    #[test]
    fn test_not_at_top_edge() {
        let p = ScreenPoint { x: 960.0, y: 50.0 };
        assert!(!is_at_edge(p, SCREEN, "top"), "y > 0 no debe ser borde superior");
    }

    // --- Borde inferior (bottom) ---

    #[test]
    fn test_at_bottom_edge_exact() {
        let p = ScreenPoint { x: 960.0, y: 1079.0 };
        assert!(is_at_edge(p, SCREEN, "bottom"), "y == height-1 debe ser borde inferior");
    }

    #[test]
    fn test_at_bottom_edge_beyond() {
        let p = ScreenPoint { x: 960.0, y: 1090.0 };
        assert!(is_at_edge(p, SCREEN, "bottom"), "y > height-1 también debe ser borde inferior");
    }

    #[test]
    fn test_not_at_bottom_edge() {
        let p = ScreenPoint { x: 960.0, y: 500.0 };
        assert!(!is_at_edge(p, SCREEN, "bottom"), "y < height-1 no debe ser borde inferior");
    }

    // --- Esquinas (corners) ---

    #[test]
    fn test_corner_top_right_matches_both_right_and_top() {
        let p = ScreenPoint { x: 1919.0, y: 0.0 };
        assert!(is_at_edge(p, SCREEN, "right"), "esquina superior-derecha debe ser borde derecho");
        assert!(is_at_edge(p, SCREEN, "top"), "esquina superior-derecha debe ser borde superior");
    }

    #[test]
    fn test_corner_bottom_left_matches_both_left_and_bottom() {
        let p = ScreenPoint { x: 0.0, y: 1079.0 };
        assert!(is_at_edge(p, SCREEN, "left"), "esquina inferior-izquierda debe ser borde izquierdo");
        assert!(is_at_edge(p, SCREEN, "bottom"), "esquina inferior-izquierda debe ser borde inferior");
    }

    // --- Centro (no es ningún borde) ---

    #[test]
    fn test_center_is_not_any_edge() {
        let p = ScreenPoint { x: 960.0, y: 540.0 };
        assert!(!is_at_edge(p, SCREEN, "left"));
        assert!(!is_at_edge(p, SCREEN, "right"));
        assert!(!is_at_edge(p, SCREEN, "top"));
        assert!(!is_at_edge(p, SCREEN, "bottom"));
    }

    // --- Lado inválido ---

    #[test]
    fn test_invalid_side_returns_false() {
        let p = ScreenPoint { x: 1919.0, y: 540.0 };
        // Un side desconocido nunca activa el borde — no debe causar panic
        assert!(!is_at_edge(p, SCREEN, "diagonal"), "side inválido debe retornar false");
        assert!(!is_at_edge(p, SCREEN, ""), "side vacío debe retornar false");
    }

    // --- Pantallas pequeñas (resoluciones no estándar) ---

    #[test]
    fn test_at_edge_small_screen() {
        let small = (800.0, 600.0);
        let p = ScreenPoint { x: 799.0, y: 300.0 };
        assert!(is_at_edge(p, small, "right"), "borde derecho en pantalla 800x600");
    }

    // --- Spec A2: si edge_switch está desactivado, capture_macos no llama is_at_edge ---
    // No es testable aquí (lógica en capture_macos), pero documentamos el invariante:
    // is_at_edge siempre retorna basado en posición; la decisión de llamarla o no es del caller.
}
