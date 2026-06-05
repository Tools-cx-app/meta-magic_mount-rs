// Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
// SPDX-License-Identifier: Apache-2.0

use std::{ffi::CString, path::Path};

use libloading::{Library, Symbol};
use rustix::mount::{UnmountFlags, unmount};

use crate::{defs, errors::Result, utils::ksucalls};

type SignFunc = unsafe extern "C" fn(*const i8, *const i8) -> i32;

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
    let lib = unsafe { Library::new(defs::LIBRARY)? };

    let verify_sign: Symbol<SignFunc> = unsafe { lib.get(b"VerifySign")? };

    let pub_key = CString::new(env!("PUB_KEY"))?;
    let path = CString::new(defs::SELF_MODULE_PATH)?;

    if unsafe { verify_sign(pub_key.as_ptr().cast::<i8>(), path.as_ptr().cast::<i8>()) } != 1 {
        log::error!("failed to verify sign");
        panic!("verify sign is broken!!");
    }

    Ok(())
}

fn init_list() {
    /*super::magic_mount::node::IGNORE_LIST.get_or_init(|| {
        fs::read_to_string(defs::IGNORE_LIST_PATH).map_or_else(
            |_| None,
            |f| {
                Some(
                    f.lines()
                        .filter(|s| !s.starts_with('#'))
                        .map(std::string::ToString::to_string)
                        .collect(),
                )
            },
        )
    });*/
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
    let _ = verify_module_safety();
    ksucalls::check_ksu();
    init_list();
}
