// Copyright 2025 Magic Mount-rs Authors
// SPDX-License-Identifier: GPL-3.0-or-later
#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]

mod config;
mod defs;
mod magic_mount;
mod misc;
mod scanner;
mod utils;

use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use mimalloc::MiMalloc;
use rustix::mount::{MountFlags, mount};
use serde_json::json;

use crate::{
    config::{ApiConfigPayload, Config},
    defs::MODULE_PATH,
};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn decode_hex(input: &str) -> Result<Vec<u8>> {
    if !input.len().is_multiple_of(2) {
        bail!("hex payload must contain an even number of characters");
    }

    let mut bytes = Vec::with_capacity(input.len() / 2);
    for chunk in input.as_bytes().chunks_exact(2) {
        let hex = std::str::from_utf8(chunk).context("hex payload is not valid utf-8")?;
        let byte = u8::from_str_radix(hex, 16)
            .with_context(|| format!("invalid hex byte '{hex}' in payload"))?;
        bytes.push(byte);
    }

    Ok(bytes)
}

fn parse_payload_arg(args: &[String]) -> Result<&str> {
    let payload = args
        .windows(2)
        .find_map(|window| (window[0] == "--payload").then_some(window[1].as_str()))
        .ok_or_else(|| anyhow!("missing required --payload argument"))?;

    Ok(payload)
}

fn handle_show_config() -> Result<()> {
    let config = Config::load_or_default();
    let ignore_list = Config::read_ignore_list()?;
    println!("{}", serde_json::to_string(&config.into_api(ignore_list))?);
    Ok(())
}

fn handle_save_config(args: &[String]) -> Result<()> {
    let payload_hex = parse_payload_arg(args)?;
    let payload_json = String::from_utf8(decode_hex(payload_hex)?)
        .context("decoded payload is not valid utf-8")?;
    let payload: ApiConfigPayload =
        serde_json::from_str(&payload_json).context("failed to parse config payload json")?;

    let ignore_list = payload.ignore_list.clone();
    let mut config = Config::load_or_default();
    config.apply_api_payload(payload);
    config.save()?;
    if let Some(ignore_list) = ignore_list {
        Config::write_ignore_list(&ignore_list)?;
    }

    println!("{}", json!({ "ok": true }));
    Ok(())
}

fn handle_gen_config() -> Result<()> {
    let config = Config::default();
    config.save()?;
    Config::write_ignore_list(&[])?;
    println!("{}", json!({ "ok": true }));
    Ok(())
}

fn main() -> Result<()> {
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
                let config = Config::load_or_default();
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

    let config = Config::load()?;

    utils::ksucalls::check_ksu();

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

    let cleanup = || {
        use rustix::mount::{UnmountFlags, unmount};
        if let Err(e) = unmount(&tempdir, UnmountFlags::DETACH) {
            log::warn!("failed to unmount tempdir: {e}");
        }
        if let Err(e) = std::fs::remove_dir(&tempdir) {
            log::warn!("failed to remove tempdir: {e}");
        }
    };

    match result {
        Ok(()) => {
            log::info!("Magic Mount Completed Successfully");
            cleanup();
            Ok(())
        }
        Err(e) => {
            log::error!("Magic Mount Failed");
            for cause in e.chain() {
                log::error!("{cause:#?}");
            }
            log::error!("{:#?}", e.backtrace());
            cleanup();
            Err(e)
        }
    }
}
