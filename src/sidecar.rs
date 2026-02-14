//! High-level Sidecar connect / disconnect helpers.

use std::thread;
use std::time::Duration;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;

use crate::sidecar_ffi;

const MAX_RETRIES: u32 = 10;
const RETRY_DELAY: Duration = Duration::from_secs(2);

/// Ensure the SidecarCore framework is loaded.
pub fn ensure_loaded() -> bool {
    if sidecar_ffi::load_framework().is_err() {
        log::error!("Failed to load SidecarCore framework");
        return false;
    }
    true
}

/// Connect to an iPad via Sidecar, retrying until the device becomes available.
pub fn connect(ipad_name: Option<&str>) {
    let Some(cls) = sidecar_ffi::display_manager_class() else {
        log::error!("SidecarDisplayManager class not found");
        return;
    };

    unsafe {
        let Some(manager) = sidecar_ffi::shared_manager(cls) else {
            log::error!("Could not get SidecarDisplayManager.sharedManager");
            return;
        };

        for attempt in 1..=MAX_RETRIES {
            if let Some(device) = find_device(&manager, ipad_name) {
                log::info!("Connecting Sidecar...");
                sidecar_ffi::connect_to_device(&manager, &device);
                return;
            }

            if attempt == 1 {
                log_available_devices(&manager, ipad_name);
            }

            if attempt < MAX_RETRIES {
                log::info!(
                    "Sidecar device not available yet (attempt {attempt}/{MAX_RETRIES}), retrying in {}s...",
                    RETRY_DELAY.as_secs()
                );
                thread::sleep(RETRY_DELAY);
            }
        }

        match ipad_name {
            Some(name) => log::warn!("Sidecar device '{name}' not found after {MAX_RETRIES} attempts"),
            None => log::warn!("No Sidecar devices available after {MAX_RETRIES} attempts"),
        }
    }
}

/// Disconnect a currently-connected iPad from Sidecar.
pub fn disconnect(ipad_name: Option<&str>) {
    let Some(cls) = sidecar_ffi::display_manager_class() else {
        log::error!("SidecarDisplayManager class not found");
        return;
    };

    unsafe {
        let Some(manager) = sidecar_ffi::shared_manager(cls) else {
            log::error!("Could not get SidecarDisplayManager.sharedManager");
            return;
        };

        let Some(device) = find_device(&manager, ipad_name) else {
            log::debug!("No matching Sidecar device found for disconnect (may already be gone)");
            return;
        };

        log::info!("Disconnecting Sidecar...");
        sidecar_ffi::disconnect_from_device(&manager, &device);
    }
}

/// Find a Sidecar device by name, normalising Unicode quotes for matching.
fn find_device(manager: &AnyObject, target_name: Option<&str>) -> Option<Retained<AnyObject>> {
    unsafe {
        let array = sidecar_ffi::devices(manager)?;
        let count = sidecar_ffi::array_count(&array);
        let normalised_target = target_name.map(normalise_quotes);

        for i in 0..count {
            let Some(device) = sidecar_ffi::array_object_at(&array, i) else { continue };

            match normalised_target.as_deref() {
                Some(target) => {
                    if let Some(name) = sidecar_ffi::device_name(&device) {
                        if normalise_quotes(&name.to_string()) == target {
                            return Some(device);
                        }
                    }
                }
                None => return Some(device),
            }
        }

        None
    }
}

/// Replace common Unicode quote variants with plain ASCII apostrophe.
pub fn normalise_quotes(s: &str) -> String {
    s.replace('\u{2019}', "'")
     .replace('\u{2018}', "'")
     .replace('\u{02BC}', "'")
}

unsafe fn log_available_devices(manager: &AnyObject, target_name: Option<&str>) {
    unsafe {
        let Some(array) = sidecar_ffi::devices(manager) else {
            log::debug!("Could not read Sidecar devices list");
            return;
        };

        let count = sidecar_ffi::array_count(&array);
        if count == 0 {
            log::info!("Sidecar devices list is currently empty");
        } else {
            let mut names: Vec<String> = Vec::with_capacity(count);
            for i in 0..count {
                if let Some(device) = sidecar_ffi::array_object_at(&array, i) {
                    let name = sidecar_ffi::device_name(&device)
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "<unnamed>".into());
                    names.push(name);
                }
            }
            log::info!("Available Sidecar devices: {:?}", names);
            if let Some(target) = target_name {
                log::info!("Looking for: {:?} (check config if name doesn't match)", target);
            }
        }
    }
}
