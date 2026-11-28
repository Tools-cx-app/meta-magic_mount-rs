// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use super::*;
use crate::framework::extension::{Extensions, ModulePlan};
use crate::framework::{mount, paths, scanner::scan_modules};
use std::os::unix::fs::symlink;

fn collect_module_files(
    module_dir: &Path,
    partitions: &[String],
    extra: &[String],
) -> Result<Option<Node>> {
    let hooks = Extensions::load(&module_dir.join("lua"), module_dir)?;
    let mut modules = scan_modules(module_dir);
    mount::collect_module_files(&mut modules, partitions, extra, &hooks)
}

#[test]
fn extra_mount_collects_top_level_modules_and_honors_exclusions() {
    let tmp = tempfile::tempdir().unwrap();
    for (id, marker) in [
        ("first", None),
        ("second", None),
        ("disabled", Some(paths::DISABLE_FILE_NAME)),
        ("removed", Some(paths::REMOVE_FILE_NAME)),
        ("skipped", Some(paths::SKIP_MOUNT_FILE_NAME)),
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
    let hooks = Extensions::load(&tmp.path().join("lua"), tmp.path()).unwrap();
    let mut modules = scan_modules(tmp.path());
    mount::collect_module_files(&mut modules, &[], &extra, &hooks).unwrap();
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

#[test]
fn remove_marker_collects_only_directories_and_overrides_replace() {
    let tmp = tempfile::tempdir().unwrap();
    let module = tmp.path().join("example");
    fs::create_dir_all(module.join("system/app/Removed")).unwrap();
    fs::create_dir_all(module.join("my_product/etc/Removed")).unwrap();
    fs::write(
        module.join("module.prop"),
        "id=example\nname=Example\nversion=1\nauthor=test\ndescription=test\n",
    )
    .unwrap();
    for path in ["system/app/Removed", "my_product/etc/Removed"] {
        let dir = module.join(path);
        fs::write(dir.join(".remove"), "").unwrap();
        fs::write(dir.join(".replace"), "").unwrap();
        fs::write(dir.join("ignored"), "must not be mounted").unwrap();
    }
    fs::write(module.join("system/app/ordinary.remove"), "ordinary file").unwrap();
    symlink("Removed", module.join("system/app/link")).unwrap();
    let root = collect_module_files(tmp.path(), &[], &["my_product".to_string()])
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
    root.collect_filtered(&module, Path::new(""), &ModulePlan::default())
        .unwrap();
    assert!(root.has_content());
    let mounts = MountList::new(tmp.path().join("mounts")).unwrap();
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

fn write_module(root: &Path, directory: &str, id: &str) -> PathBuf {
    let module = root.join(directory);
    fs::create_dir_all(&module).unwrap();
    fs::write(
        module.join("module.prop"),
        format!("id={id}\nname={id}\nversion=1\nauthor=test\ndescription=test\n"),
    )
    .unwrap();
    module
}

#[test]
fn lua_rules_filter_each_module_before_merge_and_remove_globally() {
    let root = tempfile::tempdir().unwrap();
    let first = write_module(root.path(), "directory_a", "aa");
    let second = write_module(root.path(), "directory_b", "bb");
    for module in [&first, &second] {
        fs::create_dir_all(module.join("system/etc/blocked_dir")).unwrap();
        fs::write(
            module.join("system/etc/shared"),
            module.to_string_lossy().as_bytes(),
        )
        .unwrap();
        fs::write(module.join("system/etc/blocked_dir/content"), "keep second").unwrap();
        fs::write(module.join("system/etc/remove_me"), "hide globally").unwrap();
    }
    let source = root.path().join("external-source");
    fs::write(&source, "replacement").unwrap();
    let scripts = root.path().join("lua");
    fs::create_dir(&scripts).unwrap();
    fs::write(scripts.join("rules.lua"), format!("API = 2\nMODULE = 'aa'\nfunction load() block('system//etc/./shared'); block('system/etc/blocked_dir'); replace('{0}', 'system/etc/new'); replace('{0}', 'system/etc/shared'); remove('system/etc/remove_me'); remove('system/app/Absent') end", source.display())).unwrap();
    let hooks = Extensions::load(&scripts, root.path()).unwrap();
    let mut modules = scan_modules(root.path());
    let tree = mount::collect_module_files(&mut modules, &[], &[], &hooks)
        .unwrap()
        .unwrap();
    let etc = &tree.children["system"].children["etc"];
    assert_eq!(
        etc.children["shared"].module_path,
        Some(second.join("system/etc/shared"))
    );
    assert_eq!(
        etc.children["blocked_dir"].children["content"].module_path,
        Some(second.join("system/etc/blocked_dir/content"))
    );
    assert_eq!(etc.children["new"].module_path, Some(source));
    assert_eq!(etc.children["remove_me"].file_type, NodeFileType::Whiteout);
    assert!(tree.children["system"].children["app"].children["Absent"].remove);
    assert!(first.join("system/etc/remove_me").is_file());
    assert!(second.join("system/etc/remove_me").is_file());
    let snapshot = serde_json::to_value(&modules).unwrap();
    assert_eq!(snapshot[0]["rules"]["paths"]["system/etc/shared"], "block");
    assert_eq!(snapshot[0]["rules"]["paths"]["system/etc/new"], "replace");
}

#[test]
fn rules_only_modules_support_directories_extra_mount_and_last_replace() {
    let root = tempfile::tempdir().unwrap();
    write_module(root.path(), "different_dir", "example");
    let source = root.path().join("external");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("content"), "external content").unwrap();
    symlink("missing", source.join("dangling")).unwrap();
    let later = root.path().join("later");
    fs::write(&later, "later").unwrap();
    let scripts = root.path().join("lua");
    fs::create_dir(&scripts).unwrap();
    fs::write(scripts.join("10-rules.lua"), format!("API = 2\nMODULE = 'example'\nfunction load() replace('{}', 'system/etc/renamed'); replace('{}', 'my_product/etc/new'); remove('system/bin/removed') end", source.display(), source.display())).unwrap();
    fs::write(
        scripts.join("20-rules.lua"),
        format!(
            "API = 2\nMODULE = 'example'\nfunction load() replace('{}', 'my_product/etc/new') end",
            later.display()
        ),
    )
    .unwrap();
    let hooks = Extensions::load(&scripts, root.path()).unwrap();
    let mut modules = scan_modules(root.path());
    let tree = mount::collect_module_files(&mut modules, &[], &["my_product".into()], &hooks)
        .unwrap()
        .unwrap();
    let replaced = &tree.children["system"].children["etc"].children["renamed"];
    assert_eq!(replaced.name, "renamed");
    assert!(replaced.replace);
    assert_eq!(
        replaced.children["content"].module_path,
        Some(source.join("content"))
    );
    assert_eq!(
        replaced.children["dangling"].file_type,
        NodeFileType::Symlink
    );
    assert_eq!(
        tree.children["my_product"].children["etc"].children["new"].module_path,
        Some(later)
    );
    assert!(modules[0].is_mountable());
}

#[test]
fn remove_only_creates_tree_and_hides_dangling_links_in_mirror() {
    let root = tempfile::tempdir().unwrap();
    write_module(root.path(), "example", "example");
    let scripts = root.path().join("lua");
    fs::create_dir(&scripts).unwrap();
    fs::write(scripts.join("rules.lua"), "API = 2\nMODULE = 'example'\nfunction load() remove('system/etc/link'); remove('system/etc/directory'); remove('system/etc/directory/child') end").unwrap();
    let hooks = Extensions::load(&scripts, root.path()).unwrap();
    let mut modules = scan_modules(root.path());
    let tree = mount::collect_module_files(&mut modules, &[], &[], &hooks)
        .unwrap()
        .unwrap();
    let node = &tree.children["system"].children["etc"];
    assert_eq!(node.children["directory"].file_type, NodeFileType::Whiteout);
    assert!(node.children["directory"].children.is_empty());
    let real = root.path().join("real");
    let work = root.path().join("work");
    fs::create_dir_all(real.join("etc/directory")).unwrap();
    fs::write(real.join("etc/directory/child"), "untouched").unwrap();
    symlink("missing", real.join("etc/link")).unwrap();
    fs::create_dir_all(work.join("etc")).unwrap();
    let mounts = MountList::new(root.path().join("mounts")).unwrap();
    MagicMount::new(node, &real, &work, true, false, &mounts)
        .mount_path(true)
        .unwrap();
    assert!(work.join("etc/link").symlink_metadata().is_err());
    assert!(!work.join("etc/directory").exists());
    assert!(
        real.join("etc/link")
            .symlink_metadata()
            .unwrap()
            .is_symlink()
    );
    assert_eq!(
        fs::read_to_string(real.join("etc/directory/child")).unwrap(),
        "untouched"
    );
}

#[test]
fn replacement_sources_and_partition_targets_are_validated() {
    use std::os::unix::net::UnixListener;

    let root = tempfile::tempdir().unwrap();
    write_module(root.path(), "example", "example");
    let scripts = root.path().join("lua");
    fs::create_dir(&scripts).unwrap();
    let regular = root.path().join("regular");
    fs::write(&regular, "regular").unwrap();
    let link = root.path().join("link");
    symlink(&regular, &link).unwrap();
    let socket = root.path().join("socket");
    let _listener = UnixListener::bind(&socket).unwrap();
    for (source, target) in [
        (root.path().join("missing"), "system/etc/new"),
        (link, "system/etc/new"),
        (socket, "system/etc/new"),
        (regular, "system"),
    ] {
        fs::write(
            scripts.join("rules.lua"),
            format!(
                "API = 2\nMODULE = 'example'\nfunction load() replace('{}', '{target}') end",
                source.display()
            ),
        )
        .unwrap();
        let hooks = Extensions::load(&scripts, root.path()).unwrap();
        let mut modules = scan_modules(root.path());
        assert!(mount::collect_module_files(&mut modules, &[], &[], &hooks).is_err());
    }
    fs::write(
        scripts.join("rules.lua"),
        "API = 2\nMODULE = 'example'\nfunction load() remove('system') end",
    )
    .unwrap();
    let hooks = Extensions::load(&scripts, root.path()).unwrap();
    let mut modules = scan_modules(root.path());
    assert!(mount::collect_module_files(&mut modules, &[], &[], &hooks).is_err());
}

#[test]
fn directory_replace_discards_original_children_and_honors_descendant_rules() {
    let root = tempfile::tempdir().unwrap();
    let module = write_module(root.path(), "example", "example");
    fs::create_dir_all(module.join("system/etc/destination")).unwrap();
    fs::write(module.join("system/etc/destination/old"), "old").unwrap();
    let source = root.path().join("source");
    fs::create_dir(&source).unwrap();
    for name in ["keep", "blocked", "removed"] {
        fs::write(source.join(name), name).unwrap();
    }
    let scripts = root.path().join("lua");
    fs::create_dir(&scripts).unwrap();
    fs::write(scripts.join("rules.lua"), format!("API = 2\nMODULE = 'example'\nfunction load() replace('{}', 'system/etc/destination'); block('system/etc/destination/blocked'); replace('/missing', 'system/etc/destination/blocked'); remove('system/etc/destination/removed'); replace('/missing', 'system/etc/destination/removed') end", source.display())).unwrap();
    let hooks = Extensions::load(&scripts, root.path()).unwrap();
    let mut modules = scan_modules(root.path());
    let tree = mount::collect_module_files(&mut modules, &[], &[], &hooks)
        .unwrap()
        .unwrap();
    let destination = &tree.children["system"].children["etc"].children["destination"];
    assert!(destination.replace);
    assert!(!destination.children.contains_key("old"));
    assert!(!destination.children.contains_key("blocked"));
    assert_eq!(
        destination.children["keep"].module_path,
        Some(source.join("keep"))
    );
    assert!(destination.children["removed"].remove);
    assert_eq!(
        fs::read_to_string(module.join("system/etc/destination/old")).unwrap(),
        "old"
    );
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
