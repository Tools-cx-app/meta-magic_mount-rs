// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{
    ffi::CString,
    fs, io,
    os::{fd::RawFd, unix::ffi::OsStrExt},
    path::{Path, PathBuf},
    sync::{
        OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::Context;
use parking_lot::{Mutex, const_mutex};

use crate::errors::Result;

pub static KSU: AtomicBool = AtomicBool::new(false);
static DRIVER_FD: OnceLock<RawFd> = OnceLock::new();
static LIST: Mutex<Vec<PathBuf>> = const_mutex(Vec::new());

const GET_INFO: u32 = libc::_IOR::<GetInfoCmd>(b'K' as u32, 2) as u32;
// These UAPI requests encode size 0 despite accepting command buffers.
const GET_INFO_LEGACY: u32 = libc::_IOR::<()>(b'K' as u32, 2) as u32;
const MANAGE_TRY_UMOUNT: u32 = libc::_IOW::<()>(b'K' as u32, 18) as u32;

#[repr(C)]
#[derive(Default)]
struct GetInfoCmd {
    version: u32,
    flags: u32,
    features: u32,
    uapi_version: u32,
}

// The UAPI uses __aligned_u64, including on 32-bit targets.
#[repr(C, align(8))]
struct TryUmountCmd {
    arg: u64,
    flags: u32,
    mode: u8,
}

fn scan_driver_fd(dir: &Path) -> io::Result<Option<RawFd>> {
    let mut driver = None;
    for entry in fs::read_dir(dir)?.flatten() {
        let Ok(fd) = entry.file_name().to_string_lossy().parse::<RawFd>() else {
            continue;
        };
        let Ok(target) = fs::read_link(entry.path()) else {
            continue;
        };
        if target == Path::new("anon_inode:[ksu_driver_su]") {
            return Ok(Some(fd));
        }
        if target == Path::new("anon_inode:[ksu_driver]") {
            driver = Some(fd);
        }
    }
    Ok(driver)
}

fn driver_fd() -> RawFd {
    *DRIVER_FD.get_or_init(|| {
        if let Some(fd) = scan_driver_fd(Path::new("/proc/self/fd")).ok().flatten() {
            return fd;
        }
        let mut fd: RawFd = -1;
        // SAFETY: ReSukiSU's reboot hook writes an fd to the valid output pointer.
        unsafe {
            libc::syscall(
                libc::SYS_reboot,
                0xDEADBEEF_u32,
                0xCAFEBABE_u32,
                0,
                &raw mut fd,
            );
        }
        fd
    })
}

fn ksuctl<T>(request: u32, cmd: &mut T) -> io::Result<()> {
    let fd = driver_fd();
    if fd < 0 {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "KernelSU driver fd unavailable",
        ));
    }
    // SAFETY: Private callers supply the matching repr(C) UAPI buffer, which
    // remains valid for the entire synchronous ioctl call.
    let ret = unsafe { libc::ioctl(fd, request as _, cmd as *mut T) };
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn get_info_with(
    mut call: impl FnMut(u32, &mut GetInfoCmd) -> io::Result<()>,
) -> io::Result<GetInfoCmd> {
    let mut cmd = GetInfoCmd::default();
    if call(GET_INFO, &mut cmd).is_err() {
        cmd = GetInfoCmd::default();
        call(GET_INFO_LEGACY, &mut cmd)?;
    }
    Ok(cmd)
}

fn add_umount_with(
    target: &Path,
    call: impl FnOnce(&mut TryUmountCmd) -> io::Result<()>,
) -> anyhow::Result<()> {
    (|| -> anyhow::Result<()> {
        let bytes = target.as_os_str().as_bytes();
        anyhow::ensure!(
            !bytes.is_empty() && bytes.len() <= 255,
            "mount path must be 1–255 bytes"
        );
        let path = CString::new(bytes)?;
        let mut cmd = TryUmountCmd {
            arg: path.as_ptr() as u64,
            flags: libc::MNT_DETACH as u32,
            mode: 1,
        };
        match call(&mut cmd) {
            Err(error) if error.raw_os_error() == Some(libc::EEXIST) => Ok(()),
            result => Ok(result?),
        }
    })()
    .with_context(|| {
        format!(
            "Failed to register KernelSU unmount path {}",
            target.display()
        )
    })
}

pub fn check_ksu() {
    let status = match get_info_with(ksuctl) {
        Ok(info) => {
            log::info!("KernelSU Version: {}", info.version);
            true
        }
        Err(error) => {
            log::debug!("KernelSU detection failed: {error}");
            false
        }
    };
    KSU.store(status, Ordering::Relaxed);
}

fn check_late_load_with(call: impl FnMut(u32, &mut GetInfoCmd) -> io::Result<()>) -> bool {
    get_info_with(call).is_ok_and(|info| info.flags & (1 << 2) != 0)
}

pub fn check_late_load() -> bool {
    check_late_load_with(ksuctl)
}

pub fn send_unmountable<P>(target: P)
where
    P: AsRef<Path>,
{
    if !KSU.load(Ordering::Relaxed) {
        return;
    }

    LIST.lock().push(target.as_ref().to_path_buf());
}

pub fn unmount() -> Result<()> {
    if KSU.load(Ordering::Relaxed) {
        let mut control = LIST.lock();
        for target in control.iter() {
            add_umount_with(target, |cmd| ksuctl(MANAGE_TRY_UMOUNT, cmd))?;
            log::debug!("registered KernelSU unmount path {}", target.display());
        }
        control.clear();
    }

    Ok(())
}

#[cfg(test)]
#[path = "../tests/unit/ksucalls.rs"]
mod tests;
