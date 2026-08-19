// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{
    fs::{self, DirEntry},
    io::{Error, ErrorKind},
    path::{Component, Path, PathBuf},
};

use rustix::mount::{MountFlags, UnmountFlags, mount_bind, mount_move, mount_remount, unmount};

use crate::{
    errors::Result,
    magic_mount::utils::mount_mirror,
    mount_list,
    parser::{COMMAND_LIST, MountType},
    utils::ksucalls::send_unmountable,
};

fn mirror_paths(target: &Path) -> Result<(PathBuf, PathBuf)> {
    if !target.is_absolute()
        || target
            .components()
            .any(|component| component == Component::ParentDir)
    {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("invalid mirror target: {}", target.display()),
        )
        .into());
    }

    let mut ancestor = target.parent().ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidInput,
            format!("target has no parent: {}", target.display()),
        )
    })?;
    loop {
        match fs::symlink_metadata(ancestor) {
            Ok(_) => break,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                ancestor = ancestor.parent().ok_or_else(|| {
                    Error::new(
                        ErrorKind::NotFound,
                        format!("target has no existing ancestor: {}", target.display()),
                    )
                })?;
            }
            Err(error) => return Err(error.into()),
        }
    }

    let relative_target = target
        .strip_prefix(ancestor)
        .map_err(|error| Error::new(ErrorKind::InvalidInput, error))?
        .to_path_buf();
    let ancestor = fs::canonicalize(ancestor)?;
    if ancestor.parent().is_none() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("refusing to mirror root for {}", target.display()),
        )
        .into());
    }

    Ok((ancestor, relative_target))
}

fn create_mirror_target(source: &Path, target: &Path) -> Result<()> {
    if source.is_dir() {
        fs::create_dir_all(target)?;
    } else {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(target)?;
    }
    Ok(())
}

fn validate_mirror_entry(entry: &DirEntry) -> Result<()> {
    let file_type = entry.file_type()?;
    if file_type.is_dir() {
        for entry in entry.path().read_dir()? {
            validate_mirror_entry(&entry?)?;
        }
    } else if !file_type.is_file() && !file_type.is_symlink() {
        return Err(Error::new(
            ErrorKind::Unsupported,
            format!(
                "cannot mirror unsupported file type: {}",
                entry.path().display()
            ),
        )
        .into());
    }
    Ok(())
}

fn report_rollback(success: bool, on_failure: impl FnOnce()) {
    if !success {
        on_failure();
    }
}

fn mount_readonly(
    source: &Path,
    target: &Path,
    leaked_mounts: Option<&mount_list::MountList>,
) -> Result<()> {
    mount_bind(source, target)?;
    if let Err(error) = mount_remount(target, MountFlags::BIND | MountFlags::RDONLY, "") {
        let rollback = unmount(target, UnmountFlags::DETACH);
        report_rollback(rollback.is_ok(), || {
            if let Some(mounts) = leaked_mounts {
                mounts.record(target);
            }
        });
        if let Err(unmount_error) = rollback {
            log::error!(
                "failed to roll back bind mount {}: {unmount_error}",
                target.display()
            );
        }
        return Err(error.into());
    }
    Ok(())
}

fn mount_missing_target(source: &Path, target: &Path) -> Result<PathBuf> {
    let (ancestor, relative_target) = mirror_paths(target)?;
    let first_missing_component = relative_target
        .components()
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "target already exists"))?
        .as_os_str();
    let entries = ancestor.read_dir()?.collect::<std::io::Result<Vec<_>>>()?;

    for entry in &entries {
        if entry.file_name() == first_missing_component {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                format!(
                    "target changed while preparing bind mount: {}",
                    target.display()
                ),
            )
            .into());
        }
        validate_mirror_entry(entry)?;
    }

    let workdir = tempfile::Builder::new().tempdir()?;
    mount_bind(workdir.path(), workdir.path())?;

    let result = (|| -> Result<()> {
        for entry in &entries {
            mount_mirror(ancestor.as_path(), workdir.path(), entry)?;
        }

        let mirror_target = workdir.path().join(relative_target);
        create_mirror_target(source, &mirror_target)?;
        mount_readonly(source, &mirror_target, None)?;
        mount_remount(workdir.path(), MountFlags::BIND | MountFlags::RDONLY, "")?;
        mount_move(workdir.path(), &ancestor)?;
        Ok(())
    })();

    if let Err(error) = result {
        if let Err(unmount_error) = unmount(workdir.path(), UnmountFlags::DETACH) {
            log::error!(
                "failed to clean up bind mount workdir {}: {unmount_error}",
                workdir.path().display()
            );
        }
        return Err(error);
    }

    Ok(ancestor)
}

fn mount_target(source: &Path, target: &Path, mounts: &mount_list::MountList) -> Result<PathBuf> {
    match target.metadata() {
        Ok(_) => {
            let target = fs::canonicalize(target)?;
            mount_readonly(source, &target, Some(mounts))?;
            Ok(target)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => mount_missing_target(source, target),
        Err(error) => Err(error.into()),
    }
}

pub fn bind_mount(umount: bool, mounts: &mount_list::MountList) -> Result<()> {
    let commands = COMMAND_LIST
        .get()
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "mount command list is not initialized"))?;

    for command in commands {
        let MountType::Mount { source, target } = command else {
            continue;
        };

        log::debug!("bind mount: {source} -> {target}");

        let source = Path::new(source);
        if !source.exists() {
            log::error!("source doesn't exist, skip: {}", source.display());
            continue;
        }

        let unmount_target = mount_target(source, Path::new(target), mounts)?;
        mounts.record(&unmount_target);
        if umount {
            send_unmountable(unmount_target);
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "../tests/unit/bind_mount.rs"]
mod tests;
