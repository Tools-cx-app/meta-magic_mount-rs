// src/utils.rs
use std::{
    ffi::CString,
    fs::{self, create_dir_all, remove_dir_all, remove_file, write},
    io::Write,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
    os::fd::RawFd,
};

use anyhow::{Context, Result, bail};
use rustix::mount::{mount, MountFlags};

// Tracing imports
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::non_blocking::WorkerGuard;

use crate::defs;

#[cfg(any(target_os = "linux", target_os = "android"))]
use extattr::{Flags as XattrFlags, lsetxattr};

const SELINUX_XATTR: &str = "security.selinux";
const XATTR_TEST_FILE: &str = ".xattr_test";
const DEFAULT_CONTEXT: &str = "u:object_r:system_file:s0";

// --- Advanced Logging System ---

/// Initializes the tracing logging system.
/// Returns a WorkerGuard that MUST be held by the main function to ensure logs are flushed.
pub fn init_logging(verbose: bool, log_path: &Path) -> Result<WorkerGuard> {
    // 1. Setup file appender (non-blocking for performance, but safe)
    if let Some(parent) = log_path.parent() {
        create_dir_all(parent)?;
    }
    
    let file_appender = tracing_appender::rolling::never(
        log_path.parent().unwrap(),
        log_path.file_name().unwrap()
    );
    
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // 2. Define filter levels
    let filter = if verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    // 3. Setup formatting layer for file
    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_writer(non_blocking);

    // 4. Initialize subscriber
    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .init();

    // 5. Redirect standard `log` macros to `tracing`
    // This allows us to keep using log::info! in other modules
    tracing_log::LogTracer::init().ok();

    // 6. Install Panic Hook
    // This captures panic info and writes it to the log file before crashing
    let log_path_buf = log_path.to_path_buf();
    std::panic::set_hook(Box::new(move |info| {
        let msg = match info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };
        
        let location = info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
        let error_msg = format!("\n[CRITICAL PANIC] Thread crashed at {}: {}\n", location, msg);
        
        // Use standard fs write to ensure it hits disk even if tracing channel is clogged
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&log_path_buf) {
            let _ = writeln!(file, "{}", error_msg);
        }
        
        eprintln!("{}", error_msg);
    }));

    Ok(guard)
}

// --- File System Utils ---

pub fn lsetfilecon<P: AsRef<Path>>(path: P, con: &str) -> Result<()> {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        if let Err(e) = lsetxattr(&path, SELINUX_XATTR, con, XattrFlags::empty()) {
            let io_err = std::io::Error::from(e);
            log::debug!("lsetfilecon: {} -> {} failed: {}", path.as_ref().display(), con, io_err);
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    {
        let _ = path;
        let _ = con;
    }
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn lgetfilecon<P: AsRef<Path>>(path: P) -> Result<String> {
    let con = extattr::lgetxattr(&path, SELINUX_XATTR).with_context(|| {
        format!("Failed to get SELinux context for {}", path.as_ref().display())
    })?;
    Ok(String::from_utf8_lossy(&con).to_string())
}

#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub fn lgetfilecon<P: AsRef<Path>>(_path: P) -> Result<String> {
    Ok(DEFAULT_CONTEXT.to_string())
}

pub fn copy_path_context<S: AsRef<Path>, D: AsRef<Path>>(src: S, dst: D) -> Result<()> {
    let context = if src.as_ref().exists() {
        lgetfilecon(&src).unwrap_or_else(|_| DEFAULT_CONTEXT.to_string())
    } else {
        DEFAULT_CONTEXT.to_string()
    };
    lsetfilecon(dst, &context)
}

pub fn ensure_dir_exists<T: AsRef<Path>>(dir: T) -> Result<()> {
    if !dir.as_ref().exists() {
        // log::debug might not be available yet if called before init, but that's fine
        create_dir_all(&dir)?;
    }
    Ok(())
}

// --- Stealth Utils (Process) ---

pub fn camouflage_process(name: &str) -> Result<()> {
    let c_name = CString::new(name)?;
    unsafe {
        libc::prctl(libc::PR_SET_NAME, c_name.as_ptr() as u64, 0, 0, 0);
    }
    Ok(())
}

// --- Smart Storage Utils ---

pub fn is_xattr_supported(path: &Path) -> bool {
    let test_file = path.join(XATTR_TEST_FILE);
    if let Err(e) = write(&test_file, b"test") {
        log::debug!("XATTR Check: Failed to create test file: {}", e);
        return false;
    }
    let result = lsetfilecon(&test_file, "u:object_r:system_file:s0");
    let supported = result.is_ok();
    let _ = remove_file(test_file);
    supported
}

pub fn mount_tmpfs(target: &Path) -> Result<()> {
    ensure_dir_exists(target)?;
    mount("tmpfs", target, "tmpfs", MountFlags::empty(), "mode=0755")
        .context("Failed to mount tmpfs")?;
    Ok(())
}

pub fn mount_image(image_path: &Path, target: &Path) -> Result<()> {
    ensure_dir_exists(target)?;
    let status = Command::new("mount")
        .args(["-t", "ext4", "-o", "loop,rw,noatime"])
        .arg(image_path)
        .arg(target)
        .status()
        .context("Failed to execute mount command")?;

    if !status.success() {
        bail!("Mount command failed");
    }
    Ok(())
}

fn native_cp_r(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        create_dir_all(dst)?;
        let src_meta = src.metadata()?;
        fs::set_permissions(dst, src_meta.permissions())?;
        lsetfilecon(dst, DEFAULT_CONTEXT)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ft.is_dir() {
            native_cp_r(&src_path, &dst_path)?;
        } else if ft.is_symlink() {
            let link_target = fs::read_link(&src_path)?;
            if dst_path.exists() { remove_file(&dst_path)?; }
            symlink(&link_target, &dst_path)?;
            let _ = lsetfilecon(&dst_path, DEFAULT_CONTEXT);
        } else {
            fs::copy(&src_path, &dst_path)?;
            let src_meta = src_path.metadata()?;
            fs::set_permissions(&dst_path, src_meta.permissions())?;
            lsetfilecon(&dst_path, DEFAULT_CONTEXT)?;
        }
    }
    Ok(())
}

pub fn sync_dir(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() { return Ok(()); }
    ensure_dir_exists(dst)?;
    native_cp_r(src, dst).with_context(|| {
        format!("Failed to natively sync {} to {}", src.display(), dst.display())
    })
}

pub fn cleanup_temp_dir(temp_dir: &Path) {
    if let Err(e) = remove_dir_all(temp_dir) {
        log::warn!("Failed to clean up temp dir {}: {:#}", temp_dir.display(), e);
    }
}

pub fn ensure_temp_dir(temp_dir: &Path) -> Result<()> {
    if temp_dir.exists() {
        remove_dir_all(temp_dir).ok();
    }
    create_dir_all(temp_dir)?;
    Ok(())
}

pub fn select_temp_dir() -> Result<PathBuf> {
    let run_dir = Path::new(defs::RUN_DIR);
    ensure_dir_exists(run_dir)?;
    let work_dir = run_dir.join("workdir");
    Ok(work_dir)
}

const KSU_INSTALL_MAGIC1: u32 = 0xDEADBEEF;
const KSU_INSTALL_MAGIC2: u32 = 0xCAFEBABE;
const KSU_IOCTL_NUKE_EXT4_SYSFS: u32 = 0x40004b11; 
const KSU_IOCTL_ADD_TRY_UMOUNT: u32 = 0x40004b12; 

static DRIVER_FD: OnceLock<RawFd> = OnceLock::new();

#[repr(C)]
struct KsuAddTryUmount {
    arg: u64,
    flags: u32,
    mode: u8,
}

#[repr(C)]
struct NukeExt4SysfsCmd {
    arg: u64,
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
    let path_ref = target.as_ref();
    let path_str = path_ref.to_str().unwrap_or_default(); 
    if path_str.is_empty() { return Ok(()); }

    let path = CString::new(path_str)?;
    let cmd = KsuAddTryUmount {
        arg: path.as_ptr() as u64,
        flags: 2,
        mode: 1,
    };
    let fd = *DRIVER_FD.get_or_init(grab_fd);
    if fd < 0 { return Ok(()); }

    unsafe {
        #[cfg(target_env = "gnu")]
        let _ = libc::ioctl(fd as libc::c_int, KSU_IOCTL_ADD_TRY_UMOUNT as u64, &cmd);
        #[cfg(not(target_env = "gnu"))]
        let _ = libc::ioctl(fd as libc::c_int, KSU_IOCTL_ADD_TRY_UMOUNT as i32, &cmd);
    };
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub fn send_unmountable<P>(_target: P) -> Result<()> {
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn ksu_nuke_sysfs(target: &str) -> Result<()> {
    let c_path = CString::new(target)?;
    let cmd = NukeExt4SysfsCmd {
        arg: c_path.as_ptr() as u64,
    };
    let fd = *DRIVER_FD.get_or_init(grab_fd);
    if fd < 0 {
        bail!("KSU driver not available");
    }
    let ret = unsafe {
        #[cfg(target_env = "gnu")]
        let r = libc::ioctl(fd as libc::c_int, KSU_IOCTL_NUKE_EXT4_SYSFS as u64, &cmd);
        #[cfg(not(target_env = "gnu"))]
        let r = libc::ioctl(fd as libc::c_int, KSU_IOCTL_NUKE_EXT4_SYSFS as i32, &cmd);
        r
    };
    if ret != 0 {
        bail!("ioctl failed with code {}", ret);
    }
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub fn ksu_nuke_sysfs(_target: &str) -> Result<()> {
    bail!("Not supported on this OS")
}
