// Javier: Parser de hotkeys — convierte strings como "ctrl+alt+tab" en un matcher
// que puede verificar si una combinación de teclas activa el hotkey.
//
// Esta lógica es pura (sin I/O ni OS) y está completamente cubierta por unit tests.
// Analogía Flutter: como un InputDecoration.validator — pura función que transforma y valida.
//
// Spec: A3, A4 (activación por hotkey)
// Diseño: D2 (parte del callback del tap que decide activar/desactivar)

use anyhow::{anyhow, Result};

/// Teclas modificadoras reconocidas en un hotkey.
/// Javier: Las mismas que Modifiers en protocol.rs pero independientes del protocolo UDP.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HotkeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool, // Cmd en macOS, Windows key en Win, Super en Linux
}

/// Un hotkey parseado, listo para comparar contra eventos reales.
/// Javier: Piensa en esto como el "patrón compilado" del hotkey — se parsea una vez
/// al arrancar y luego se usa miles de veces por segundo en el tap callback.
#[derive(Debug, Clone, PartialEq)]
pub struct HotkeyMatcher {
    pub modifiers: HotkeyModifiers,
    /// Nombre de la tecla principal en minúsculas (ej: "tab", "f1", "a")
    pub key_name: String,
}

impl HotkeyMatcher {
    /// Verifica si este hotkey coincide con los flags CGEvent y el código de tecla recibidos.
    ///
    /// Javier: El callback del tap llama a este método en cada KeyDown.
    /// Si retorna true → togglear SwitchState.active.
    ///
    /// `flags`: CGEventFlags del sistema (bitmask — ver constantes macOS abajo)
    /// `key_code`: código de tecla virtual macOS (CGKeyCode / kVK_*)
    pub fn matches(&self, flags: u64, key_code: u16) -> bool {
        // Verificar modificadores
        let needs_ctrl = self.modifiers.ctrl;
        let needs_alt = self.modifiers.alt;
        let needs_shift = self.modifiers.shift;
        let needs_meta = self.modifiers.meta;

        // Javier: Máscaras de los flags CGEvent en macOS
        // Equivalentes a NX_COMMANDMASK, NX_CONTROLMASK, etc.
        let has_ctrl = (flags & 0x0004_0000) != 0;  // kCGEventFlagMaskControl
        let has_alt = (flags & 0x0008_0000) != 0;   // kCGEventFlagMaskAlternate
        let has_shift = (flags & 0x0002_0000) != 0; // kCGEventFlagMaskShift
        let has_meta = (flags & 0x0010_0000) != 0;  // kCGEventFlagMaskCommand

        if needs_ctrl != has_ctrl
            || needs_alt != has_alt
            || needs_shift != has_shift
            || needs_meta != has_meta
        {
            return false;
        }

        // Verificar la tecla principal por key_code
        let expected_code = key_name_to_keycode(&self.key_name);
        match expected_code {
            Some(code) => key_code == code,
            // Si la tecla no está en nuestra tabla de mappings, no coincide
            None => false,
        }
    }
}

/// Parsea un string de hotkey (ej: "ctrl+alt+tab") y retorna un HotkeyMatcher.
///
/// Javier: Esta es la función pública que se llama al arrancar con la config cargada.
/// Retorna error si el formato es inválido o la tecla principal no es reconocida.
///
/// Formato aceptado:
///   - Separador: "+" (ej: "ctrl+alt+tab")
///   - Modificadores válidos: ctrl, alt, shift, meta, cmd (sinónimo de meta), command (sinónimo)
///   - Teclas válidas: tab, space, return/enter, escape/esc, f1-f12, a-z, 0-9, etc.
///   - Case insensitive: "CTRL+ALT+TAB" == "ctrl+alt+tab"
///
/// Spec A3, A4 — la tecla se configura en cfg.input.hotkey como Option<String>
pub fn parse_hotkey(s: &str) -> Result<HotkeyMatcher> {
    if s.trim().is_empty() {
        return Err(anyhow!("hotkey string vacío"));
    }

    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
    if parts.is_empty() {
        return Err(anyhow!("hotkey no tiene partes"));
    }

    let mut modifiers = HotkeyModifiers::default();
    let mut key_name: Option<String> = None;

    for part in &parts {
        let lower = part.to_lowercase();
        match lower.as_str() {
            "ctrl" | "control" => modifiers.ctrl = true,
            "alt" | "option" | "opt" => modifiers.alt = true,
            "shift" => modifiers.shift = true,
            "meta" | "cmd" | "command" | "win" | "super" => modifiers.meta = true,
            other => {
                // Es la tecla principal
                if key_name.is_some() {
                    return Err(anyhow!(
                        "hotkey '{}' tiene más de una tecla principal",
                        s
                    ));
                }
                // Verificar que la tecla es conocida
                if key_name_to_keycode(other).is_none() {
                    return Err(anyhow!(
                        "tecla '{}' no reconocida en hotkey '{}'",
                        other,
                        s
                    ));
                }
                key_name = Some(other.to_string());
            }
        }
    }

    let key_name = key_name.ok_or_else(|| {
        anyhow!("hotkey '{}' no tiene tecla principal (solo modificadores)", s)
    })?;

    Ok(HotkeyMatcher { modifiers, key_name })
}

/// Traduce nombre de tecla a código virtual macOS (kVK_*).
///
/// Javier: Esta tabla mapea nombres amigables a los códigos que usa macOS internamente.
/// Los códigos están en /System/Library/Frameworks/Carbon.framework/Headers/HIToolbox/Events.h
/// Solo incluimos las teclas más usadas para hotkeys de sistema.
fn key_name_to_keycode(name: &str) -> Option<u16> {
    // Javier: Los códigos kVK son constantes del framework Carbon de Apple
    match name {
        // --- Teclas especiales ---
        "tab" => Some(0x30),
        "return" | "enter" => Some(0x24),
        "space" => Some(0x31),
        "escape" | "esc" => Some(0x35),
        "delete" | "backspace" => Some(0x33),
        "forward_delete" | "del" => Some(0x75),
        "home" => Some(0x73),
        "end" => Some(0x77),
        "page_up" | "pageup" => Some(0x74),
        "page_down" | "pagedown" => Some(0x79),

        // --- Teclas de función F1–F12 ---
        "f1" => Some(0x7A),
        "f2" => Some(0x78),
        "f3" => Some(0x63),
        "f4" => Some(0x76),
        "f5" => Some(0x60),
        "f6" => Some(0x61),
        "f7" => Some(0x62),
        "f8" => Some(0x64),
        "f9" => Some(0x65),
        "f10" => Some(0x6D),
        "f11" => Some(0x67),
        "f12" => Some(0x6F),

        // --- Teclas de cursor ---
        "left" => Some(0x7B),
        "right" => Some(0x7C),
        "down" => Some(0x7D),
        "up" => Some(0x7E),

        // --- Letras (layout ANSI) ---
        "a" => Some(0x00),
        "b" => Some(0x0B),
        "c" => Some(0x08),
        "d" => Some(0x02),
        "e" => Some(0x0E),
        "f" => Some(0x03),
        "g" => Some(0x05),
        "h" => Some(0x04),
        "i" => Some(0x22),
        "j" => Some(0x26),
        "k" => Some(0x28),
        "l" => Some(0x25),
        "m" => Some(0x2E),
        "n" => Some(0x2D),
        "o" => Some(0x1F),
        "p" => Some(0x23),
        "q" => Some(0x0C),
        "r" => Some(0x0F),
        "s" => Some(0x01),
        "t" => Some(0x11),
        "u" => Some(0x20),
        "v" => Some(0x09),
        "w" => Some(0x0D),
        "x" => Some(0x07),
        "y" => Some(0x10),
        "z" => Some(0x06),

        // --- Dígitos fila superior ---
        "0" => Some(0x1D),
        "1" => Some(0x12),
        "2" => Some(0x13),
        "3" => Some(0x14),
        "4" => Some(0x15),
        "5" => Some(0x17),
        "6" => Some(0x16),
        "7" => Some(0x1A),
        "8" => Some(0x1C),
        "9" => Some(0x19),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----------------------------------------------------------------
    // Javier: Tests escritos PRIMERO (TDD RED phase) — Spec A3, A4.
    // Cubren parse_hotkey y HotkeyMatcher::matches con lógica pura.
    // ----------------------------------------------------------------

    // --- parse_hotkey: casos válidos ---

    #[test]
    fn test_parse_hotkey_ctrl_alt_tab() {
        let m = parse_hotkey("ctrl+alt+tab").expect("debe parsear ctrl+alt+tab");
        assert!(m.modifiers.ctrl);
        assert!(m.modifiers.alt);
        assert!(!m.modifiers.shift);
        assert!(!m.modifiers.meta);
        assert_eq!(m.key_name, "tab");
    }

    #[test]
    fn test_parse_hotkey_ctrl_shift_f1() {
        let m = parse_hotkey("ctrl+shift+f1").expect("debe parsear ctrl+shift+f1");
        assert!(m.modifiers.ctrl);
        assert!(m.modifiers.shift);
        assert!(!m.modifiers.alt);
        assert_eq!(m.key_name, "f1");
    }

    #[test]
    fn test_parse_hotkey_case_insensitive() {
        // Spec A3: el string puede venir en mayúsculas de la config
        let m = parse_hotkey("CTRL+ALT+TAB").expect("debe parsear en mayúsculas");
        assert!(m.modifiers.ctrl);
        assert!(m.modifiers.alt);
        assert_eq!(m.key_name, "tab");
    }

    #[test]
    fn test_parse_hotkey_cmd_alias() {
        // "cmd" es sinónimo de "meta" en macOS
        let m = parse_hotkey("cmd+tab").expect("debe parsear cmd+tab");
        assert!(m.modifiers.meta);
        assert_eq!(m.key_name, "tab");
    }

    #[test]
    fn test_parse_hotkey_single_modifier_and_letter() {
        let m = parse_hotkey("ctrl+a").expect("debe parsear ctrl+a");
        assert!(m.modifiers.ctrl);
        assert_eq!(m.key_name, "a");
    }

    // --- parse_hotkey: casos inválidos ---

    #[test]
    fn test_parse_hotkey_empty_string_fails() {
        assert!(parse_hotkey("").is_err(), "string vacío debe fallar");
        assert!(parse_hotkey("   ").is_err(), "string con espacios vacío debe fallar");
    }

    #[test]
    fn test_parse_hotkey_unknown_key_fails() {
        // Una tecla no reconocida debe retornar error
        assert!(
            parse_hotkey("ctrl+xyzzy").is_err(),
            "tecla desconocida debe fallar"
        );
    }

    #[test]
    fn test_parse_hotkey_only_modifiers_fails() {
        // Sin tecla principal, el hotkey no tiene sentido
        assert!(
            parse_hotkey("ctrl+alt").is_err(),
            "solo modificadores sin tecla principal debe fallar"
        );
    }

    #[test]
    fn test_parse_hotkey_multiple_keys_fails() {
        // "ctrl+tab+f1" tiene dos posibles teclas principales → error
        assert!(
            parse_hotkey("ctrl+tab+f1").is_err(),
            "múltiples teclas principales debe fallar"
        );
    }

    // --- HotkeyMatcher::matches ---

    #[test]
    fn test_matches_correct_combo() {
        let m = parse_hotkey("ctrl+alt+tab").unwrap();
        // flags: ctrl (0x40000) + alt (0x80000) = 0xC0000
        let flags = 0x0004_0000u64 | 0x0008_0000u64;
        let tab_keycode = 0x30u16;
        assert!(m.matches(flags, tab_keycode), "debe coincidir con ctrl+alt+tab");
    }

    #[test]
    fn test_matches_wrong_key_fails() {
        let m = parse_hotkey("ctrl+alt+tab").unwrap();
        let flags = 0x0004_0000u64 | 0x0008_0000u64;
        let space_keycode = 0x31u16;
        assert!(
            !m.matches(flags, space_keycode),
            "no debe coincidir si la tecla es space en vez de tab"
        );
    }

    #[test]
    fn test_matches_missing_modifier_fails() {
        let m = parse_hotkey("ctrl+alt+tab").unwrap();
        // Solo ctrl, falta alt
        let flags = 0x0004_0000u64;
        let tab_keycode = 0x30u16;
        assert!(
            !m.matches(flags, tab_keycode),
            "no debe coincidir si falta modificador"
        );
    }

    #[test]
    fn test_matches_extra_modifier_fails() {
        let m = parse_hotkey("ctrl+alt+tab").unwrap();
        // ctrl + alt + shift (extra) → no debe coincidir porque el matcher no espera shift
        let flags = 0x0004_0000u64 | 0x0008_0000u64 | 0x0002_0000u64;
        let tab_keycode = 0x30u16;
        assert!(
            !m.matches(flags, tab_keycode),
            "no debe coincidir si hay modificadores extra no esperados"
        );
    }

    #[test]
    fn test_matches_no_flags_fails() {
        let m = parse_hotkey("ctrl+tab").unwrap();
        assert!(
            !m.matches(0, 0x30),
            "sin flags no debe coincidir con hotkey que requiere ctrl"
        );
    }

    // --- Spec A4: None passthrough (verificado en capture_macos, no aquí, pero documentado) ---
    // Si cfg.input.hotkey es None, capture_macos simplemente no crea HotkeyMatcher.
    // Este módulo no necesita saber de ese caso — la lógica de guardado está en capture_macos.
}
