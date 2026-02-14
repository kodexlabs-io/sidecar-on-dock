//! JSON configuration file (`~/.config/sidecar-on-dock/config.json`).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Runtime configuration loaded from a JSON file.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Thunderbolt dock UID as a hex string, e.g. `"0x003DA86E85A8CB00"`.
    pub dock_uid: String,
    /// Optional iPad name to target. If `None`, the first available Sidecar device is used.
    pub ipad_name: Option<String>,
}

impl Config {
    /// Default config file path.
    pub fn default_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        PathBuf::from(home)
            .join(".config")
            .join("sidecar-on-dock")
            .join("config.json")
    }

    /// Load configuration from a JSON file.
    pub fn load(path: &Path) -> Result<Self, String> {
        let data = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config at {}: {e}", path.display()))?;
        serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse config at {}: {e}", path.display()))
    }

    /// Save configuration to a JSON file (pretty-printed).
    #[allow(dead_code)]
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {e}"))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialise config: {e}"))?;
        fs::write(path, json)
            .map_err(|e| format!("Failed to write config to {}: {e}", path.display()))
    }

    /// Parse `dock_uid` from its hex string representation to a `u64`.
    pub fn dock_uid_u64(&self) -> Result<u64, String> {
        let s = self.dock_uid.trim().trim_start_matches("0x").trim_start_matches("0X");
        u64::from_str_radix(s, 16)
            .map_err(|e| format!("Invalid dock_uid '{}': {e}", self.dock_uid))
    }
}
