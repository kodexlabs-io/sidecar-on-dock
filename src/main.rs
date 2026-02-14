use sidecar_on_dock::{config, discovery, dock_monitor, launchd};

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "Monitor a Thunderbolt dock and automatically manage Sidecar display extension.")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// List connected Thunderbolt devices and available Sidecar (iPad) devices.
    Discover,
    /// Run the daemon (default when no subcommand is given).
    Run {
        /// Path to the JSON config file.
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// Print the default config file path.
    ConfigPath,
    /// Install a launchd agent so the daemon starts automatically on login.
    Install,
    /// Uninstall the launchd agent.
    Uninstall,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Command::Discover) => cmd_discover(),
        Some(Command::Run { config }) => cmd_run(config),
        Some(Command::ConfigPath) => cmd_config_path(),
        Some(Command::Install) => cmd_install(),
        Some(Command::Uninstall) => cmd_uninstall(),
        None => cmd_run(None),
    }
}

fn cmd_discover() {
    if let Err(e) = discovery::print_discovery() {
        log::error!("{e}");
        std::process::exit(1);
    }
}

fn cmd_run(config_path: Option<PathBuf>) {
    let path = config_path.unwrap_or_else(config::Config::default_path);

    let cfg = match config::Config::load(&path) {
        Ok(c) => c,
        Err(e) => {
            log::error!("{e}");
            log::info!("Hint: run `sidecar-on-dock discover` to find your dock, then create {}", path.display());
            std::process::exit(1);
        }
    };

    let dock_uid = match cfg.dock_uid_u64() {
        Ok(u) => u,
        Err(e) => {
            log::error!("{e}");
            std::process::exit(1);
        }
    };

    log::info!(
        "Config loaded. Dock UID: 0x{:016X}, iPad: {}",
        dock_uid,
        cfg.ipad_name.as_deref().unwrap_or("(first available)")
    );

    dock_monitor::run(dock_uid, cfg.ipad_name);
}

fn cmd_config_path() {
    println!("{}", config::Config::default_path().display());
}

fn cmd_install() {
    if let Err(e) = launchd::install() {
        log::error!("{e}");
        std::process::exit(1);
    }
}

fn cmd_uninstall() {
    if let Err(e) = launchd::uninstall() {
        log::error!("{e}");
        std::process::exit(1);
    }
}
