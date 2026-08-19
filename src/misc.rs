// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use rustix::mount::{UnmountFlags, unmount};

use crate::{defs, errors::Result, mount_list, utils::ksucalls};

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

fn init_list() {
    super::parser::COMMAND_LIST
        .get_or_init(|| super::parser::parser_custom(defs::CUSTOM_LIST_PATH));
}

fn init_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |e| {
        hook(e);
        log::error!("panicked!!, err: {e}");
    }));
}

pub fn emulated_soft_reboot() -> Result<()> {
    mount_list::MountList::unmount_persisted()
}

pub fn cleanup() {
    if let Err(e) = unmount("/debug_ramdisk", UnmountFlags::DETACH) {
        log::warn!("failed to unmount tempdir: {e}");
    }
}

pub fn pre_init() {
    if std::env::var("KSU_LATE_LOAD").is_ok() {
        log::info!("late load mode!!");
    }

    init_logger();
    init_hook();
    ksucalls::check_ksu();
    init_list();
}
