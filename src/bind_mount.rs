// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: Apache-2.0

use std::{fs, path::Path};

use rustix::mount::mount_bind;

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
        let mut has_mirror = false;

        if let Some(parent) = target.parent()
            && parent.exists()
        {
            for entry in parent.read_dir()?.flatten() {
                mount_mirror(parent, workdir.path(), &entry)?;
                has_mirror = true;
            }
        }
        if !source.exists() || (!target.exists() && !has_mirror) {
            log::error!("source/target isn't existed, skip!!");
            continue;
        }

        if has_mirror {
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

            mount_bind(source, &mirror_target)?;
        } else {
            mount_bind(source, target)?;
        }
        if umount {
            send_unmountable(&t);
        }
    }
    Ok(())
}
