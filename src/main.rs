// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]

mod bind_mount;
mod config;
mod defs;
mod errors;
mod magic_mount;
mod misc;
mod parser;
mod scanner;
mod utils;

use std::path::Path;

use rustix::mount::{MountFlags, mount};

use crate::{
    bind_mount::bind_mount,
    config::{Config, handle_gen_config, handle_save_config, handle_show_config},
    defs::MODULE_PATH,
    errors::Result,
    misc::cleanup,
    utils::ksucalls::unmount,
};

fn main() -> Result<()> {
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    compile_error!("unsupported platform!");

    misc::pre_init();

    let args: Vec<_> = std::env::args().collect();

    if let Some(arg) = args.get(1) {
        match arg.as_str() {
            "show-config" => {
                handle_show_config()?;
            }
            "save-config" => {
                handle_save_config(&args[2..])?;
            }
            "gen-config" => {
                handle_gen_config()?;
            }
            "modules" => {
                let config = Config::load_or_default(defs::CONFIG_FILE);
                let modules = scanner::list_modules(MODULE_PATH, &config.partitions);
                println!("{}", serde_json::to_string(&modules)?);
            }
            "version" => {
                println!("{{ \"version\": \"{}\" }}", env!("CARGO_PKG_VERSION"));
            }
            _ => {}
        }

        return Ok(());
    }

    let config = Config::load(defs::CONFIG_FILE)?;

    log::info!("Magic Mount Starting");
    log::info!("config info:\n{config}");

    log::debug!(
        "current selinux: {}",
        std::fs::read_to_string("/proc/self/attr/current")?
    );

    let tempdir = utils::generate_tmp();

    utils::ensure_dir_exists(&tempdir)?;

    if let Err(e) = mount(
        &config.mountsource,
        &tempdir,
        "tmpfs",
        MountFlags::empty(),
        None,
    ) {
        log::error!("mount tmpfs failed: {e}");
        std::process::exit(1);
    }

    let magic_mount_result = magic_mount::magic_mount(
        &tempdir,
        Path::new(MODULE_PATH),
        &config.mountsource,
        &config.partitions,
        config.umount,
    );
    let bind_mount_result = if magic_mount_result.is_ok() {
        Some(bind_mount(config.umount))
    } else {
        None
    };

    cleanup(tempdir);
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
            }
        }
    }

    Ok(())
}
