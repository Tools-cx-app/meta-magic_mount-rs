// Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

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
            if let crate::parser::Command::Mount { source, target } = s {
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

        if !target.exists()
            && let Some(parent) = target.parent()
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
            mount_bind(workdir.path(), &t)?;
        } else {
            mount_bind(&s, &t)?;
        }
        if umount {
            send_unmountable(&t);
        }
    }
    Ok(())
}
