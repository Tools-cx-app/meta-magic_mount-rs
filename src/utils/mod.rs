// Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
// SPDX-License-Identifier: Apache-2.0

pub mod ksucalls;

use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
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

pub fn generate_tmp() -> PathBuf {
    let mut name = String::new();

    for _ in 0..10 {
        name.push(fastrand::alphanumeric());
    }

    Path::new("/mnt").join(name)
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

pub fn update_desc(file: u32, symbol: u32, ignore: u32) -> Result<()> {
    let text = format!(
        "[😋 MF {file},MS {symbol},IG {ignore}] An implementation of a metamodule using Magic Mount."
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
            "failed to set module config override.description: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_validate_module_id_valid() {
        assert!(validate_module_id("testModule").is_ok());
        assert!(validate_module_id("a.b-c_d").is_ok());
    }

    #[test]
    fn test_validate_module_id_invalid() {
        assert!(validate_module_id("123test").is_err());
        assert!(validate_module_id(".test").is_err());
        assert!(validate_module_id("test@test").is_err());
        assert!(validate_module_id("").is_err());
    }

    #[test]
    fn test_generate_tmp() {
        let path = generate_tmp();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let path2 = generate_tmp();

        assert_eq!(path.parent(), Some(Path::new("/mnt")));

        assert_eq!(file_name.len(), 10);

        assert!(file_name.chars().all(|c| c.is_ascii_alphanumeric()));

        assert_ne!(path, path2);
    }

    #[test]
    fn test_ensure_dir_exists_success() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let target_dir = tmp_dir.path().join("test");

        assert!(ensure_dir_exists(&target_dir).is_ok());
        assert!(target_dir.is_dir());
    }

    #[test]
    fn test_ensure_dir_exists_failure_when_file_exists() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("test");

        fs::write(&file_path, "test").unwrap();

        let result = ensure_dir_exists(&file_path);
        assert!(result.is_err());

        match result {
            Err(Error::RegularDirectory { path }) => {
                assert_eq!(path, file_path.display().to_string());
            }
            _ => panic!("Expected Error::RegularDirectory"),
        }
    }
}
