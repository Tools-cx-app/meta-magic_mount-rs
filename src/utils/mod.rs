// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

pub mod ksucalls;

use std::{
    fs::{self, create_dir_all},
    io::Write,
    path::Path,
    process::Command,
};

use anyhow::Context;
use extattr::{Flags as XattrFlags, lgetxattr, lsetxattr};
use regex_lite::Regex;

use crate::{
    defs,
    errors::{Error, Result},
    utils::ksucalls::KSU,
};

/// Validate `module_id` format and security
/// Module ID must match: ^[a-zA-Z][a-zA-Z0-9._-]+$
/// - Must start with a letter (a-zA-Z)
/// - Followed by one or more alphanumeric, dot, underscore, or hyphen characters
/// - Minimum length: 2 characters
pub fn validate_module_id(module_id: &str) -> Result<()> {
    let re = Regex::new(r"^[a-zA-Z][a-zA-Z0-9._-]+$")?;
    if re.is_match(module_id) {
        Ok(())
    } else {
        Err(Error::InvalidModuleID {
            module_id: module_id.to_string(),
        })
    }
}

pub fn lsetfilecon<P: AsRef<Path>>(path: P, con: &str) -> Result<()> {
    log::debug!("file: {},con: {}", path.as_ref().display(), con);
    lsetxattr(&path, defs::SELINUX_XATTR, con, XattrFlags::empty()).with_context(|| {
        format!(
            "Failed to change SELinux context for {}",
            path.as_ref().display()
        )
    })?;
    Ok(())
}

pub fn lgetfilecon<P>(path: P) -> Result<String>
where
    P: AsRef<Path>,
{
    let con = lgetxattr(&path, defs::SELINUX_XATTR).with_context(|| {
        format!(
            "Failed to get SELinux context for {}",
            path.as_ref().display()
        )
    })?;
    let con = String::from_utf8_lossy(&con);
    Ok(con.to_string())
}

pub fn ensure_dir_exists<P>(dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let result = create_dir_all(&dir);
    if dir.as_ref().is_dir() && result.is_ok() {
        Ok(())
    } else {
        Err(Error::RegularDirectory {
            path: dir.as_ref().display().to_string(),
        })
    }
}

fn legacy_update_desc<S: ToString>(desc: &S) -> Result<()> {
    let prop = fs::read_to_string(defs::MODULE_PROP)?;
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

    fs::rename(temp.path(), defs::MODULE_PROP)?;

    Ok(())
}

pub fn update_desc(files: u32, symbols: u32) -> Result<()> {
    let text = format!(
        "[😋 mount files/symbol {}] An implementation of a metamodule using Magic Mount.",
        files + symbols
    );

    let cmd = if KSU.load(std::sync::atomic::Ordering::Relaxed) {
        "ksud"
    } else {
        "apd"
    };

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

#[cfg(test)]
#[path = "../../tests/unit/magic_mount/utils.rs"]
mod tests;
