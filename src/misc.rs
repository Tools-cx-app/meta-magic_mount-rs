// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: Apache-2.0

use std::{fs, io::Read, path::Path};

use rustix::mount::{UnmountFlags, unmount};

use crate::{defs, errors::Result, utils::ksucalls};

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

fn verify_module_safety() -> Result<()> {
    let mut buf = [0u8; ed25519_dalek::PUBLIC_KEY_LENGTH];
    let file = fs::OpenOptions::new().read(true).open(".key");
    if file.is_err() && cfg!(test) {
        return Ok(());
    }
    file?.read(&mut buf)?;

    let key = ed25519_dalek::VerifyingKey::from_bytes(&buf)?;
    if verification::verify(&key, defs::SELF_MODULE_PATH).is_err() {
        log::error!("verify is broken!!");
        panic!("broken!!");
    }
    Ok(())
}

fn init_list() {
    super::parser::COMMAND_LIST
        .get_or_init(|| super::parser::parser_custom(defs::CUSTOM_LIST_PATH));
}

pub fn cleanup<P>(tempdir: P)
where
    P: AsRef<Path>,
{
    if let Err(e) = unmount(
        tempdir.as_ref().to_string_lossy().to_string(),
        UnmountFlags::DETACH,
    ) {
        log::warn!("failed to unmount tempdir: {e}");
    }
    if let Err(e) = std::fs::remove_dir(&tempdir) {
        log::warn!("failed to remove tempdir: {e}");
    }
}

pub fn pre_init() {
    assert!(
        !(std::env::var("KSU_LATE_LOAD").is_ok() && std::env::var("KSU").is_ok()),
        "! unsupported late load mode"
    );

    init_logger();
    #[cfg(not(test))]
    let _ = verify_module_safety();
    ksucalls::check_ksu();
    init_list();
}
