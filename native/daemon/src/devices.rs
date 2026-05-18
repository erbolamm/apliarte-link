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

    pub fn list_input_peripherals() -> Result<Vec<PeripheralInfo>> {
        // Linux HID scanning requires libudev; return empty for now
        Ok(Vec::new())
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
