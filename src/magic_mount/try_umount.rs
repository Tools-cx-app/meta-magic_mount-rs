#![allow(clippy::unreadable_literal)]

#[cfg(any(target_os = "linux", target_os = "android"))]
use std::{os::fd::RawFd, path::Path, sync::OnceLock};

#[cfg(any(target_os = "linux", target_os = "android"))]
use anyhow::Result;

const KSU_INSTALL_MAGIC1: u32 = 0xDEADBEEF;
const KSU_IOCTL_ADD_TRY_UMOUNT: u32 = 0x40004b12;
const KSU_INSTALL_MAGIC2: u32 = 0xCAFEBABE;
#[cfg(any(target_os = "linux", target_os = "android"))]
static DRIVER_FD: OnceLock<RawFd> = OnceLock::new();

#[repr(C)]
struct KsuAddTryUmount {
    arg: u64,
    flags: u32,
    mode: u8,
}

fn grab_fd() -> i32 {
    let mut fd = -1;
    unsafe {
        libc::syscall(
            libc::SYS_reboot,
            KSU_INSTALL_MAGIC1,
            KSU_INSTALL_MAGIC2,
            0,
            &mut fd,
        );
    };
    fd
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn send_unmountable<P>(target: P) -> Result<()>
where
    P: AsRef<Path>,
{
    use std::ffi::CString;

    use rustix::path::Arg;

    let path = CString::new(target.as_ref().as_str()?)?;
    let cmd = KsuAddTryUmount {
        arg: path.as_ptr() as u64,
        flags: 2,
        mode: 1,
    };
    #[allow(clippy::redundant_closure)]
    let fd = *DRIVER_FD.get_or_init(|| grab_fd());

    unsafe {
        #[cfg(target_env = "gnu")]
        let ret = libc::ioctl(fd as libc::c_int, KSU_IOCTL_ADD_TRY_UMOUNT as u64, &cmd);

        #[cfg(not(target_env = "gnu"))]
        let ret = libc::ioctl(fd as libc::c_int, KSU_IOCTL_ADD_TRY_UMOUNT as i32, &cmd);

        if ret < 0 {
            use std::io;

            log::error!(
                "umount {} failed: {}",
                target.as_ref().display(),
                io::Error::last_os_error()
            );

            return Err(anyhow::anyhow!("{}", io::Error::last_os_error()));
        }

        log::info!("umount {} successful!", target.as_ref().display());
    };

    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub fn send_unmountable() {
    unimplemented!()
}
