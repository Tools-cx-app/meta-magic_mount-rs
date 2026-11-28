// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::path::Path;

use rustix::mount::{MountFlags, UnmountFlags, mount, unmount};

use super::config::Config;
use super::error::{Error, Result};
use super::extension::{BindPhase, Extensions};
use super::mount::{self, MountList, bind_mount};
use super::paths;
use super::scanner::{self, AppModule};
use crate::platform::ksu;

pub(crate) struct Runtime {
    config: Config,
    extra_mount: Vec<String>,
    modules: Vec<AppModule>,
}

impl Runtime {
    pub(crate) fn load() -> Result<Self> {
        pre_init();
        let config = Config::load(paths::CONFIG_FILE)?;
        let extra_mount = config.extra_mount_partitions(Path::new("/"));
        let modules = scanner::scan_modules(paths::MODULE_PATH);
        Ok(Self {
            config,
            extra_mount,
            modules,
        })
    }

    pub(crate) fn modules(self) -> Result<Vec<AppModule>> {
        scanner::show_modules(self.modules, Path::new(paths::SCANNED_LIST))
    }

    pub(crate) fn mount(self) -> Result<()> {
        let Self {
            config,
            extra_mount,
            mut modules,
        } = self;
        let hooks = Extensions::load(
            Path::new(paths::CUSTOM_SCRIPT_PATH),
            Path::new(paths::MODULE_PATH),
        )?;
        let before_commands = hooks.bind_mounts(BindPhase::Before)?;
        let root =
            mount::collect_module_files(&mut modules, &config.partitions, &extra_mount, &hooks)?;

        let _ = std::fs::write(
            paths::SCANNED_LIST,
            &serde_json::to_string_pretty(&modules)?,
        );

        log::info!("Magic Mount Starting");
        log::info!("config info:\n{config}");

        log::debug!(
            "current selinux: {}",
            std::fs::read_to_string("/proc/self/attr/current")?
        );

        let mounts = MountList::persistent()?;

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

        let magic_mount_result = bind_mount(&before_commands, config.umount, &mounts)
            .and_then(|()| mount::magic_mount(root, &config.mountsource, config.umount, &mounts));
        let bind_mount_result = if magic_mount_result.is_ok() {
            Some((|| -> Result<()> {
                let commands = hooks.bind_mounts(BindPhase::After)?;
                bind_mount(&commands, config.umount, &mounts)
            })())
        } else {
            None
        };

        cleanup();
        ksu::unmount()?;

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
                return Err(Error::AnyHow(e));
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
                    return Err(Error::AnyHow(e));
                }
            }
        }

        Ok(())
    }
}

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

fn cleanup() {
    if let Err(e) = unmount("/debug_ramdisk", UnmountFlags::DETACH) {
        log::warn!("failed to unmount tempdir: {e}");
    }
}

fn pre_init() {
    init_logger();
    init_hook();
    ksu::check_ksu();

    if ksu::check_late_load() || std::env::var("KSU_LATE_LOAD").is_ok() {
        log::info!("late load mode!!");
    }
}
