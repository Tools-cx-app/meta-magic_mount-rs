// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use super::*;

#[test]
fn extra_mount_collects_top_level_modules_and_honors_exclusions() {
    COMMAND_LIST.get_or_init(Vec::new);
    let tmp = tempfile::tempdir().unwrap();
    for (id, marker) in [
        ("first", None),
        ("second", None),
        ("disabled", Some(defs::DISABLE_FILE_NAME)),
        ("removed", Some(defs::REMOVE_FILE_NAME)),
        ("skipped", Some(defs::SKIP_MOUNT_FILE_NAME)),
    ] {
        let module = tmp.path().join(id);
        fs::create_dir_all(module.join("my_product/etc")).unwrap();
        fs::write(module.join("my_product/etc").join(id), id).unwrap();
        fs::write(
            module.join("module.prop"),
            format!("id={id}\nname={id}\nversion=1\nauthor=test\ndescription=test\n"),
        )
        .unwrap();
        if let Some(marker) = marker {
            fs::write(module.join(marker), "").unwrap();
        }
    }
    fs::create_dir_all(tmp.path().join("first/my_product/replaced")).unwrap();
    fs::write(tmp.path().join("first/my_product/replaced/.replace"), "").unwrap();
    let extra = vec!["my_product".to_string()];
    assert!(
        collect_module_files(tmp.path(), &[], &[])
            .unwrap()
            .is_none()
    );
    let root = collect_module_files(tmp.path(), &[], &extra)
        .unwrap()
        .unwrap();
    let partition = &root.children["my_product"];
    assert!(partition.module_path.as_ref().unwrap().is_dir());
    let files = &partition.children["etc"].children;
    assert_eq!(files.len(), 2);
    for id in ["first", "second"] {
        assert_eq!(
            fs::read_to_string(files[id].module_path.as_ref().unwrap()).unwrap(),
            id
        );
    }
    assert!(partition.children["replaced"].replace);
    let modules = crate::scanner::list_modules(tmp.path(), &extra);
    let modules = serde_json::to_value(modules).unwrap();
    let mounted: Vec<_> = modules
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["is_mounted"] == true)
        .map(|m| m["id"].as_str().unwrap())
        .collect();
    assert_eq!(mounted, ["first", "second"]);
    fs::create_dir_all(tmp.path().join("first/system/etc")).unwrap();
    fs::write(tmp.path().join("first/system/etc/system_file"), "system").unwrap();
    let root = collect_module_files(tmp.path(), &[], &extra)
        .unwrap()
        .unwrap();
    assert!(
        root.children["system"].children["etc"]
            .children
            .contains_key("system_file")
    );
    assert!(root.children.contains_key("my_product"));
}
