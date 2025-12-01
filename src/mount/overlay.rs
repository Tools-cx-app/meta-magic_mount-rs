
use anyhow::{Context, Result};
use log::{info, warn};
use std::path::{Path, PathBuf};

use procfs::process::Process;
use rustix::{fd::AsFd, fs::CWD, mount::*};

use crate::defs::KSU_OVERLAY_SOURCE;
use crate::utils::send_unmountable;

/// Low-level function to mount overlayfs using modern fsopen API or fallback to mount()
pub fn mount_overlayfs(
    lower_dirs: &[String],
    lowest: &str,
    upperdir: Option<PathBuf>,
    workdir: Option<PathBuf>,
    dest: impl AsRef<Path>,
    disable_umount: bool,
) -> Result<()> {
    let lowerdir_config = lower_dirs
        .iter()
        .map(|s| s.as_ref())
        .chain(std::iter::once(lowest))
        .collect::<Vec<_>>()
        .join(":");
    
    info!(
        "mount overlayfs on {:?}, lowerdir={}, upperdir={:?}, workdir={:?}",
        dest.as_ref(),
        lowerdir_config,
        upperdir,
        workdir
    );

    let upperdir = upperdir
        .filter(|up| up.exists())
        .map(|e| e.display().to_string());
    let workdir = workdir
        .filter(|wd| wd.exists())
        .map(|e| e.display().to_string());

    let result = (|| {
        let fs = fsopen("overlay", FsOpenFlags::FSOPEN_CLOEXEC)?;
        let fs = fs.as_fd();
        fsconfig_set_string(fs, "lowerdir", &lowerdir_config)?;
        if let (Some(upperdir), Some(workdir)) = (&upperdir, &workdir) {
            fsconfig_set_string(fs, "upperdir", upperdir)?;
            fsconfig_set_string(fs, "workdir", workdir)?;
        }
        fsconfig_set_string(fs, "source", KSU_OVERLAY_SOURCE)?;
        fsconfig_create(fs)?;
        let mount = fsmount(fs, FsMountFlags::FSMOUNT_CLOEXEC, MountAttrFlags::empty())?;
        move_mount(
            mount.as_fd(),
            "",
            CWD,
            dest.as_ref(),
            MoveMountFlags::MOVE_MOUNT_F_EMPTY_PATH,
        )
    })();

    if let Err(e) = result {
        warn!("fsopen mount failed: {e:#}, fallback to mount");
        let mut data = format!("lowerdir={lowerdir_config}");
        if let (Some(upperdir), Some(workdir)) = (upperdir, workdir) {
            data = format!("{data},upperdir={upperdir},workdir={workdir}");
        }
        mount(
            KSU_OVERLAY_SOURCE,
            dest.as_ref(),
            "overlay",
            MountFlags::empty(),
            data,
        )?;
    }
    
    if !disable_umount {
        let _ = send_unmountable(dest.as_ref());
    }
    
    Ok(())
}

/// Helper to bind mount a path
pub fn bind_mount(from: impl AsRef<Path>, to: impl AsRef<Path>, disable_umount: bool) -> Result<()> {
    info!(
        "bind mount {} -> {}",
        from.as_ref().display(),
        to.as_ref().display()
    );
    let tree = open_tree(
        CWD,
        from.as_ref(),
        OpenTreeFlags::OPEN_TREE_CLOEXEC
            | OpenTreeFlags::OPEN_TREE_CLONE
            | OpenTreeFlags::AT_RECURSIVE,
    )?;
    move_mount(
        tree.as_fd(),
        "",
        CWD,
        to.as_ref(),
        MoveMountFlags::MOVE_MOUNT_F_EMPTY_PATH,
    )?;
    
    if !disable_umount {
        let _ = send_unmountable(to.as_ref());
    }
    
    Ok(())
}

/// Handles recursive overlay mounting for child mount points (e.g. /system/vendor)
fn mount_overlay_child(
    mount_point: &str,
    relative: &str,
    module_roots: &[String],
    stock_root: &str,
    disable_umount: bool,
) -> Result<()> {
    
    let has_modification = module_roots.iter().any(|lower| {
        let path = Path::new(lower).join(relative.trim_start_matches('/'));
        path.exists()
    });

    if !has_modification {
        return bind_mount(stock_root, mount_point, disable_umount);
    }

    if !Path::new(stock_root).is_dir() {
        return Ok(());
    }

    let mut lower_dirs: Vec<String> = vec![];
    for lower in module_roots {
        let path = Path::new(lower).join(relative.trim_start_matches('/'));
        if path.is_dir() {
            lower_dirs.push(path.display().to_string());
        } else if path.exists() {
            return Ok(());
        }
    }

    if lower_dirs.is_empty() {
        return Ok(());
    }

    if let Err(e) = mount_overlayfs(&lower_dirs, stock_root, None, None, mount_point, disable_umount) {
        warn!("failed to overlay child {mount_point}: {e:#}, fallback to bind mount");
        bind_mount(stock_root, mount_point, disable_umount)?;
    }
    Ok(())
}

/// The main entry point for overlay mounting with robustness
pub fn mount_overlay(
    target_root: &str,
    module_roots: &[String],
    workdir: Option<PathBuf>,
    upperdir: Option<PathBuf>,
    disable_umount: bool,
) -> Result<()> {
    info!("Starting robust overlay mount for {target_root}");
    
    std::env::set_current_dir(target_root)
        .with_context(|| format!("failed to chdir to {target_root}"))?;
    
    let stock_root = ".";

    let mounts = Process::myself()?
        .mountinfo()
        .with_context(|| "get mountinfo")?;
        
    let mut mount_seq = mounts.0.iter()
        .filter(|m| {
            m.mount_point.starts_with(target_root) && 
            m.mount_point != Path::new(target_root)
        })
        .map(|m| m.mount_point.to_string_lossy().to_string())
        .collect::<Vec<_>>();
        
    mount_seq.sort();
    mount_seq.dedup();

    mount_overlayfs(module_roots, target_root, upperdir, workdir, target_root, disable_umount)
        .with_context(|| format!("mount overlayfs for root {target_root} failed"))?;

    for mount_point in mount_seq {
        let relative = mount_point.replacen(target_root, "", 1);
        
        let stock_root_relative = format!("{}{}", stock_root, relative);
        
        if !Path::new(&stock_root_relative).exists() {
            continue;
        }

        if let Err(e) = mount_overlay_child(&mount_point, &relative, module_roots, &stock_root_relative, disable_umount) {
            warn!("failed to restore child mount {mount_point}: {e:#}");
        }
    }
    
    Ok(())
}
