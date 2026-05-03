// Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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

    let result = magic_mount::magic_mount(
        &tempdir,
        Path::new(MODULE_PATH),
        &config.mountsource,
        &config.partitions,
        config.umount,
    );

    cleanup(tempdir);

    match result {
        Ok(()) => {
            log::info!("Magic Mount Completed Successfully");
            let result = bind_mount();

            match result {
                Ok(()) => {
                    log::info!("Bind mount Completed Successfully");
                    Ok(())
                }
                Err(e) => {
                    log::error!("Bind mount Failed");
                    let e = anyhow::Error::from(e);
                    for cause in e.chain() {
                        log::error!("{cause:#?}");
                    }
                    log::error!("{:#?}", e.backtrace());
                    Err(e.into())
                }
            }
        }
        Err(e) => {
            log::error!("Magic Mount Failed");
            log::error!("Dont run bind mount stage!!");
            let e = anyhow::Error::from(e);
            for cause in e.chain() {
                log::error!("{cause:#?}");
            }
            log::error!("{:#?}", e.backtrace());
            Err(e.into())
        }
    }
}
