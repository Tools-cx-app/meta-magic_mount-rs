// meta-hybrid_mount/src/utils.rs
use std::{
    ffi::CString,
    fs::{self, create_dir_all, remove_dir_all, remove_file, write, OpenOptions},
    io::Write,
    os::unix::fs::symlink, // Removed unused PermissionsExt
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
    sync::OnceLock,
    os::fd::RawFd,
};

use anyhow::{Context, Result, bail};
use rustix::mount::{mount, MountFlags};

use crate::defs;

#[cfg(any(target_os = "linux", target_os = "android"))]
use extattr::{Flags as XattrFlags, lsetxattr};

const SELINUX_XATTR: &str = "security.selinux";
const XATTR_TEST_FILE: &str = ".xattr_test";
const DEFAULT_CONTEXT: &str = "u:object_r:system_file:s0";

// --- File Logger Implementation ---
struct FileLogger {
    file: Mutex<std::fs::File>,
}

impl log::Log for FileLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let mut file = self.file.lock().unwrap();
            let _ = writeln!(
                file,
                "[{}] [{}] {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {
        let _ = self.file.lock().unwrap().flush();
    }
}

pub fn init_logger(verbose: bool, log_path: &Path) -> Result<()> {
    let level = if verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    if let Some(parent) = log_path.parent() {
        create_dir_all(parent)?;
    }

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(log_path)
        .with_context(|| format!("Failed to open log file: {}", log_path.display()))?;

    let logger = Box::new(FileLogger {
        file: Mutex::new(file),
    });

    log::set_boxed_logger(logger)
        .map(|()| log::set_max_level(level))
        .map_err(|e| anyhow::anyhow!("Failed to set logger: {}", e))?;

    Ok(())
}

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
        log::debug!("Creating directory: {}", dir.as_ref().display());
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
    log::debug!("Process name disguised as: {}", name);
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
    log::debug!("XATTR Support on {}: {}", path.display(), supported);
    let _ = remove_file(test_file);
    supported
}

pub fn mount_tmpfs(target: &Path) -> Result<()> {
    ensure_dir_exists(target)?;
    log::debug!("Mounting tmpfs at {}", target.display());
    mount("tmpfs", target, "tmpfs", MountFlags::empty(), "mode=0755")
        .context("Failed to mount tmpfs")?;
    Ok(())
}

pub fn mount_image(image_path: &Path, target: &Path) -> Result<()> {
    ensure_dir_exists(target)?;
    log::debug!("Mounting image {} at {}", image_path.display(), target.display());
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
        // Initial context, will be fixed by repair_contexts later
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
            if dst_path.exists() {
                remove_file(&dst_path)?; 
            }
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
    log::debug!("Cleaning up temp dir: {}", temp_dir.display());
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
    log::debug!("Selected temp dir: {}", work_dir.display());
    Ok(work_dir)
}

pub fn get_kernel_release() -> Result<String> {
    let output = Command::new("uname").arg("-r").output()?;
    let release = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(release)
}

pub struct ScopedKptrRestrict {
    original: String,
}

impl ScopedKptrRestrict {
    pub fn new() -> Self {
        let path = "/proc/sys/kernel/kptr_restrict";
        let original = std::fs::read_to_string(path).unwrap_or_else(|_| "2".to_string()).trim().to_string();
        
        if let Err(e) = write(path, "0") {
            log::warn!("Failed to lower kptr_restrict: {}", e);
        } else {
            log::debug!("Temporarily lowered kptr_restrict to 0 (was {})", original);
        }
        
        Self { original }
    }
}

impl Drop for ScopedKptrRestrict {
    fn drop(&mut self) {
        let path = "/proc/sys/kernel/kptr_restrict";
        if let Err(e) = write(path, &self.original) {
            log::warn!("Failed to restore kptr_restrict: {}", e);
        } else {
            log::debug!("Restored kptr_restrict to {}", self.original);
        }
    }
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
    // Fixed: as_str() is incorrect for Path, changed to to_str()
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
