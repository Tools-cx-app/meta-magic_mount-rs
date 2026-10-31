// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use super::*;

#[test]
fn extra_mount_collects_top_level_modules_and_honors_exclusions() {
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
        collect_module_files(tmp.path(), &[], &[], &FxHashSet::default())
            .unwrap()
            .is_none()
    );
    let root = collect_module_files(tmp.path(), &[], &extra, &FxHashSet::default())
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
    let root = collect_module_files(tmp.path(), &[], &extra, &FxHashSet::default())
        .unwrap()
        .unwrap();
    assert!(
        root.children["system"].children["etc"]
            .children
            .contains_key("system_file")
    );
    assert!(root.children.contains_key("my_product"));
}

#[test]
fn remove_marker_collects_only_directories_and_overrides_replace() {
    let tmp = tempfile::tempdir().unwrap();
    let module = tmp.path().join("example");
    fs::create_dir_all(module.join("system/app/Removed")).unwrap();
    fs::create_dir_all(module.join("my_product/etc/Removed")).unwrap();
    fs::write(module.join("module.prop"), "id=example\n").unwrap();
    for path in ["system/app/Removed", "my_product/etc/Removed"] {
        let dir = module.join(path);
        fs::write(dir.join(".remove"), "").unwrap();
        fs::write(dir.join(".replace"), "").unwrap();
        fs::write(dir.join("ignored"), "must not be mounted").unwrap();
    }
    fs::write(module.join("system/app/ordinary.remove"), "ordinary file").unwrap();
    symlink("Removed", module.join("system/app/link")).unwrap();
    let root = collect_module_files(
        tmp.path(),
        &[],
        &["my_product".to_string()],
        &FxHashSet::default(),
    )
    .unwrap()
    .unwrap();
    let app = &root.children["system"].children["app"];
    for node in [
        &app.children["Removed"],
        &root.children["my_product"].children["etc"].children["Removed"],
    ] {
        assert_eq!(node.file_type, NodeFileType::Directory);
        assert!(node.remove);
        assert!(node.replace);
        assert!(node.children.is_empty());
    }
    assert!(!app.children["ordinary.remove"].remove);
    assert!(!app.children["ordinary.remove"].skip);
    assert_eq!(app.children["link"].file_type, NodeFileType::Symlink);
    assert!(!app.children["link"].remove);
}

#[test]
fn remove_marker_omits_directory_from_parent_mirror() {
    let tmp = tempfile::tempdir().unwrap();
    let module = tmp.path().join("module");
    let real = tmp.path().join("real");
    let work = tmp.path().join("work");
    for base in [&module, &real] {
        fs::create_dir_all(base.join("app/Removed")).unwrap();
        fs::write(base.join("app/Removed/content"), "unchanged").unwrap();
    }
    fs::write(module.join("app/Removed/.remove"), "").unwrap();
    fs::write(module.join("app/Removed/.replace"), "").unwrap();
    fs::create_dir_all(module.join("app/Dangling")).unwrap();
    fs::write(module.join("app/Dangling/.remove"), "").unwrap();
    symlink("missing", real.join("app/Dangling")).unwrap();
    fs::create_dir_all(work.join("app")).unwrap();
    let mut root = Node::new_root("");
    assert!(root.collect_module_files(&module).unwrap());
    let mounts = mount_list::MountList::new(tmp.path().join("mounts")).unwrap();
    let mut mount = MagicMount::new(&root.children["app"], &real, &work, true, false, &mounts);
    mount.mount_path(true).unwrap();
    assert!(!work.join("app/Removed").exists());
    assert!(work.join("app/Dangling").symlink_metadata().is_err());
    assert_eq!(
        fs::read_link(real.join("app/Dangling")).unwrap(),
        Path::new("missing")
    );
    assert_eq!(
        fs::read_to_string(real.join("app/Removed/content")).unwrap(),
        "unchanged"
    );
    assert_eq!(
        fs::read_to_string(module.join("app/Removed/content")).unwrap(),
        "unchanged"
    );
}
