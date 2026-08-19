// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{fs, io, path::PathBuf};

use super::*;

#[test]
fn drop_persists_recorded_targets() {
    let temp = tempfile::tempdir().unwrap();
    let list = temp.path().join("umount.list");

    {
        let mounts = MountList::new(&list).unwrap();
        mounts.record("/system");
        mounts.record("/vendor");
    }

    assert_eq!(fs::read_to_string(list).unwrap(), "/system\n/vendor\n");
}

#[test]
fn records_only_mounts_already_at_their_final_path() {
    let temp = tempfile::tempdir().unwrap();
    let list = temp.path().join("umount.list");

    {
        let mounts = MountList::new(&list).unwrap();
        mounts.record_if_final("/system", false);
        mounts.record_if_final("/vendor/lib/module.so", true);
    }

    assert_eq!(fs::read_to_string(list).unwrap(), "/system\n");
}

#[test]
fn commits_staged_mounts_after_parent_move() {
    let temp = tempfile::tempdir().unwrap();
    let list = temp.path().join("umount.list");

    {
        let mounts = MountList::new(&list).unwrap();
        mounts.record_if_final("/vendor/lib/module.so", true);
        mounts.commit_staged_under("/vendor");
    }

    assert_eq!(fs::read_to_string(list).unwrap(), "/vendor/lib/module.so\n");
}

#[test]
fn unmounts_deepest_paths_first_and_retains_failures() {
    let temp = tempfile::tempdir().unwrap();
    let list = temp.path().join("umount.list");
    fs::write(&list, "/system\n/system/lib/modules\n/vendor\n").unwrap();
    let mut attempted = Vec::new();

    unmount_from(&list, |target| {
        attempted.push(target.to_path_buf());
        if target == std::path::Path::new("/system") {
            Err(crate::errors::Error::Io(io::Error::other("busy")))
        } else {
            Ok(())
        }
    })
    .unwrap();

    assert_eq!(
        attempted,
        ["/system/lib/modules", "/system", "/vendor"]
            .map(PathBuf::from)
            .to_vec()
    );
    assert_eq!(fs::read_to_string(list).unwrap(), "/system\n");
}

#[test]
fn removes_list_after_all_targets_unmount() {
    let temp = tempfile::tempdir().unwrap();
    let list = temp.path().join("umount.list");
    fs::write(&list, "/system\n/vendor\n").unwrap();

    unmount_from(&list, |_| Ok::<_, crate::errors::Error>(())).unwrap();

    assert!(!list.exists());
}
