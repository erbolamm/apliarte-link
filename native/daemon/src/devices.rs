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
    use std::collections::HashSet;

    /// Inventario HID real en Linux sin libudev:
    /// parseamos /proc/bus/input/devices (formato texto del kernel).
    /// Bloque típico:
    ///   I: Bus=0003 Vendor=046d Product=c52b Version=0111
    ///   N: Name="Logitech USB Receiver"
    ///   H: Handlers=kbd mouse0 event3 js0
    pub fn list_input_peripherals() -> Result<Vec<PeripheralInfo>> {
        let content = std::fs::read_to_string("/proc/bus/input/devices")?;
        let mut peripherals = Vec::new();
        let mut seen = HashSet::new();

        let mut bus: Option<String> = None;
        let mut vendor: u16 = 0;
        let mut product: u16 = 0;
        let mut name = String::new();
        let mut handlers = String::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                // Fin de bloque: clasificar y guardar
                if let Some(kind) = classify_handlers(&handlers) {
                    let id = format!(
                        "{}-{:04x}-{:04x}-{}",
                        kind,
                        vendor,
                        product,
                        name.trim().to_lowercase()
                    );
                    if seen.insert(id.clone()) {
                        peripherals.push(PeripheralInfo {
                            id,
                            name: if name.trim().is_empty() {
                                format!("{:04x}:{:04x}", vendor, product)
                            } else {
                                name.trim().to_string()
                            },
                            kind,
                            bus: bus.clone().unwrap_or_else(|| "unknown".to_string()),
                            connected: true,
                            vendor_id: vendor,
                            product_id: product,
                        });
                    }
                }
                bus = None;
                vendor = 0;
                product = 0;
                name.clear();
                handlers.clear();
                continue;
            }

            if let Some(rest) = line.strip_prefix("I:") {
                bus = parse_bus(rest);
                vendor = parse_hex_field(rest, "Vendor");
                product = parse_hex_field(rest, "Product");
            } else if let Some(rest) = line.strip_prefix("N:") {
                name = rest.trim().trim_matches('"').to_string();
            } else if let Some(rest) = line.strip_prefix("H:") {
                handlers = rest.to_string();
            }
        }

        // Último bloque sin línea vacía final
        if let Some(kind) = classify_handlers(&handlers) {
            let id = format!(
                "{}-{:04x}-{:04x}-{}",
                kind,
                vendor,
                product,
                name.trim().to_lowercase()
            );
            if seen.insert(id.clone()) {
                peripherals.push(PeripheralInfo {
                    id,
                    name: if name.trim().is_empty() {
                        format!("{:04x}:{:04x}", vendor, product)
                    } else {
                        name.trim().to_string()
                    },
                    kind,
                    bus: bus.clone().unwrap_or_else(|| "unknown".to_string()),
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

    fn classify_handlers(handlers: &str) -> Option<String> {
        let lower = handlers.to_lowercase();
        if lower.contains("kbd") {
            Some("keyboard".to_string())
        } else if lower.contains("mouse") {
            Some("mouse".to_string())
        } else {
            None
        }
    }

    fn parse_bus(line: &str) -> Option<String> {
        let bus_hex = parse_hex_field(line, "Bus");
        match bus_hex {
            0x0003 => Some("usb".to_string()),
            0x0005 => Some("bluetooth".to_string()),
            0x0011 => Some("ps2".to_string()),
            0x0006 => Some("serio".to_string()),
            other => Some(format!("bus{:04x}", other)),
        }
    }

    fn parse_hex_field(line: &str, field: &str) -> u16 {
        line.split_whitespace()
            .find_map(|token| {
                let token = token.trim_end_matches('=');
                token
                    .strip_prefix(field)
                    .and_then(|value| u16::from_str_radix(value.trim_start_matches('='), 16).ok())
            })
            .unwrap_or(0)
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
