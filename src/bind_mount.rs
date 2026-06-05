// Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
// SPDX-License-Identifier: Apache-2.0

use std::fs;

use rustix::mount::mount_bind;

use crate::{errors::Result, parser::COMMAND_LIST, utils::ksucalls::send_unmountable};

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

        if fs::exists(&s).is_err() || fs::exists(&t).is_err() {
            log::error!("source/target isn't existed, skip!!");
            continue;
        }

        mount_bind(&s, &t)?;
        if umount {
            send_unmountable(&t);
        }
    }
    Ok(())
}
