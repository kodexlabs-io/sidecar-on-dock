//! Manage a launchd plist so the daemon auto-starts on login.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

const LABEL: &str = "com.sidecar-on-dock.daemon";

/// Path to the launchd plist file.
pub fn plist_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home)
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{LABEL}.plist"))
}

/// Install the launchd plist and load it.
pub fn install() -> Result<(), String> {
    let binary = std::env::current_exe()
        .map_err(|e| format!("Cannot determine current executable path: {e}"))?;

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LABEL}</string>

    <key>ProgramArguments</key>
    <array>
        <string>{binary}</string>
        <string>run</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>StandardOutPath</key>
    <string>/tmp/sidecar-on-dock.stdout.log</string>

    <key>StandardErrorPath</key>
    <string>/tmp/sidecar-on-dock.stderr.log</string>
</dict>
</plist>
"#,
        binary = binary.display(),
    );

    let path = plist_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create LaunchAgents dir: {e}"))?;
    }

    fs::write(&path, &plist)
        .map_err(|e| format!("Failed to write plist to {}: {e}", path.display()))?;

    let status = Command::new("launchctl")
        .args(["load", "-w"])
        .arg(&path)
        .status()
        .map_err(|e| format!("Failed to run launchctl: {e}"))?;

    if !status.success() {
        return Err(format!("launchctl load exited with {status}"));
    }

    println!("Installed and loaded: {}", path.display());
    println!("The daemon will now start automatically on login.");
    Ok(())
}

/// Unload and remove the launchd plist.
pub fn uninstall() -> Result<(), String> {
    let path = plist_path();

    if !path.exists() {
        println!("Nothing to uninstall (plist not found at {})", path.display());
        return Ok(());
    }

    let _ = Command::new("launchctl")
        .args(["unload", "-w"])
        .arg(&path)
        .status();

    fs::remove_file(&path)
        .map_err(|e| format!("Failed to remove plist: {e}"))?;

    println!("Uninstalled: {}", path.display());
    Ok(())
}
