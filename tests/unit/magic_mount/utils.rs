// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

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

    match ensure_dir_exists(&file_path) {
        Err(Error::RegularDirectory { path }) => {
            assert_eq!(path, file_path.display().to_string());
        }
        _ => panic!("Expected Error::RegularDirectory"),
    }
}
