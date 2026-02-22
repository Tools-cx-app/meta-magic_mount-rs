// Copyright 2025 Magic Mount-rs Authors
// SPDX-License-Identifier: GPL-3.0-or-later
#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]

mod config;
mod defs;
mod magic_mount;
mod scanner;
mod utils;

use std::path::Path;

use anyhow::Result;
use mimalloc::MiMalloc;
use rustix::{
    mount::{MountFlags, mount},
    path::Arg,
};

use crate::{config::Config, defs::MODULE_PATH};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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

fn main() -> Result<()> {
    let config = Config::load()?;

    let args: Vec<_> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "scan" => {
                let modules = scanner::scan_modules(MODULE_PATH, &config.partitions);

                if let Some(s) = args.get(2)
                    && s.as_str() == "--json"
                {
                    let json = serde_json::to_string(&modules)?;
                    println!("{json}");
                } else {
                    for module in modules {
                        println!("{}", module.id);
                    }
                }
                return Ok(());
            }
            "version" => {
                println!("{{ \"version\": \"{}\" }}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            _ => {}
        }
    }

    init_logger();

    utils::ksucalls::check_ksu();

    log::info!("Magic Mount Starting");
    log::info!("config info:\n{config}");

    log::debug!(
        "current selinux: {}",
        std::fs::read_to_string("/proc/self/attr/current")?
    );

    let tempdir = utils::generate_tmp();

    let _ = utils::ksucalls::try_umount::TMPFS.set(tempdir.as_str()?.to_string());

    utils::ensure_dir_exists(&tempdir)?;

    if let Err(e) = mount(
        &config.mountsource,
        &tempdir,
        "tmpfs",
        MountFlags::empty(),
        None,
    ) {
        panic!("mount tmpfs failed: {e}");
    }

    let result = magic_mount::magic_mount(
        &tempdir,
        Path::new(MODULE_PATH),
        &config.mountsource,
        &config.partitions,
        #[cfg(any(target_os = "linux", target_os = "android"))]
        config.umount,
    );

    match result {
        Ok(()) => {
            log::info!("Magic Mount Completed Successfully");
            Ok(())
        }
        Err(e) => {
            log::error!("Magic Mount Failed");
            for cause in e.chain() {
                log::error!("{cause:#?}");
            }
            log::error!("{:#?}", e.backtrace());
            Err(e)
        }
    }
}
