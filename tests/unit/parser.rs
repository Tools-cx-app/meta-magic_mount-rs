// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use super::*;

fn module(id: &str, path: &Path) -> AppModule {
    serde_json::from_value(serde_json::json!({
        "id": id, "name": id, "version": "1", "author": "test", "description": "test",
        "mode": "magic", "is_mounted": true, "enabled": true,
        "source_path": path, "rules": {"default_mode": "magic", "paths": {}}
    }))
    .unwrap()
}

#[test]
fn directory_loads_all_scripts_in_order_with_independent_hooks() {
    let dir = tempfile::tempdir().unwrap();
    // Create in reverse order to distinguish filename order from discovery order.
    fs::write(
        dir.path().join("20-second.lua"),
        r#"
        assert(private == nil)
        local ran = false
        function before() ran = true end
        function after()
            assert(ran)
            bind('/second', '/system/second')
        end
    "#,
    )
    .unwrap();
    fs::write(
        dir.path().join("10-first.lua"),
        r#"
        private = 'first only'
        local ran = false
        function before() ran = true end
        function after()
            assert(ran and private == 'first only')
            bind('/first', '/system/first')
        end
    "#,
    )
    .unwrap();
    let parser = LuaParser::load(dir.path(), &[], dir.path()).unwrap();
    parser.before(&mut []).unwrap();
    assert_eq!(
        parser.after().unwrap(),
        vec![
            BindMount {
                source: "/first".into(),
                target: "/system/first".into()
            },
            BindMount {
                source: "/second".into(),
                target: "/system/second".into()
            },
        ]
    );
}

#[test]
fn directory_ignores_non_lua_nested_symlink_and_legacy_files() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("lua");
    fs::create_dir(&dir).unwrap();
    for path in [
        root.path().join("custom"),
        root.path().join("custom.lua"),
        dir.join("rules.txt"),
    ] {
        fs::write(path, "error('must not load')").unwrap();
    }
    fs::create_dir(dir.join("nested.lua")).unwrap();
    fs::write(
        dir.join("nested.lua/child.lua"),
        "error('must not recurse')",
    )
    .unwrap();
    std::os::unix::fs::symlink(root.path().join("custom.lua"), dir.join("alias.lua")).unwrap();
    fs::write(
        dir.join("hooks.lua"),
        "function after() bind('/valid', '/target') end",
    )
    .unwrap();
    let parser = LuaParser::load(&dir, &[], root.path()).unwrap();
    assert_eq!(
        parser.after().unwrap(),
        vec![BindMount {
            source: "/valid".into(),
            target: "/target".into()
        }]
    );
    assert!(LuaParser::load(&root.path().join("custom.lua"), &[], root.path()).is_err());
}

#[test]
fn missing_directory_is_optional_but_invalid_scripts_report_path() {
    let dir = tempfile::tempdir().unwrap();
    let parser = LuaParser::load(&dir.path().join("missing"), &[], dir.path()).unwrap();
    assert!(parser.before(&mut []).unwrap().is_empty());
    assert!(parser.after().unwrap().is_empty());
    let path = dir.path().join("broken.lua");
    fs::write(&path, "function before(").unwrap();
    let error = LuaParser::load(dir.path(), &[], dir.path()).err().unwrap();
    assert!(format!("{error:#}").contains(&path.display().to_string()));
    fs::write(&path, "before = false").unwrap();
    assert!(LuaParser::load(dir.path(), &[], dir.path()).is_err());
}

#[test]
fn before_skip_from_any_script_wins_and_updates_all_after_contexts() {
    let dir = tempfile::tempdir().unwrap();
    let first_path = dir.path().join("first");
    let second_path = dir.path().join("second");
    let mut modules = vec![module("first", &first_path), module("second", &second_path)];
    modules[1].skip_mount();
    fs::write(
        dir.path().join("10-skip.lua"),
        r#"
        function before() modules.first.skip = true end
        function after()
            assert(modules.first.skip and not modules.first.is_mounted)
            bind('/skipped', '/checked')
        end
    "#,
    )
    .unwrap();
    fs::write(
        dir.path().join("20-keep.lua"),
        r#"
        function before() modules.first.skip = false; modules.second.skip = false end
        function after()
            assert(modules.first.skip and modules.second.skip)
            bind('/markers', '/respected')
        end
    "#,
    )
    .unwrap();
    let parser = LuaParser::load(dir.path(), &modules, dir.path()).unwrap();
    let skipped = parser.before(&mut modules).unwrap();
    assert_eq!(skipped, FxHashSet::from_iter([first_path, second_path]));
    assert!(modules.iter().all(|module| !module.is_mountable()));
    assert_eq!(
        parser.after().unwrap(),
        vec![
            BindMount {
                source: "/skipped".into(),
                target: "/checked".into()
            },
            BindMount {
                source: "/markers".into(),
                target: "/respected".into()
            },
        ]
    );
}

#[test]
fn before_skip_excludes_module_files_without_persistent_markers() {
    let root = tempfile::tempdir().unwrap();
    for id in ["first", "second"] {
        let path = root.path().join(id);
        fs::create_dir_all(path.join("my_product/etc")).unwrap();
        fs::write(path.join("my_product/etc").join(id), id).unwrap();
        fs::write(
            path.join("module.prop"),
            format!("id={id}\nname={id}\nversion=1\nauthor=test\ndescription=test\n"),
        )
        .unwrap();
    }
    let dir = root.path().join("lua");
    fs::create_dir(&dir).unwrap();
    fs::write(
        dir.join("skip.lua"),
        "function before() assert(modules.first.author == 'test'); modules.first.skip = true end",
    )
    .unwrap();
    let partitions = vec!["my_product".to_string()];
    let mut modules = crate::scanner::list_modules(root.path(), &partitions);
    let parser = LuaParser::load(&dir, &modules, root.path()).unwrap();
    let skipped = parser.before(&mut modules).unwrap();
    let root_node =
        crate::magic_mount::collect_module_files(root.path(), &[], &partitions, &skipped)
            .unwrap()
            .unwrap();
    let files = &root_node.children["my_product"].children["etc"].children;
    assert!(!files.contains_key("first"));
    assert!(files.contains_key("second"));
    assert!(!root.path().join("first/skip_mount").exists());
}

#[test]
fn hook_errors_cancel_stage_and_identify_script() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("10-valid.lua"),
        "function after() bind('/source', '/target') end",
    )
    .unwrap();
    let path = dir.path().join("20-failure.lua");
    for body in [
        "function after() bind('/s', '/ok'); error('abort') end",
        "function after() bind('/s', '/system/../bad') end",
        "function after() bind('/s', 'relative') end",
        "function after() bind('', '/target') end",
        "function after() bind('/bad\\000path', '/target') end",
        "function after() ignore('/source') end",
    ] {
        fs::write(&path, body).unwrap();
        let parser = LuaParser::load(dir.path(), &[], dir.path()).unwrap();
        let error = parser.after().unwrap_err();
        assert!(format!("{error:#}").contains(&path.display().to_string()));
    }
    fs::write(&path, "function before() bind('/s', '/t') end").unwrap();
    let parser = LuaParser::load(dir.path(), &[], dir.path()).unwrap();
    let error = parser.before(&mut []).unwrap_err();
    assert!(format!("{error:#}").contains(&path.display().to_string()));
}

#[test]
fn hooks_cannot_access_process_or_file_apis() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("sandbox.lua"),
        r#"
        function before()
            assert(os == nil and io == nil and package == nil and debug == nil)
            assert(dofile == nil and loadfile == nil and bind == nil)
        end
        function after() bind('/safe', '/target') end
    "#,
    )
    .unwrap();
    let parser = LuaParser::load(dir.path(), &[], dir.path()).unwrap();
    parser.before(&mut []).unwrap();
    assert_eq!(
        parser.after().unwrap(),
        vec![BindMount {
            source: "/safe".into(),
            target: "/target".into()
        }]
    );
}
