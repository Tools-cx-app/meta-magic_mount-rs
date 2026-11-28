// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::path::Path;

use anyhow::Context;
use extattr::{Flags as XattrFlags, lgetxattr, lsetxattr};

use crate::framework::Result;

const SELINUX_XATTR: &str = "security.selinux";

pub fn lsetfilecon<P>(path: P, con: &str) -> Result<()>
where
    P: AsRef<Path>,
{
    log::debug!("file: {},con: {}", path.as_ref().display(), con);
    lsetxattr(&path, SELINUX_XATTR, con, XattrFlags::empty()).with_context(|| {
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
    let con = lgetxattr(&path, SELINUX_XATTR).with_context(|| {
        format!(
            "Failed to get SELinux context for {}",
            path.as_ref().display()
        )
    })?;
    let con = String::from_utf8_lossy(&con);
    Ok(con.to_string())
}
