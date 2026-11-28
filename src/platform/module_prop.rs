// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{fs, io::Write, process::Command};

use super::ksu;
use crate::framework::{Result, paths};

fn legacy_update_desc<S>(desc: &S) -> Result<()>
where
    S: ToString,
{
    let prop = fs::read_to_string(paths::MODULE_PROP)?;
    let mut temp = tempfile::Builder::new().tempfile()?;

    let new: Vec<String> = prop
        .lines()
        .map(|l| {
            if l.starts_with("description") {
                format!("description={}", desc.to_string())
            } else {
                l.to_string()
            }
        })
        .collect();

    let _ = temp
        .write_all(new.join("\n").as_bytes())
        .map_err(|e| log::error!("Failed to update description: {e}"));

    fs::rename(temp.path(), paths::MODULE_PROP)?;

    Ok(())
}

pub fn update_desc(files: u32, symbols: u32) -> Result<()> {
    let text = format!(
        "[😋 mount files/symbol {}] An implementation of a metamodule using Magic Mount.",
        files + symbols
    );

    let cmd = if ksu::check_ksu() { "ksud" } else { "apd" };

    let output = Command::new(cmd)
        .args([
            "module",
            "config",
            "set",
            "override.description",
            &text,
            "--temp",
        ])
        .envs([
            ("KSU_MODULE", env!("MODULE_ID")),
            ("AP_MODULE", env!("MODULE_ID")),
        ])
        .output()?;

    if output.status.success() {
        log::debug!("module config override.description successful set!");
    } else {
        log::warn!(
            "failed to set module config override.description: {}, fallback to write regular file",
            String::from_utf8_lossy(&output.stderr)
        );
        legacy_update_desc(&text)?;
    }

    Ok(())
}
