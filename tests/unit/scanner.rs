// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::fs;

use super::*;

fn create_valid_prop_content(id: &str) -> String {
    format!("id={id}\nname=Test Module\nversion=1.0.0\nauthor=Tester\ndescription=A test module\n")
}

#[test]
fn test_read_prop_success() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let prop_path = tmp_dir.path().join("module.prop");
    fs::write(&prop_path, "id=test\nname=test\nversion=v1.0\n").unwrap();

    let res = read_prop(&prop_path).unwrap();
    assert_eq!(res.get("id").unwrap(), "test");
    assert_eq!(res.get("name").unwrap(), "test");
    assert_eq!(res.get("version").unwrap(), "v1.0");
}

#[test]
fn test_read_prop_file_not_found() {
    assert!(read_prop("non_existent_file.prop").is_err());
}

#[test]
fn test_list_modules_integration() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let module_dir = tmp_dir.path();

    let test1 = module_dir.join("test1");
    fs::create_dir_all(test1.join("system")).unwrap();
    fs::write(
        test1.join("module.prop"),
        create_valid_prop_content("test1"),
    )
    .unwrap();

    let test2 = module_dir.join("test2");
    fs::create_dir_all(test2.join("system")).unwrap();
    fs::write(
        test2.join("module.prop"),
        create_valid_prop_content("test2"),
    )
    .unwrap();
    fs::File::create(test2.join(defs::DISABLE_FILE_NAME)).unwrap();

    let test3 = module_dir.join("test3");
    fs::create_dir_all(test3.join("system")).unwrap();
    fs::write(
        test3.join("module.prop"),
        create_valid_prop_content("test3"),
    )
    .unwrap();
    fs::File::create(test3.join(defs::SKIP_MOUNT_FILE_NAME)).unwrap();

    let test4 = module_dir.join("test4");
    fs::create_dir_all(&test4).unwrap();
    fs::write(test4.join("module.prop"), "id=test4\n").unwrap();

    let test5 = module_dir.join("test5");
    fs::create_dir_all(&test5).unwrap();
    fs::write(
        test5.join("module.prop"),
        create_valid_prop_content("test5"),
    )
    .unwrap();

    let test6 = module_dir.join("test6");
    fs::create_dir_all(test6.join("vendor")).unwrap();
    fs::write(
        test6.join("module.prop"),
        create_valid_prop_content("test6"),
    )
    .unwrap();

    let result: Vec<api::Module> = list_modules(module_dir, &["vendor".to_string()]);
    assert_eq!(result.len(), 5);
    assert_eq!(result[0].id, "test1");
    assert!(result[0].is_mounted);
    assert!(!result[1].is_mounted);
    assert!(!result[2].is_mounted);
    assert!(!result[3].is_mounted);
    assert!(result[4].is_mounted);
}

#[test]
fn test_list_modules_empty_dir() {
    let tmp_dir = tempfile::tempdir().unwrap();
    assert_eq!(list_modules(tmp_dir.path(), &[]), Vec::new());
}
