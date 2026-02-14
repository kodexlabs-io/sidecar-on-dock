//! Enumerate connected Thunderbolt devices and available Sidecar (iPad) devices
//! so the user can populate their config file.

use std::process::Command;

use crate::sidecar_ffi;

/// A Thunderbolt peripheral discovered via `system_profiler`.
#[derive(Debug)]
pub struct ThunderboltDevice {
    pub name: String,
    pub uid: String,
    pub vendor: String,
}

/// An iPad reachable for Sidecar display extension.
#[derive(Debug)]
pub struct SidecarDevice {
    pub name: String,
}

/// Discover non-Apple Thunderbolt devices by parsing `system_profiler SPThunderboltDataType -xml`.
pub fn discover_thunderbolt_devices() -> Result<Vec<ThunderboltDevice>, String> {
    let output = Command::new("system_profiler")
        .args(["SPThunderboltDataType", "-xml"])
        .output()
        .map_err(|e| format!("Failed to run system_profiler: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "system_profiler exited with status {}",
            output.status
        ));
    }

    let value: plist::Value =
        plist::from_bytes(&output.stdout).map_err(|e| format!("Failed to parse plist: {e}"))?;

    let mut devices = Vec::new();
    extract_devices(&value, &mut devices);
    Ok(devices)
}

/// List iPads reachable for Sidecar display extension via SidecarCore.
pub fn discover_sidecar_devices() -> Vec<SidecarDevice> {
    if sidecar_ffi::load_framework().is_err() {
        log::warn!("Could not load SidecarCore framework");
        return Vec::new();
    }

    let Some(cls) = sidecar_ffi::display_manager_class() else {
        log::warn!("SidecarDisplayManager class not found");
        return Vec::new();
    };

    unsafe {
        let Some(manager) = sidecar_ffi::shared_manager(cls) else {
            log::warn!("Could not obtain SidecarDisplayManager.sharedManager");
            return Vec::new();
        };

        let Some(array) = sidecar_ffi::devices(&manager) else {
            log::warn!("Could not obtain devices array");
            return Vec::new();
        };

        let count = sidecar_ffi::array_count(&array);
        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            if let Some(device) = sidecar_ffi::array_object_at(&array, i) {
                let name = sidecar_ffi::device_name(&device)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "<unnamed>".into());
                result.push(SidecarDevice { name });
            }
        }
        result
    }
}

/// Run full discovery and print results to stdout.
pub fn print_discovery() -> Result<(), String> {
    println!("=== Thunderbolt Devices ===\n");

    let tb_devices = discover_thunderbolt_devices()?;
    if tb_devices.is_empty() {
        println!("  (no external Thunderbolt devices found)\n");
    } else {
        for d in &tb_devices {
            println!("  Name:   {}", d.name);
            println!("  Vendor: {}", d.vendor);
            println!("  UID:    {}", d.uid);
            println!();
        }
    }

    println!("=== Sidecar Devices (iPads) ===\n");

    let sc_devices = discover_sidecar_devices();
    if sc_devices.is_empty() {
        println!("  (no Sidecar-capable devices found)\n");
    } else {
        for d in &sc_devices {
            println!("  Name: {}", d.name);
        }
        println!();
    }

    if !tb_devices.is_empty() {
        println!("Hint: copy the UID of your dock into the config file.");
        println!(
            "Default config path: {}",
            crate::config::Config::default_path().display()
        );
    }

    Ok(())
}

fn extract_devices(value: &plist::Value, out: &mut Vec<ThunderboltDevice>) {
    let Some(root_array) = value.as_array() else {
        return;
    };

    for entry in root_array {
        if let Some(dict) = entry.as_dictionary()
            && let Some(items) = dict.get("_items").and_then(|v| v.as_array())
        {
            for item in items {
                extract_device_from_dict(item, out);
            }
        }
    }
}

/// Extract non-Apple devices from a plist dict, recursing into nested `_items`.
fn extract_device_from_dict(value: &plist::Value, out: &mut Vec<ThunderboltDevice>) {
    let Some(dict) = value.as_dictionary() else {
        return;
    };

    let vendor = dict
        .get("vendor_name_key")
        .and_then(|v| v.as_string())
        .unwrap_or("")
        .to_string();

    if vendor != "Apple Inc." {
        let name = dict
            .get("device_name_key")
            .or_else(|| dict.get("_name"))
            .and_then(|v| v.as_string())
            .unwrap_or("Unknown")
            .to_string();

        let uid = dict
            .get("switch_uid_key")
            .and_then(|v| v.as_string())
            .unwrap_or("N/A")
            .to_string();

        out.push(ThunderboltDevice { name, uid, vendor });
    }

    if let Some(items) = dict.get("_items").and_then(|v| v.as_array()) {
        for item in items {
            extract_device_from_dict(item, out);
        }
    }
}
