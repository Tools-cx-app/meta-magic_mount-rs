// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use super::*;
use std::{
    ffi::OsStr,
    mem,
    os::unix::{ffi::OsStrExt, fs::symlink},
};

#[test]
fn late_load_detects_only_bit_two_and_returns_false_on_probe_failure() {
    let ksu_before = KSU.load(Ordering::Relaxed);
    for (flags, expected) in [(0, false), (1, false), (2, false), (4, true), (7, true)] {
        assert_eq!(
            check_late_load_with(|_, cmd| {
                cmd.flags = flags;
                Ok(())
            }),
            expected
        );
    }
    assert!(!check_late_load_with(|_, _| Err(
        io::Error::from_raw_os_error(libc::ENOTTY)
    )));
    assert!(check_late_load_with(|request, cmd| {
        if request == 0x80104b02 {
            return Err(io::Error::from_raw_os_error(libc::ENOTTY));
        }
        cmd.flags = 4;
        Ok(())
    }));
    assert_eq!(KSU.load(Ordering::Relaxed), ksu_before);
}

#[test]
fn commands_match_kernel_abi() {
    assert_eq!(mem::size_of::<GetInfoCmd>(), 16);
    assert_eq!(mem::offset_of!(GetInfoCmd, uapi_version), 12);
    assert_eq!(mem::size_of::<TryUmountCmd>(), 16);
    assert_eq!(mem::align_of::<TryUmountCmd>(), 8);
    assert_eq!(mem::offset_of!(TryUmountCmd, flags), 8);
    assert_eq!(mem::offset_of!(TryUmountCmd, mode), 12);
}

#[test]
fn get_info_falls_back_to_legacy_and_propagates_failure() {
    let mut requests = Vec::new();
    let info = get_info_with(|request, cmd| {
        requests.push(request);
        if request == 0x80104b02 {
            return Err(std::io::Error::from_raw_os_error(libc::ENOTTY));
        }
        cmd.version = 40001;
        Ok(())
    })
    .unwrap();
    assert_eq!(info.version, 40001);
    assert_eq!(requests, [0x80104b02, 0x80004b02]);
    assert!(get_info_with(|_, _| Err(std::io::Error::from_raw_os_error(libc::ENOTTY))).is_err());
    let info = get_info_with(|request, cmd| {
        assert_eq!(request, 0x80104b02);
        cmd.version = 50001;
        Ok(())
    })
    .unwrap();
    assert_eq!(info.version, 50001);
}

#[test]
fn inherited_su_driver_is_preferred() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(scan_driver_fd(dir.path()).unwrap(), None);
    symlink("anon_inode:[ksu_driver]", dir.path().join("10")).unwrap();
    symlink("anon_inode:[other]", dir.path().join("11")).unwrap();
    assert_eq!(scan_driver_fd(dir.path()).unwrap(), Some(10));
    symlink("anon_inode:[ksu_driver_su]", dir.path().join("12")).unwrap();
    assert_eq!(scan_driver_fd(dir.path()).unwrap(), Some(12));
}

#[test]
fn registration_preserves_path_bytes_and_validates_kernel_limits() {
    let path = Path::new(OsStr::from_bytes(b"/system/\xff"));
    add_umount_with(path, |cmd| {
        assert_eq!(cmd.flags, 2);
        assert_eq!(cmd.mode, 1);
        let path = unsafe { std::ffi::CStr::from_ptr(cmd.arg as *const libc::c_char) };
        assert_eq!(path.to_bytes(), b"/system/\xff");
        Ok(())
    })
    .unwrap();
    add_umount_with(Path::new(&"x".repeat(255)), |_| Ok(())).unwrap();
    for path in ["", "x\0y", &"x".repeat(256)] {
        assert!(
            add_umount_with(Path::new(path), |_| panic!("invalid path reached ioctl")).is_err()
        );
    }
}

#[test]
fn duplicate_registration_succeeds_but_other_errors_keep_path_context() {
    let path = Path::new("/system/example");
    add_umount_with(path, |_| {
        Err(std::io::Error::from_raw_os_error(libc::EEXIST))
    })
    .unwrap();
    let error = add_umount_with(path, |_| {
        Err(std::io::Error::from_raw_os_error(libc::EPERM))
    })
    .unwrap_err();
    assert!(error.to_string().contains("/system/example"));
    assert_eq!(
        error
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .raw_os_error(),
        Some(libc::EPERM)
    );
}
