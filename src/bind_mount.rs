// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{
    fs,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

use rustix::mount::{MountFlags, UnmountFlags, mount_bind, mount_move, mount_remount, unmount};

use crate::{
    errors::Result, magic_mount::utils::mount_mirror, parser::COMMAND_LIST,
    utils::ksucalls::send_unmountable,
};

fn mirror_paths(target: &Path) -> Result<(PathBuf, PathBuf)> {
    let mut ancestor = target.parent().ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidInput,
            format!("target has no parent: {}", target.display()),
        )
    })?;

    while !ancestor.exists() {
        ancestor = ancestor.parent().ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                format!("target has no existing ancestor: {}", target.display()),
            )
        })?;
    }

    if ancestor.parent().is_none() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!(
                "refusing to mirror the root directory for {}",
                target.display()
            ),
        )
        .into());
    }

    let relative_target = target.strip_prefix(ancestor).map_err(|error| {
        Error::new(
            ErrorKind::InvalidInput,
            format!(
                "cannot resolve {} relative to {}: {error}",
                target.display(),
                ancestor.display()
            ),
        )
    })?;

    Ok((ancestor.to_path_buf(), relative_target.to_path_buf()))
}

fn create_mirror_target(source: &Path, target: &Path) -> Result<()> {
    if source.is_dir() {
        fs::create_dir_all(target)?;
    } else {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::File::create(target)?;
    }

    Ok(())
}

fn mount_readonly(source: &Path, target: &Path) -> Result<()> {
    mount_bind(source, target)?;
    if let Err(error) = mount_remount(target, MountFlags::BIND | MountFlags::RDONLY, "") {
        if let Err(unmount_error) = unmount(target, UnmountFlags::DETACH) {
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
    let workdir = tempfile::Builder::new().tempdir()?;
    mount_bind(workdir.path(), workdir.path())?;

    let result = (|| -> Result<()> {
        for entry in ancestor.read_dir()? {
            mount_mirror(ancestor.as_path(), workdir.path(), &entry?)?;
        }

        let mirror_target = workdir.path().join(relative_target);
        create_mirror_target(source, &mirror_target)?;
        mount_readonly(source, &mirror_target)?;
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

pub fn bind_mount(umount: bool) -> Result<()> {
    let bind_mount_list: Vec<_> = COMMAND_LIST
        .get()
        .unwrap()
        .iter()
        .filter_map(|s| {
            if let crate::parser::MountType::Mount { source, target } = s {
                Some((source.clone(), target.clone()))
            } else {
                None
            }
        })
        .collect();

    for (s, t) in bind_mount_list {
        log::debug!("bind mount: {s} -> {t}");

        let source = Path::new(&s);
        let target = Path::new(&t);
        if !source.exists() {
            log::error!("source doesn't exist, skip: {}", source.display());
            continue;
        }

        let unmount_target = if target.exists() {
            mount_readonly(source, target)?;
            target.to_path_buf()
        } else {
            mount_missing_target(source, target)?
        };

        if umount {
            send_unmountable(unmount_target);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_mirror_paths_for_missing_target() {
        let tempdir = tempfile::tempdir().unwrap();
        let target = tempdir.path().join("missing/parent/target");

        let (ancestor, relative_target) = mirror_paths(&target).unwrap();
        assert_eq!(ancestor, tempdir.path());
        assert_eq!(relative_target, Path::new("missing/parent/target"));
    }

    #[test]
    fn creates_missing_file_in_mirror() {
        let tempdir = tempfile::tempdir().unwrap();
        let source = tempdir.path().join("source");
        fs::File::create(&source).unwrap();
        let target = tempdir.path().join("mirror/parent/target");

        create_mirror_target(&source, &target).unwrap();
        assert!(target.is_file());
    }

    #[test]
    fn creates_missing_directory_in_mirror() {
        let tempdir = tempfile::tempdir().unwrap();
        let source = tempdir.path().join("source");
        fs::create_dir(&source).unwrap();
        let target = tempdir.path().join("mirror/parent/target");

        create_mirror_target(&source, &target).unwrap();
        assert!(target.is_dir());
    }
}
