// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]

mod bind_mount;
mod daemon;
mod defs;
mod errors;
mod magic_mount;
mod misc;
mod parser;
mod utils;

use rustix::mount::{MountFlags, mount};

use crate::{
    bind_mount::bind_mount,
    daemon::load_config,
    defs::{CONFIG_FILE, CUSTOM_LIST_PATH, MODULE_PATH},
    errors::Result,
    misc::{cleanup, emulated_soft_reboot},
    parser::init_from_config,
    utils::ksucalls::unmount,
};

fn main() -> Result<()> {
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    compile_error!("unsupported platform!");

    misc::pre_init();

    let config = load_config(
        std::path::Path::new(CONFIG_FILE),
        std::path::Path::new(CUSTOM_LIST_PATH),
    )?;
    init_from_config(&config)?;
    if let Some(command) = std::env::args().nth(1) {
        if command == "emulated-soft-reboot" {
            emulated_soft_reboot(&config.mountsource)?;
            return Ok(());
        }
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("unknown command: {command}"),
        )
        .into());
    }

    log::info!("Magic Mount Starting");
    log::info!("config mount source: {}", config.mountsource);

    log::debug!(
        "current selinux: {}",
        std::fs::read_to_string("/proc/self/attr/current")?
    );

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
        config.umount,
    );
    let bind_mount_result = if magic_mount_result.is_ok() {
        Some(bind_mount(config.umount))
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
