// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

mod bind;
mod collect;
mod magic;
mod mirror;
mod mount_list;
mod tree;

use super::Result;
pub(super) use bind::bind_mount;
pub(super) use collect::collect_module_files;
pub(super) use magic::magic_mount;
pub(super) use mount_list::MountList;

pub(crate) fn emulated_soft_reboot() -> Result<()> {
    MountList::unmount_persisted()
}
