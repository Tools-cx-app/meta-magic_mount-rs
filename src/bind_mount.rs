// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{fs, path::Path};

use rustix::mount::{MountFlags, UnmountFlags, mount_bind, mount_remount, unmount};

use crate::{
    errors::Result, magic_mount::utils::mount_mirror, parser::COMMAND_LIST,
    utils::ksucalls::send_unmountable,
};

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
        let workdir = tempfile::Builder::new().tempdir()?;
        let need_mirror = target.parent().is_some_and(|parent| {
            parent.exists() && parent.read_dir().is_ok_and(|mut rd| rd.next().is_some())
        }) && target.exists()
            && target.is_dir();

        if let Some(parent) = target.parent()
            && need_mirror
        {
            for entry in parent.read_dir()?.flatten() {
                mount_mirror(parent, workdir.path(), &entry)?;
            }
        }
        if !source.exists() || (!target.exists() && !need_mirror) {
            log::error!("source/target isn't existed, skip!!");
            continue;
        }

        let target = if need_mirror {
            // mirror source file to workdir
            let mirror_target = workdir.path().join(target.file_name().unwrap());
            if source.is_dir() {
                fs::create_dir_all(&mirror_target)?;
            } else {
                if let Some(p) = mirror_target.parent() {
                    std::fs::create_dir_all(p)?;
                }
                fs::File::create(&mirror_target)?;
            }

            mirror_target
        } else {
            target.to_path_buf()
        };

        mount_bind(source, &target)?;
        mount_remount(&target, MountFlags::BIND | MountFlags::RDONLY, "")?;

        if umount {
            send_unmountable(&target);
        }

        let _ = unmount(workdir.path(), UnmountFlags::DETACH);
        drop(workdir);
    }
    Ok(())
}
