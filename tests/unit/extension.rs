// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::fs;

use super::*;
use crate::framework::{mount::collect_module_files, scanner::scan_modules};

#[test]
fn lua55_global_function_runs_as_a_mount_hook() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("global.lua"),
        "API = 1
global bind_mount
global function before() bind_mount('/source', '/target') end",
    )
    .unwrap();
    let extensions = Extensions::load(dir.path(), dir.path()).unwrap();
    assert_eq!(
        extensions.bind_mounts(BindPhase::Before).unwrap(),
        [BindMount {
            source: "/source".into(),
            target: "/target".into(),
        }]
    );
}

#[test]
fn api2_skip_runs_before_module_content_scan() {
    let root = tempfile::tempdir().unwrap();
    let module = root.path().join("different-directory");
    fs::create_dir(&module).unwrap();
    fs::write(
        module.join("module.prop"),
        "id=example\nname=Example\nversion=1\nauthor=test\ndescription=test\n",
    )
    .unwrap();
    std::os::unix::fs::symlink("system", module.join("system")).unwrap();
    let scripts = root.path().join("lua");
    fs::create_dir(&scripts).unwrap();
    fs::write(scripts.join("10-skip.lua"), "API = 2\nMODULE = 'example'\nfunction load() remove('system/app/Gone'); skip(); skip() end").unwrap();
    fs::write(
        scripts.join("20-unreachable.lua"),
        "API = 2\nMODULE = 'example'\nfunction load() error('must not run') end",
    )
    .unwrap();
    let mut modules = scan_modules(root.path());
    let hooks = Extensions::load(&scripts, root.path()).unwrap();
    assert!(
        collect_module_files(root.path(), &mut modules, &[], &[], &hooks)
            .unwrap()
            .is_none()
    );
    assert!(!modules[0].is_mountable());
    assert!(!module.join("skip_mount").exists());
}

#[test]
fn directory_loads_all_scripts_in_order_with_independent_hooks() {
    let dir = tempfile::tempdir().unwrap();
    // Create in reverse order to distinguish filename order from discovery order.
    fs::write(
        dir.path().join("20-second.lua"),
        r#"
API = 1
        assert(private == nil)
        local ran = false
        function before() ran = true end
        function after()
            assert(ran)
            bind_mount('/second', '/system/second')
        end
    "#,
    )
    .unwrap();
    fs::write(
        dir.path().join("10-first.lua"),
        r#"
API = 1
        private = 'first only'
        local ran = false
        function before() ran = true end
        function after()
            assert(ran and private == 'first only')
            bind_mount('/first', '/system/first')
        end
    "#,
    )
    .unwrap();
    let extensions = Extensions::load(dir.path(), dir.path()).unwrap();
    extensions.bind_mounts(BindPhase::Before).unwrap();
    assert_eq!(
        extensions.bind_mounts(BindPhase::After).unwrap(),
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
        "API = 1\nfunction after() bind_mount('/valid', '/target') end",
    )
    .unwrap();
    let extensions = Extensions::load(&dir, root.path()).unwrap();
    assert_eq!(
        extensions.bind_mounts(BindPhase::After).unwrap(),
        vec![BindMount {
            source: "/valid".into(),
            target: "/target".into()
        }]
    );
    assert!(Extensions::load(&root.path().join("custom.lua"), root.path()).is_err());
}

#[test]
fn missing_directory_is_optional_but_invalid_scripts_report_path() {
    let dir = tempfile::tempdir().unwrap();
    let extensions = Extensions::load(&dir.path().join("missing"), dir.path()).unwrap();
    assert!(
        extensions
            .bind_mounts(BindPhase::Before)
            .unwrap()
            .is_empty()
    );
    assert!(extensions.bind_mounts(BindPhase::After).unwrap().is_empty());
    let path = dir.path().join("broken.lua");
    fs::write(&path, "API = 1\nfunction before(").unwrap();
    let error = Extensions::load(dir.path(), dir.path()).err().unwrap();
    assert!(format!("{error:#}").contains(&path.display().to_string()));
    fs::write(&path, "API = 1\nbefore = false").unwrap();
    assert!(Extensions::load(dir.path(), dir.path()).is_err());
}

#[test]
fn hook_errors_cancel_stage_and_identify_script() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("10-valid.lua"),
        "API = 1\nfunction after() bind_mount('/source', '/target') end",
    )
    .unwrap();
    let path = dir.path().join("20-failure.lua");
    for body in [
        "API = 1\nfunction after() bind_mount('/s', '/ok'); error('abort') end",
        "API = 1\nfunction after() bind_mount('/s', '/system/../bad') end",
        "API = 1\nfunction after() bind_mount('/s', 'relative') end",
        "API = 1\nfunction after() bind_mount('', '/target') end",
        "API = 1\nfunction after() bind_mount('/bad\\000path', '/target') end",
        "API = 1\nfunction after() bind_mount('', '') end",
        "API = 1\nfunction after() bind_mount('/bad\\000path', 'relative') end",
        "API = 1\nfunction after() bind_mount('/s', '/bad\\npath') end",
        "API = 1\nfunction after() bind_mount('/s', '/bad\\000path') end",
        "API = 1\nfunction after() ignore('/source') end",
    ] {
        fs::write(&path, body).unwrap();
        let extensions = Extensions::load(dir.path(), dir.path()).unwrap();
        let error = extensions.bind_mounts(BindPhase::After).unwrap_err();
        assert!(
            format!("{error:#}").contains(&path.display().to_string()),
            "wrong failing script for {body:?}: {error:#}"
        );
    }
    fs::write(
        &path,
        "API = 1\nfunction before() block('system/etc/a') end",
    )
    .unwrap();
    let extensions = Extensions::load(dir.path(), dir.path()).unwrap();
    let error = extensions.bind_mounts(BindPhase::Before).unwrap_err();
    assert!(format!("{error:#}").contains(&path.display().to_string()));
}

#[test]
fn hooks_cannot_access_process_or_file_apis() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("sandbox.lua"),
        r#"
API = 1
        function before()
            assert(os == nil and io == nil and package == nil and debug == nil)
            assert(dofile == nil and loadfile == nil and type(bind_mount) == "function")
        end
        function after() bind_mount('/safe', '/target') end
    "#,
    )
    .unwrap();
    let extensions = Extensions::load(dir.path(), dir.path()).unwrap();
    extensions.bind_mounts(BindPhase::Before).unwrap();
    assert_eq!(
        extensions.bind_mounts(BindPhase::After).unwrap(),
        vec![BindMount {
            source: "/safe".into(),
            target: "/target".into()
        }]
    );
}

#[test]
fn headers_versions_constants_and_callbacks_are_checked() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("script.lua");
    for content in [
        "function after() end",
        "API = 3",
        "API = '1'",
        "API = 1 + 0",
        "-- API = 1\nfunction after() end",
        "API = 2\nfunction load() end",
        "API = 2\nMODULE = '../bad'\nfunction load() end",
        "API = 2\nMODULE = 'example'",
        "API = 2\nMODULE = 'example'\nload = false",
        "API = 2\nMODULE = 'example'\nfunction load(arg) end",
        "API = 2\nMODULE = 'example'\nfunction before() end\nfunction load() end",
        "API = 1\nfunction load() end",
        "API = 1\nAPI = 2",
        "API = 1\nMODULE = 'example'",
        "API = 1\nrawset(_G, 'API', 2)",
        "API = 1\nbind_mount('/a', '/b')",
        "API = 2\nMODULE = 'example'\nskip()\nfunction load() end",
    ] {
        fs::write(&path, content).unwrap();
        let error = Extensions::load(dir.path(), dir.path())
            .err()
            .unwrap_or_else(|| panic!("accepted invalid script: {content}"));
        assert!(format!("{error:#}").contains(&path.display().to_string()));
    }
    fs::write(
        &path,
        "--[=[\nAPI = 999\n]=]\nAPI = 1\n-- comment\nfunction before() assert(API == 1) end",
    )
    .unwrap();
    Extensions::load(dir.path(), dir.path())
        .unwrap()
        .bind_mounts(BindPhase::Before)
        .unwrap();
}

#[test]
fn api1_collects_separate_before_and_after_queues() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("hooks.lua"), "API = 1\nfunction before() bind_mount('/before', '/target') end\nfunction after() assert(bind == nil and skip == nil and modules == nil); bind_mount('/after', '/target') end").unwrap();
    let extensions = Extensions::load(dir.path(), dir.path()).unwrap();
    assert_eq!(
        extensions.bind_mounts(BindPhase::Before).unwrap()[0].source,
        "/before"
    );
    assert_eq!(
        extensions.bind_mounts(BindPhase::After).unwrap()[0].source,
        "/after"
    );
}

#[test]
fn api2_validates_path_arguments_and_only_runs_for_eligible_modules() {
    let root = tempfile::tempdir().unwrap();
    let module = root.path().join("example");
    fs::create_dir(&module).unwrap();
    fs::write(
        module.join("module.prop"),
        "id=example\nname=Example\nversion=1\nauthor=test\ndescription=test\n",
    )
    .unwrap();
    let scripts = root.path().join("lua");
    fs::create_dir(&scripts).unwrap();
    let path = scripts.join("rules.lua");
    for body in [
        "block('/system/a')",
        "remove('../system/a')",
        "block('')",
        "block('.')",
        "block('system/a/../../b')",
        "remove('other/a')",
        "remove('system/a\\000b')",
        "block(1)",
        "block('system/a', 'system/b')",
        "replace('relative', 'system/a')",
        "replace('/source', '/system/a')",
        "replace('/source', 'system/a', 'extra')",
        "skip(true)",
        "bind_mount('/source', '/system/a')",
        "API = 1",
        "MODULE = 'other'",
    ] {
        fs::write(
            &path,
            format!("API = 2\nMODULE = 'example'\nfunction load() {body} end"),
        )
        .unwrap();
        let extensions = Extensions::load(&scripts, root.path()).unwrap();
        let mut modules = scan_modules(root.path());
        let error =
            collect_module_files(root.path(), &mut modules, &[], &[], &extensions).unwrap_err();
        assert!(
            format!("{error:#}").contains(&path.display().to_string()),
            "{body}: {error:#}"
        );
    }
    fs::write(
        &path,
        "API = 2\nMODULE = 'example'\nfunction load() error('must not run') end",
    )
    .unwrap();
    for marker in ["disable", "remove", "skip_mount"] {
        fs::write(module.join(marker), "").unwrap();
        let extensions = Extensions::load(&scripts, root.path()).unwrap();
        let mut modules = scan_modules(root.path());
        assert!(
            collect_module_files(root.path(), &mut modules, &[], &[], &extensions)
                .unwrap()
                .is_none()
        );
        fs::remove_file(module.join(marker)).unwrap();
    }
    fs::write(
        &path,
        "API = 2\nMODULE = 'not_installed'\nfunction load() error('must not run') end",
    )
    .unwrap();
    let extensions = Extensions::load(&scripts, root.path()).unwrap();
    let mut modules = scan_modules(root.path());
    assert!(
        collect_module_files(root.path(), &mut modules, &[], &[], &extensions)
            .unwrap()
            .is_none()
    );
}

#[test]
fn domain_errors_remain_typed_with_script_context() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("bad.lua"), "API = 3").unwrap();
    let error = Extensions::load(dir.path(), dir.path()).err().unwrap();
    match error {
        Error::LuaScript { source, .. } => assert!(matches!(*source, Error::LuaApi { .. })),
        error => panic!("missing typed script context: {error}"),
    }
    fs::write(
        dir.path().join("bad.lua"),
        "API = 1\nfunction before() bind_mount('../escape', '/target') end",
    )
    .unwrap();
    let extensions = Extensions::load(dir.path(), dir.path()).unwrap();
    let error = extensions.bind_mounts(BindPhase::Before).unwrap_err();
    let Error::LuaScript { source, .. } = error else {
        panic!("missing script context: {error}");
    };
    let Error::Mlua(error) = *source else {
        panic!("missing callback error: {source}");
    };
    assert!(
        matches!(error.downcast_ref::<Error>(), Some(Error::RulePath { path, .. }) if path == "../escape")
    );
}

#[test]
fn skip_cancels_all_module_rules_and_does_not_persist_between_boots() {
    let root = tempfile::tempdir().unwrap();
    for id in ["example", "other"] {
        let module = root.path().join(id);
        fs::create_dir_all(module.join("my_product/etc")).unwrap();
        fs::write(module.join("my_product/etc/content"), id).unwrap();
        fs::write(
            module.join("module.prop"),
            format!("id={id}\nname={id}\nversion=1\nauthor=test\ndescription=test\n"),
        )
        .unwrap();
    }
    let scripts = root.path().join("lua");
    fs::create_dir(&scripts).unwrap();
    fs::write(scripts.join("10-rules.lua"), "API = 2\nMODULE = 'example'\nfunction load() replace('/missing-source', 'my_product/etc/content'); remove('my_product/etc/content'); block('my_product/etc/blocked') end").unwrap();
    fs::write(scripts.join("20-skip.lua"), "API = 2\nMODULE = 'example'\nfunction load() skip(); replace('/another-missing', 'my_product/ignored'); remove('my_product/etc/content') end").unwrap();
    let hooks = Extensions::load(&scripts, root.path()).unwrap();
    let mut modules = scan_modules(root.path());
    let extra = vec!["my_product".into()];
    let tree = collect_module_files(root.path(), &mut modules, &[], &extra, &hooks)
        .unwrap()
        .unwrap();
    assert!(!modules[0].is_mountable());
    assert!(modules[1].is_mountable());
    let content = &tree.children["my_product"].children["etc"].children["content"];
    assert_eq!(
        content.module_path,
        Some(root.path().join("other/my_product/etc/content"))
    );
    let snapshot = serde_json::to_value(&modules).unwrap();
    assert_eq!(snapshot[0]["rules"]["paths"], serde_json::json!({}));
    assert_eq!(snapshot[0]["mode"], "ignore");
    assert!(!root.path().join("example/skip_mount").exists());
    fs::write(
        scripts.join("10-rules.lua"),
        "API = 2\nMODULE = 'example'\nfunction load() end",
    )
    .unwrap();
    fs::write(
        scripts.join("20-skip.lua"),
        "API = 2\nMODULE = 'example'\nfunction load() end",
    )
    .unwrap();
    let hooks = Extensions::load(&scripts, root.path()).unwrap();
    let mut modules = scan_modules(root.path());
    collect_module_files(root.path(), &mut modules, &[], &extra, &hooks).unwrap();
    assert!(modules[0].is_mountable());
}
