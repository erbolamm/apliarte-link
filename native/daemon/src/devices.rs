use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PeripheralInfo {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub bus: String,
    pub connected: bool,
    pub vendor_id: u16,
    pub product_id: u16,
}

#[cfg(not(target_os = "linux"))]
mod imp {
    use super::*;
    use hidapi::{DeviceInfo, HidApi};
    use std::collections::HashSet;

    pub fn list_input_peripherals() -> Result<Vec<PeripheralInfo>> {
        let api = HidApi::new()?;
        let mut seen = HashSet::new();
        let mut peripherals = Vec::new();

        for device in api.device_list() {
            let Some(kind) = classify_device(device) else {
                continue;
            };

            let id = build_device_id(device, &kind);
            if !seen.insert(id.clone()) {
                continue;
            }

            peripherals.push(PeripheralInfo {
                id,
                name: display_name(device, &kind),
                kind,
                bus: format!("{:?}", device.bus_type()).to_lowercase(),
                connected: true,
                vendor_id: device.vendor_id(),
                product_id: device.product_id(),
            });
        }

        peripherals.sort_by(|left, right| {
            left.kind
                .cmp(&right.kind)
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
        });

        Ok(peripherals)
    }

    fn classify_device(device: &DeviceInfo) -> Option<String> {
        if device.usage_page() == 0x01 {
            match device.usage() {
                0x02 => return Some("mouse".to_string()),
                0x06 => return Some("keyboard".to_string()),
                _ => {}
            }
        }

        classify_from_strings(device)
    }

    fn classify_from_strings(device: &DeviceInfo) -> Option<String> {
        let manufacturer = device.manufacturer_string().unwrap_or("");
        let product = device.product_string().unwrap_or("");
        let summary = format!("{} {}", manufacturer, product).to_lowercase();

        if contains_any(
            &summary,
            &[
                "keyboard",
                "teclado",
                "keypad",
                "magic keyboard",
                "internal keyboard",
            ],
        ) {
            Some("keyboard".to_string())
        } else if contains_any(
            &summary,
            &[
                "mouse",
                "raton",
                "trackpad",
                "touchpad",
                "magic mouse",
                "magic trackpad",
                "pointing",
                "multitouch",
            ],
        ) {
            Some("mouse".to_string())
        } else {
            None
        }
    }

    fn contains_any(summary: &str, keywords: &[&str]) -> bool {
        keywords.iter().any(|keyword| summary.contains(keyword))
    }

    fn display_name(device: &DeviceInfo, kind: &str) -> String {
        let product = device.product_string().unwrap_or("").trim();
        let manufacturer = device.manufacturer_string().unwrap_or("").trim();

        if !product.is_empty() {
            product.to_string()
        } else if !manufacturer.is_empty() {
            format!("{} {}", base_kind_label(kind), manufacturer)
        } else {
            format!(
                "{} {:04x}:{:04x}",
                base_kind_label(kind),
                device.vendor_id(),
                device.product_id()
            )
        }
    }

    fn build_device_id(device: &DeviceInfo, kind: &str) -> String {
        let suffix = device
            .serial_number()
            .map(str::to_owned)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| device.path().to_string_lossy().into_owned());

        format!(
            "{}-{:04x}-{:04x}-{}-{}",
            kind,
            device.vendor_id(),
            device.product_id(),
            device.interface_number(),
            suffix
        )
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use super::*;
    use evdev::{enumerate, BusType, Key};
    use std::collections::HashSet;

    /// Inventario HID real en Linux usando evdev (el kernel ya resuelve
    /// teclados y ratones por capacidades, no por texto).
    /// Mismo criterio que el listener de captura:
    ///   - teclado: tiene la tecla KEY_A
    ///   - ratón: tiene el botón BTN_LEFT
    /// Así descartamos ruido como Power Button / Sleep Button / Video Bus,
    /// que sí listan "kbd" en /proc pero no son teclados reales.
    pub fn list_input_peripherals() -> Result<Vec<PeripheralInfo>> {
        let mut peripherals = Vec::new();
        let mut seen = HashSet::new();

        for (path, dev) in enumerate() {
            let Some(keys) = dev.supported_keys() else {
                continue;
            };

            let is_mouse = keys.contains(Key::BTN_LEFT);
            let is_keyboard = keys.contains(Key::KEY_A);
            let kind = if is_keyboard {
                Some("keyboard")
            } else if is_mouse {
                Some("mouse")
            } else {
                None
            };
            let Some(kind) = kind else {
                continue;
            };

            let name = dev.name().unwrap_or("").trim().to_string();
            let input_id = dev.input_id();
            let vendor = input_id.vendor();
            let product = input_id.product();
            let bus = match input_id.bus_type() {
                BusType(0x03) => "usb".to_string(),
                BusType(0x05) => "bluetooth".to_string(),
                BusType(0x11) => "ps2".to_string(),
                BusType(0x06) => "virtual".to_string(),
                BusType(0x18) => "i2c".to_string(),
                BusType(other) => format!("bus{:04x}", other),
            };

            let id = format!(
                "{}-{:04x}-{:04x}-{}",
                kind,
                vendor,
                product,
                name.to_lowercase()
            );
            if seen.insert(id.clone()) {
                tracing::debug!(
                    "[devices] Periférico detectado: {} ({:?})",
                    name,
                    path
                );
                peripherals.push(PeripheralInfo {
                    id,
                    name: if name.is_empty() {
                        format!("{:04x}:{:04x}", vendor, product)
                    } else {
                        name
                    },
                    kind: kind.to_string(),
                    bus,
                    connected: true,
                    vendor_id: vendor,
                    product_id: product,
                });
            }
        }

        peripherals.sort_by(|left, right| {
            left.kind
                .cmp(&right.kind)
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
        });

        Ok(peripherals)
    }
}

pub use imp::list_input_peripherals;

fn base_kind_label(kind: &str) -> &'static str {
    match kind {
        "keyboard" => "Teclado",
        "mouse" => "Raton",
        _ => "Periferico",
    }
}
