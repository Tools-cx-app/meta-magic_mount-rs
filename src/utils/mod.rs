// Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
        .env("KSU_MODULE", env!("MODULE_ID"))
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
