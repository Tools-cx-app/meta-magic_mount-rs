mod kernel;

use std::{
    fs::{self, read_dir},
    path::Path,
    sync::{OnceLock, atomic::AtomicBool},
};

use anyhow::Result;

use crate::defs::{DISABLE_FILE_NAME, REMOVE_FILE_NAME, SKIP_MOUNT_FILE_NAME};

static LAST: AtomicBool = AtomicBool::new(false);
pub static TMPFS: OnceLock<String> = OnceLock::new();

pub fn send_unmountable<P>(target: P) -> Result<()>
where
    P: AsRef<Path>,
{
    if LAST.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(());
    }

    for entry in read_dir("/data/adb/modules")?.flatten() {
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        if !path.join("module.prop").exists() {
            continue;
        }

        let disabled =
            path.join(DISABLE_FILE_NAME).exists() || path.join(REMOVE_FILE_NAME).exists();
        let skip = path.join(SKIP_MOUNT_FILE_NAME).exists();
        if disabled || skip {
            continue;
        }

        if !path.ends_with("zygisksu") {
            continue;
        }

        if fs::read_to_string("/data/adb/zygisksu/denylist_enforce")?.trim() != "0"
            && TMPFS.get().is_some_and(|s| s.trim() == "/debug_ramdisk")
        {
            log::warn!("zn was detected, and try_umount was cancelled.");
            LAST.store(true, std::sync::atomic::Ordering::Relaxed);
            return Ok(());
        }
    }

    kernel::send_kernel_umount(target.as_ref())?;
    Ok(())
}
