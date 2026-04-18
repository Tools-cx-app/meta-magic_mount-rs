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

use std::{
    path::Path,
    sync::{LazyLock, Mutex, atomic::AtomicBool},
};

#[cfg(any(target_os = "linux", target_os = "android"))]
use anyhow::Result;
#[cfg(any(target_os = "linux", target_os = "android"))]
use ksu::{TryUmount, TryUmountFlags};

static KSU: AtomicBool = AtomicBool::new(false);

pub fn check_ksu() {
    let status = ksu::version().is_some_and(|v| {
        log::info!("KernelSU Version: {v}");
        true
    });

    KSU.store(status, std::sync::atomic::Ordering::Relaxed);
}

static FLAG: AtomicBool = AtomicBool::new(false);
static LIST: LazyLock<Mutex<TryUmount>> = LazyLock::new(|| Mutex::new(TryUmount::new()));

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

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn unmount() -> Result<()> {
    if KSU.load(std::sync::atomic::Ordering::Relaxed) {
        let mut control = LIST.lock().unwrap();

        control.flags(TryUmountFlags::MNT_DETACH);
        control.format_msg(|p| format!("umount {p:?} successful"));
        control.umount()?;
    }

    Ok(())
}
