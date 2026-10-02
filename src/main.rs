// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

mod bind_mount;
mod config;
mod defs;
mod errors;
mod ksucalls;
mod magic_mount;
mod misc;
mod mount_list;
mod parser;
mod scanner;
mod utils;

#[cfg(test)]
#[path = "../tests/unit/cli.rs"]
mod cli_tests;

use clap::{Parser, Subcommand};
use rustix::mount::{MountFlags, mount};

use crate::{
    bind_mount::bind_mount,
    config::{Config, handle_gen_config, handle_save_config, handle_show_config},
    defs::MODULE_PATH,
    errors::Result,
    ksucalls::unmount,
    misc::{cleanup, emulated_soft_reboot},
};

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

    misc::pre_init();

    let config = Config::load(defs::CONFIG_FILE)?;
    let extra_mount = config.extra_mount_partitions(std::path::Path::new("/"));
    let mut scan_partitions = config.partitions.clone();
    scan_partitions.extend(extra_mount.iter().cloned());
    let modules = scanner::list_modules(MODULE_PATH, &scan_partitions);

    if let Some(command) = cli.command {
        match command {
            Commands::ShowConfig => {
                handle_show_config()?;
            }
            Commands::EmulatedSoftReboot => {
                emulated_soft_reboot()?;
            }
            Commands::SaveConfig { payload } => {
                handle_save_config(&payload)?;
            }
            Commands::GenConfig => {
                handle_gen_config()?;
            }
            Commands::Modules => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&scanner::show_modules(modules)?)?
                );
            }
            Commands::Version => {
                println!("{{ \"version\": \"{}\" }}", env!("CARGO_PKG_VERSION"));
            }
        }

        return Ok(());
    }

    let _ = std::fs::write(defs::SCANNED_LIST, &serde_json::to_string_pretty(&modules)?);

    log::info!("Magic Mount Starting");
    log::info!("config info:\n{config}");

    log::debug!(
        "current selinux: {}",
        std::fs::read_to_string("/proc/self/attr/current")?
    );

    let mounts = mount_list::MountList::persistent()?;

    if let Err(e) = mount(
        &config.mountsource,
        "/debug_ramdisk",
        "tmpfs",
        MountFlags::empty(),
        None,
    ) {
        log::error!("mount tmpfs failed: {e}");
        std::process::exit(1);
    }

    let magic_mount_result = magic_mount::magic_mount(
        MODULE_PATH,
        &config.mountsource,
        &config.partitions,
        &extra_mount,
        config.umount,
        &mounts,
    );
    let bind_mount_result = if magic_mount_result.is_ok() {
        Some(bind_mount(config.umount, &mounts))
    } else {
        None
    };

    cleanup();
    unmount()?;

    match magic_mount_result {
        Ok(()) => {
            log::info!("Magic Mount Completed Successfully");
        }
        Err(e) => {
            log::error!("Magic Mount Failed");
            log::error!("Dont run bind mount stage!!");
            let e = anyhow::Error::from(e);
            for cause in e.chain() {
                log::error!("{cause:#?}");
            }
            log::error!("{:#?}", e.backtrace());
            return Err(errors::Error::AnyHow(e));
        }
    }

    if let Some(bind_mount_result) = bind_mount_result {
        match bind_mount_result {
            Ok(()) => {
                log::info!("Bind mount Completed Successfully");
            }
            Err(e) => {
                log::error!("Bind mount Failed");
                let e = anyhow::Error::from(e);
                for cause in e.chain() {
                    log::error!("{cause:#?}");
                }
                log::error!("{:#?}", e.backtrace());
                return Err(errors::Error::AnyHow(e));
            }
        }
    }

    Ok(())
}
