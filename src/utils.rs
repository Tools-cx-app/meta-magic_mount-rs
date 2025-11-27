use std::{
    fs::{create_dir, create_dir_all, remove_dir, remove_dir_all, remove_file, write, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
};

use anyhow::{Context, Result, bail};
use rustix::mount::{mount, MountFlags};
#[cfg(any(target_os = "linux", target_os = "android"))]
use extattr::{Flags as XattrFlags, lsetxattr};

const SELINUX_XATTR: &str = "security.selinux";
const XATTR_TEST_FILE: &str = ".xattr_test";

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
            if io_err.kind() == std::io::ErrorKind::PermissionDenied {
                log::warn!("SELinux permission denied for {} (ignored)", path.as_ref().display());
            } else {
                log::warn!("Failed to set SELinux context for {}: {}", path.as_ref().display(), io_err);
            }
        }
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
    unimplemented!()
}

pub fn ensure_dir_exists<T: AsRef<Path>>(dir: T) -> Result<()> {
    if !dir.as_ref().exists() {
        create_dir_all(&dir)?;
    }
    Ok(())
}

// --- Smart Storage Utils ---

pub fn is_xattr_supported(path: &Path) -> bool {
    let test_file = path.join(XATTR_TEST_FILE);
    if let Err(_) = write(&test_file, b"test") {
        return false;
    }
    let supported = lsetfilecon(&test_file, "u:object_r:system_file:s0").is_ok();
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

pub fn sync_dir(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() { return Ok(()); }
    ensure_dir_exists(dst)?;

    let status = Command::new("cp")
        .arg("-af")
        .arg(format!("{}/.", src.display()))
        .arg(dst)
        .status()
        .context("Failed to execute cp command")?;

    if !status.success() {
        bail!("Failed to sync {} to {}", src.display(), dst.display());
    }

    let _ = Command::new("chcon")
        .arg("-R")
        .arg("u:object_r:system_file:s0")
        .arg(dst)
        .status();

    Ok(())
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

/// Selects a suitable temporary directory for mounting.
/// Iterates through common tmpfs locations to find a writable one.
pub fn select_temp_dir() -> Result<PathBuf> {
    // Candidates for the base directory.
    // We prioritize memory-backed filesystems (tmpfs/ramfs) for stealth and speed.
    // /data/adb/meta-hybrid is the fallback (persistent but writable).
    let candidates = [
        "/debug_ramdisk",
        "/sbin",
        "/dev",
        "/mnt",
        "/data/local/tmp",
        "/data/adb/meta-hybrid"
    ];

    for base in candidates {
        let path = Path::new(base);
        // Must exist and be a directory
        if !path.is_dir() {
            continue;
        }

        // Try to create a probe directory to verify writability
        let probe_dir = path.join(".mm_rw_probe");
        if create_dir(&probe_dir).is_ok() {
            // Cleanup and select this base
            let _ = remove_dir(&probe_dir);
            let work_dir = path.join("meta_hybrid_work");
            log::debug!("Selected temp dir base: {}", path.display());
            return Ok(work_dir);
        }
    }

    bail!("No writable temporary directory found! Checked: {:?}", candidates)
}
