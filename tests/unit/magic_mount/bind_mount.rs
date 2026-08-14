// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use super::*;

#[test]
fn rejects_parent_component_in_mirror_target() {
    let tempdir = tempfile::tempdir().unwrap();
    let target = tempdir.path().join("missing/../target");

    assert!(mirror_paths(&target).is_err());
}

#[test]
fn resolves_mirror_paths_for_missing_target() {
    let tempdir = tempfile::tempdir().unwrap();
    let target = tempdir.path().join("missing/parent/target");

    let (ancestor, relative_target) = mirror_paths(&target).unwrap();
    assert_eq!(ancestor, tempdir.path());
    assert_eq!(relative_target, Path::new("missing/parent/target"));
}

#[test]
fn resolves_existing_symlink_ancestor() {
    use std::os::unix::fs::symlink;

    let tempdir = tempfile::tempdir().unwrap();
    let real = tempdir.path().join("real");
    let link = tempdir.path().join("link");
    fs::create_dir(&real).unwrap();
    symlink(&real, &link).unwrap();

    let (ancestor, relative_target) = mirror_paths(&link.join("missing/target")).unwrap();
    assert_eq!(ancestor, real);
    assert_eq!(relative_target, Path::new("missing/target"));
}

#[test]
fn rejects_dangling_symlink_ancestor() {
    use std::os::unix::fs::symlink;

    let tempdir = tempfile::tempdir().unwrap();
    let link = tempdir.path().join("link");
    symlink(tempdir.path().join("missing"), &link).unwrap();

    assert!(mirror_paths(&link.join("target")).is_err());
}

#[test]
fn refuses_to_mirror_root() {
    let name = format!("missing-bind-target-{}", std::process::id());
    assert!(mirror_paths(&Path::new("/").join(name).join("target")).is_err());
}

#[test]
fn creates_missing_file_in_mirror() {
    let tempdir = tempfile::tempdir().unwrap();
    let source = tempdir.path().join("source");
    fs::File::create(&source).unwrap();
    let target = tempdir.path().join("mirror/parent/target");

    create_mirror_target(&source, &target).unwrap();
    assert!(target.is_file());
}

#[test]
fn does_not_overwrite_existing_mirror_file() {
    let tempdir = tempfile::tempdir().unwrap();
    let source = tempdir.path().join("source");
    let target = tempdir.path().join("target");
    fs::write(&source, b"source").unwrap();
    fs::write(&target, b"keep").unwrap();

    assert!(create_mirror_target(&source, &target).is_err());
    assert_eq!(fs::read(&target).unwrap(), b"keep");
}

#[test]
fn creates_missing_directory_in_mirror() {
    let tempdir = tempfile::tempdir().unwrap();
    let source = tempdir.path().join("source");
    fs::create_dir(&source).unwrap();
    let target = tempdir.path().join("mirror/parent/target");

    create_mirror_target(&source, &target).unwrap();
    assert!(target.is_dir());
}

#[test]
fn rejects_unsupported_mirror_entries() {
    use std::os::unix::net::UnixListener;

    let tempdir = tempfile::tempdir().unwrap();
    let socket_path = tempdir.path().join("socket");
    let _listener = UnixListener::bind(&socket_path).unwrap();
    let entry = tempdir.path().read_dir().unwrap().next().unwrap().unwrap();

    let error = validate_mirror_entry(&entry).unwrap_err();
    assert!(error.to_string().contains("unsupported file type"));
}
