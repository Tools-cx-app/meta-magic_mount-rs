// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

mod framework;
mod platform;

#[cfg(test)]
#[path = "../tests/unit/cli.rs"]
mod cli_tests;

use clap::{Parser, Subcommand};
use framework::{Result, Runtime};

#[derive(Debug, Parser)]
#[command(
    about = "Magic Mount metamodule",
    after_help = "With no subcommand, perform module mounts."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, PartialEq, Subcommand)]
enum Commands {
    /// Show the current configuration as JSON
    ShowConfig,
    /// Unmount persisted mounts for an emulated soft reboot
    EmulatedSoftReboot,
    /// Save a hex-encoded JSON configuration
    SaveConfig {
        #[arg(long, value_name = "HEX")]
        payload: String,
    },
    /// Generate the default configuration
    GenConfig,
    /// List modules as JSON
    Modules,
    /// Show the version as JSON
    Version,
}

fn main() -> Result<()> {
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    compile_error!("unsupported platform!");

    let cli = Cli::parse();

    let runtime = Runtime::load()?;
    match cli.command {
        None => runtime.mount(),
        Some(Commands::ShowConfig) => framework::handle_show_config(),
        Some(Commands::EmulatedSoftReboot) => framework::emulated_soft_reboot(),
        Some(Commands::SaveConfig { payload }) => framework::handle_save_config(&payload),
        Some(Commands::GenConfig) => framework::handle_gen_config(),
        Some(Commands::Modules) => {
            println!("{}", serde_json::to_string_pretty(&runtime.modules()?)?);
            Ok(())
        }
        Some(Commands::Version) => {
            println!("{{ \"version\": \"{}\" }}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}
