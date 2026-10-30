// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::fs;

use super::*;

fn create_valid_prop_content(id: &str) -> String {
    format!(
        "id={}\nname=Test Module\nversion=1.0.0\nauthor=Tester\ndescription=A test module\n",
        id
    )
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

    let result = list_modules(module_dir, &["vendor".to_string()]);
    assert_eq!(result.len(), 5);
    assert_eq!(result[0].id, "test1");
    assert_eq!(result[1].id, "test2");
    assert_eq!(result[2].id, "test3");
    assert_eq!(result[3].id, "test5");
    assert_eq!(result[4].id, "test6");
    assert!(result[0].is_mounted);
    assert!(result[0].enabled);
    assert_eq!(result[0].mode, "magic");
    assert!(!result[1].is_mounted);
    assert!(!result[1].enabled);
    assert_eq!(result[1].mode, "ignore");
    assert!(!result[2].is_mounted);
    assert!(result[2].enabled);
    assert_eq!(result[2].mode, "ignore");
    assert!(!result[3].is_mounted);
    assert!(result[3].enabled);
    assert_eq!(result[3].mode, "ignore");
    assert!(result[4].is_mounted);
    assert_eq!(result[4].mode, "magic");
}

#[test]
fn test_list_modules_empty_dir() {
    let tmp_dir = tempfile::tempdir().unwrap();
    assert!(list_modules(tmp_dir.path(), &[]).is_empty());
}

#[test]
fn modules_status_preserves_lua_snapshot_by_id() {
    let root = tempfile::tempdir().unwrap();
    for id in ["skipped", "mounted"] {
        let path = root.path().join(id);
        fs::create_dir_all(path.join("system")).unwrap();
        fs::write(path.join("module.prop"), create_valid_prop_content(id)).unwrap();
    }
    let hooks_dir = root.path().join("lua");
    fs::create_dir(&hooks_dir).unwrap();
    fs::write(
        hooks_dir.join("skip.lua"),
        "function before() modules.skipped.skip = true end",
    )
    .unwrap();
    let mut boot_modules = list_modules(root.path(), &[]);
    let hooks = crate::parser::LuaParser::load(&hooks_dir, &boot_modules, root.path()).unwrap();
    hooks.before(&mut boot_modules).unwrap();
    let snapshot = root.path().join("scan.ret");
    fs::write(&snapshot, serde_json::to_vec(&boot_modules).unwrap()).unwrap();

    // Rename a mounted module and disable it for next boot; its current mount remains.
    fs::write(
        root.path().join("mounted/module.prop"),
        create_valid_prop_content("mounted").replace("Test Module", "Renamed Module"),
    )
    .unwrap();
    fs::write(root.path().join("mounted/disable"), "").unwrap();
    fs::create_dir_all(root.path().join("new_module/system")).unwrap();
    fs::write(
        root.path().join("new_module/module.prop"),
        create_valid_prop_content("new_module"),
    )
    .unwrap();

    let shown = show_modules(list_modules(root.path(), &[]), &snapshot).unwrap();
    let skipped = shown.iter().find(|module| module.id == "skipped").unwrap();
    assert!(!skipped.is_mounted);
    assert_eq!(skipped.mode, "ignore");
    assert_eq!(skipped.rules.default_mode, "ignore");
    assert!(skipped.enabled);
    let mounted = shown.iter().find(|module| module.id == "mounted").unwrap();
    assert!(mounted.is_mounted);
    assert_eq!(mounted.mode, "magic");
    assert_eq!(mounted.rules.default_mode, "magic");
    assert_eq!(mounted.name, "Renamed Module");
    assert!(!mounted.enabled);
    let new_module = shown
        .iter()
        .find(|module| module.id == "new_module")
        .unwrap();
    assert!(!new_module.is_mounted);
    assert_eq!(new_module.mode, "ignore");
    assert_eq!(new_module.rules.default_mode, "ignore");
}
