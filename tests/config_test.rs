use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use sidecar_on_dock::config::Config;

fn cfg(dock_uid: &str, ipad_name: Option<&str>) -> Config {
    Config {
        dock_uid: dock_uid.into(),
        ipad_name: ipad_name.map(Into::into),
    }
}

fn tempdir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "sidecar-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

// --- dock_uid parsing ---

#[test]
fn parse_uid_with_0x_prefix() {
    assert_eq!(
        cfg("0x003DA86E85A8CB00", None).dock_uid_u64().unwrap(),
        0x003DA86E85A8CB00
    );
}

#[test]
fn parse_uid_with_0x_uppercase() {
    assert_eq!(
        cfg("0X003DA86E85A8CB00", None).dock_uid_u64().unwrap(),
        0x003DA86E85A8CB00
    );
}

#[test]
fn parse_uid_without_prefix() {
    assert_eq!(
        cfg("003DA86E85A8CB00", None).dock_uid_u64().unwrap(),
        0x003DA86E85A8CB00
    );
}

#[test]
fn parse_uid_with_whitespace() {
    assert_eq!(cfg("  0x00FF  ", None).dock_uid_u64().unwrap(), 0xFF);
}

#[test]
fn parse_uid_invalid() {
    assert!(cfg("not_hex", None).dock_uid_u64().is_err());
}

// --- load / save ---

#[test]
fn load_valid_json() {
    let dir = tempdir();
    let path = dir.join("config.json");
    let mut f = fs::File::create(&path).unwrap();
    writeln!(f, r#"{{"dock_uid": "0xABCD", "ipad_name": "My iPad"}}"#).unwrap();

    let c = Config::load(&path).unwrap();
    assert_eq!(c.dock_uid, "0xABCD");
    assert_eq!(c.ipad_name.as_deref(), Some("My iPad"));
}

#[test]
fn load_minimal_json() {
    let dir = tempdir();
    let path = dir.join("config.json");
    fs::write(&path, r#"{"dock_uid": "0xFF"}"#).unwrap();

    let c = Config::load(&path).unwrap();
    assert_eq!(c.dock_uid, "0xFF");
    assert!(c.ipad_name.is_none());
}

#[test]
fn load_missing_file() {
    assert!(Config::load(Path::new("/nonexistent/config.json")).is_err());
}

#[test]
fn load_invalid_json() {
    let dir = tempdir();
    let path = dir.join("config.json");
    fs::write(&path, "not json").unwrap();
    assert!(Config::load(&path).is_err());
}

#[test]
fn save_and_reload() {
    let dir = tempdir();
    let path = dir.join("sub").join("config.json");

    let original = cfg("0xDEAD", Some("Test iPad"));
    original.save(&path).unwrap();

    let loaded = Config::load(&path).unwrap();
    assert_eq!(loaded.dock_uid, "0xDEAD");
    assert_eq!(loaded.ipad_name.as_deref(), Some("Test iPad"));
}
