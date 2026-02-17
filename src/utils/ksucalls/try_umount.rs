// Copyright 2025 Magic Mount-rs Authors
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    path::Path,
    sync::{LazyLock, Mutex, OnceLock, atomic::AtomicBool},
};

use ksu::TryUmount;


static LAST: AtomicBool = AtomicBool::new(false);
pub static TMPFS: OnceLock<String> = OnceLock::new();
pub static LIST: LazyLock<Mutex<TryUmount>> = LazyLock::new(|| Mutex::new(TryUmount::new()));

pub fn send_unmountable<P>(target: P)
where
    P: AsRef<Path>,
{
    if !super::KSU.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }

    if LAST.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }

    LIST.lock().unwrap().add(target);
}
