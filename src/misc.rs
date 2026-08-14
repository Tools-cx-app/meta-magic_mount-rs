// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use rustix::mount::{UnmountFlags, unmount};

use crate::{errors::Result, utils::ksucalls};

fn init_logger() {
    #[cfg(not(target_os = "android"))]
    {
        use std::io::Write;

        let mut builder = env_logger::Builder::new();

        builder.format(|buf, record| {
            writeln!(
                buf,
                "[{}] [{}] {}",
                record.level(),
                record.target(),
                record.args()
            )
        });
        builder.filter_level(log::LevelFilter::Debug).init();
    }

    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("MagicMount"),
        );
    }
}

fn init_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |e| {
        hook(e);
        log::error!("panicked!!, err: {e}");
    }));
}

pub fn emulated_soft_reboot(source: &str) -> Result<()> {
    for mount in procfs::process::Process::myself()?.mountinfo()? {
        if mount.mount_source.is_some_and(|s| s == source) {
            log::debug!("unmounting {source} during emulated soft reboot");
            unmount(mount.mount_point, UnmountFlags::DETACH)?;
        }
    }
    Ok(())
}

pub fn cleanup() {
    if let Err(e) = unmount("/debug_ramdisk", UnmountFlags::DETACH) {
        log::warn!("failed to unmount tempdir: {e}");
    }
    if let Err(e) = std::fs::remove_dir("/debug_ramdisk") {
        log::warn!("failed to remove tempdir: {e}");
    }
}

pub fn pre_init() {
    init_logger();
    if std::env::var("KSU_LATE_LOAD").is_ok() {
        log::info!("late load mode!!");
    }

    init_hook();
    ksucalls::check_ksu();
}
