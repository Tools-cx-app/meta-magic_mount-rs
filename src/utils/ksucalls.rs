// Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
// SPDX-License-Identifier: Apache-2.0

use std::{
    path::Path,
    sync::{LazyLock, Mutex, atomic::AtomicBool},
};

use ksu::{TryUmount, TryUmountFlags};

use crate::errors::Result;

pub static KSU: AtomicBool = AtomicBool::new(false);
static FLAG: AtomicBool = AtomicBool::new(false);
static LIST: LazyLock<Mutex<TryUmount>> = LazyLock::new(|| Mutex::new(TryUmount::new()));

pub fn check_ksu() {
    let status = ksu::version().is_some_and(|v| {
        log::info!("KernelSU Version: {v}");
        if v.to_string().starts_with('4') {
            log::warn!(
                "The ioctl function of SukiSU-Ultra has been broken, and umount is now disabled."
            );
            FLAG.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        true
    });

    KSU.store(status, std::sync::atomic::Ordering::Relaxed);
}

pub fn send_unmountable<P>(target: P)
where
    P: AsRef<Path>,
{
    if !KSU.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }

    if FLAG.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }

    LIST.lock().unwrap().add(target);
}

pub fn unmount() -> Result<()> {
    if KSU.load(std::sync::atomic::Ordering::Relaxed) {
        let mut control = LIST.lock().unwrap();

        control.flags(TryUmountFlags::MNT_DETACH);
        control.format_msg(|p| format!("umount {p:?} successful"));
        control.umount()?;
    }

    Ok(())
}
