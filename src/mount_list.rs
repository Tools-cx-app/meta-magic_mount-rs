// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{
    cell::RefCell,
    cmp::Reverse,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use rustix::mount::{UnmountFlags, unmount};

use crate::{
    defs,
    errors::{Error, Result},
};

pub struct MountList {
    path: PathBuf,
    targets: RefCell<Vec<PathBuf>>,
    staged: RefCell<Vec<PathBuf>>,
}

impl MountList {
    pub fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        Ok(Self {
            path,
            targets: RefCell::new(Vec::new()),
            staged: RefCell::new(Vec::new()),
        })
    }

    pub fn persistent() -> Result<Self> {
        Self::new(defs::UMOUNT_LIST)
    }

    pub fn record<T>(&self, target: T)
    where
        T: AsRef<Path>,
    {
        let target = target.as_ref();
        let mut targets = self.targets.borrow_mut();
        if !targets.iter().any(|mount| mount == target) {
            targets.push(target.to_path_buf());
        }
    }

    pub fn record_if_final<T>(&self, target: T, has_tmpfs: bool)
    where
        T: AsRef<Path>,
    {
        if has_tmpfs {
            self.staged.borrow_mut().push(target.as_ref().to_path_buf());
        } else {
            self.record(target);
        }
    }

    pub fn commit_staged_under<P>(&self, parent: P)
    where
        P: AsRef<Path>,
    {
        let parent = parent.as_ref();
        let mut staged = self.staged.borrow_mut();
        let mut committed = Vec::new();
        staged.retain(|target| {
            if target.starts_with(parent) {
                committed.push(target.clone());
                false
            } else {
                true
            }
        });
        drop(staged);
        for target in committed {
            self.record(target);
        }
    }

    fn save(&self) -> Result<()> {
        let content = self
            .targets
            .borrow()
            .iter()
            .map(|target| target.to_string_lossy())
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&self.path, content + "\n")?;
        Ok(())
    }

    pub fn unmount_persisted() -> Result<()> {
        unmount_from(Path::new(defs::UMOUNT_LIST), |target| {
            unmount(target, UnmountFlags::DETACH).map_err(Error::from)
        })
    }
}

impl Drop for MountList {
    fn drop(&mut self) {
        if let Err(error) = self.save() {
            log::error!("failed to persist mount list: {error}");
        }
    }
}

fn unmount_from<F>(path: &Path, mut detach: F) -> Result<()>
where
    F: FnMut(&Path) -> Result<()>,
{
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    let mut targets: Vec<_> = content
        .lines()
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect();
    targets.sort_by_key(|target| Reverse(target.components().count()));

    let mut failed = Vec::new();
    for target in targets {
        match detach(&target) {
            Ok(()) => log::debug!("unmounted {} in emulated-soft-reboot", target.display()),
            Err(error) => {
                log::warn!(
                    "failed to unmount {} in emulated-soft-reboot: {error}",
                    target.display()
                );
                failed.push(target);
            }
        }
    }

    if failed.is_empty() {
        fs::remove_file(path)?;
    } else {
        let content = failed
            .iter()
            .map(|target| target.to_string_lossy())
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        fs::write(path, content)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "../tests/unit/mount_list.rs"]
mod tests;
