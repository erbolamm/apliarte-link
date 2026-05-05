// Javier: Este módulo gestiona la configuración del servicio.
// Analogía Flutter: como un AppConfig o SharedPreferences,
// pero guardado en un archivo JSON en el directorio del usuario.
//
// El archivo de configuración se guarda en:
//   macOS:   ~/Library/Application Support/apliarte-switch/config.json
//   Linux:   ~/.config/apliarte-switch/config.json
//   Windows: %APPDATA%\apliarte-switch\config.json

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuración completa de la app.
/// Javier: Piensa en esto como el "modelo" de un Form en Flutter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub network: NetworkConfig,
    pub input: InputConfig,
    pub ui: UiConfig,
}

/// Configuración de red
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Puerto UDP para comunicarse con otros PCs (por defecto 4242)
    pub port: u16,
    /// Si se usa encriptación TLS en los paquetes de red
    pub encrypted: bool,
    /// Dirección IP:puerto del peer destino (ej: "192.168.1.50:4242").
    /// Javier: Si se especifica, siempre se usa como destino UDP del SENDER.
    /// Si no se especifica, se usa el último peer que nos mandó un paquete (last_peer).
    /// Con #[serde(default)] las configs antiguas sin este campo siguen funcionando.
    #[serde(default)]
    pub peer: Option<String>,
}

/// Configuración de cómo se activa el switch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    /// Tecla para cambiar de PC manualmente (ej: "ctrl+alt+tab")
    pub hotkey: Option<String>,
    /// Si el switch se activa al mover el ratón al borde de la pantalla
    pub edge_switch: bool,
    /// En qué borde activa el switch (left, right, top, bottom)
    pub edge_side: String,
}

/// Configuración de la interfaz
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Tema visual: "dark" o "light"
    pub theme: String,
    /// Idioma: "es" o "en"
    pub language: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig {
                port: 4242,
                encrypted: true,
                peer: None,
            },
            input: InputConfig {
                hotkey: Some("ctrl+alt+tab".to_string()),
                edge_switch: true,
                edge_side: "right".to_string(),
            },
            ui: UiConfig {
                theme: "dark".to_string(),
                language: "es".to_string(),
            },
        }
    }
}

/// Devuelve la ruta donde se guarda config.json según el SO
fn config_path() -> PathBuf {
    // Javier: dirs::config_dir() detecta automáticamente la carpeta correcta
    // por sistema operativo. Como Platform.isIOS en Flutter pero para el escritorio.
    let base = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("apliarte-switch").join("config.json")
}

/// Carga la configuración desde disco.
/// Si no existe el archivo, crea uno con los valores por defecto.
pub fn load() -> Result<AppConfig> {
    let path = config_path();

    if path.exists() {
        // Javier: Leemos el archivo JSON y lo convertimos a AppConfig
        // Es como json.decode() en Dart, pero automático con serde
        let content = std::fs::read_to_string(&path)?;
        let cfg: AppConfig = serde_json::from_str(&content)?;
        Ok(cfg)
    } else {
        // Si no existe, creamos la configuración por defecto y la guardamos
        let cfg = AppConfig::default();
        save(&cfg)?;
        Ok(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Javier: Verifica que configs antiguas (sin campo `peer`) deserializan correctamente
    // Spec F1 — backward-compatible config loading
    #[test]
    fn test_network_config_without_peer_defaults_to_none() {
        let json = r#"{"port": 4242, "encrypted": false}"#;
        let cfg: NetworkConfig = serde_json::from_str(json).expect("debe deserializar sin peer");
        assert_eq!(cfg.peer, None, "peer debe ser None cuando no está en el JSON");
    }

    // Javier: Verifica que configs nuevas con campo `peer` lo leen correctamente
    // Spec F2 — explicit peer field used as UDP target
    #[test]
    fn test_network_config_with_peer_field() {
        let json = r#"{"port": 4242, "encrypted": false, "peer": "192.168.1.50:4242"}"#;
        let cfg: NetworkConfig = serde_json::from_str(json).expect("debe deserializar con peer");
        assert_eq!(
            cfg.peer,
            Some("192.168.1.50:4242".to_string()),
            "peer debe leerse del JSON"
        );
    }
}

/// Guarda la configuración en disco
pub fn save(cfg: &AppConfig) -> Result<()> {
    let path = config_path();
    // Creamos el directorio si no existe
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // Serializamos a JSON con formato bonito
    let content = serde_json::to_string_pretty(cfg)?;
    std::fs::write(&path, content)?;
    Ok(())
}
