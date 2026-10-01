// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use super::*;

#[test]
fn lexer_handles_spaces_quotes_and_empty_tokens() {
    assert_eq!(LexParser::tokenize(""), Some(vec![]));
    assert_eq!(
        LexParser::tokenize("bind '/source path' \"/target path\""),
        Some(vec![
            "bind".into(),
            "/source path".into(),
            "/target path".into()
        ])
    );
    assert_eq!(
        LexParser::tokenize("ignore ''"),
        Some(vec!["ignore".into(), "".into()])
    );
    assert_eq!(LexParser::tokenize("bind '/broken /dst"), None);
}

#[test]
fn parse_bind_valid() {
    assert_eq!(
        Parser::parse_line("bind /src /dst extra"),
        Some(Command::Bind {
            source: "/src".into(),
            target: "/dst".into()
        })
    );
    assert_eq!(
        Parser::parse_line("bind '/source path' \"/target path\""),
        Some(Command::Bind {
            source: "/source path".into(),
            target: "/target path".into()
        })
    );
    assert_eq!(Parser::parse_line("bind /src"), None);
    assert_eq!(Parser::parse_line("binder /src /dst"), None);
    assert_eq!(Parser::parse_line("bind /src /dst 'broken"), None);
}

#[test]
fn parse_ignore_and_include() {
    assert_eq!(
        Parser::parse_line("ignore '/quoted path'"),
        Some(Command::Ignore {
            source: "/quoted path".into()
        })
    );
    assert_eq!(
        Parser::parse_line("file '/path with spaces'"),
        Some(Command::Include {
            path: "/path with spaces".into()
        })
    );
    assert_eq!(
        Parser::parse_line("add /path"),
        Some(Command::Include {
            path: "/path".into()
        })
    );
    assert_eq!(Parser::parse_line("ignore"), None);
    assert_eq!(Parser::parse_line("file ''"), None);
}

#[test]
fn parse_multiple_commands_and_skip_invalid_lines() {
    assert_eq!(
        parse("# comment\nbind /src /dst\nignore /ignored\nbind '/app/data' '/mnt/data'\n"),
        vec![
            MountType::Mount {
                source: "/src".into(),
                target: "/dst".into()
            },
            MountType::Ignore {
                source: "/ignored".into()
            },
            MountType::Mount {
                source: "/app/data".into(),
                target: "/mnt/data".into()
            },
        ]
    );
    assert!(parse("").is_empty());
    assert_eq!(
        parse(
            "binder /bad /bad\nbind '/broken /bad\nbind '/mixed\" /bad\nbind '' /bad\nignore ''\nbind /src /dst"
        ),
        vec![MountType::Mount {
            source: "/src".into(),
            target: "/dst".into()
        }]
    );
    assert_eq!(
        parse("ignore /path\x00with\x01control"),
        vec![MountType::Ignore {
            source: "/pathwithcontrol".into()
        }]
    );
}

#[test]
fn parser_custom_file_not_found() {
    assert!(parser_custom("/nonexistent/path/to/file").is_empty());
}

#[test]
fn parser_custom_repeated_calls_have_independent_includes() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root");
    let child = dir.path().join("child");
    fs::write(&child, "bind /a /b\n").unwrap();
    fs::write(&root, format!("file {}\n", child.display())).unwrap();
    let expected = vec![MountType::Mount {
        source: "/a".into(),
        target: "/b".into(),
    }];
    assert_eq!(parser_custom(&root), expected);
    assert_eq!(parser_custom(&root), expected);
}

#[test]
fn parser_custom_skips_duplicate_and_cyclic_includes() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root");
    let child = dir.path().join("child");
    fs::write(
        &root,
        format!(
            "bind /first /one\nfile {}\nadd {}\nbind /last /end\n",
            child.display(),
            child.display()
        ),
    )
    .unwrap();
    fs::write(&child, format!("ignore /inside\nfile {}\n", root.display())).unwrap();
    assert_eq!(
        parser_custom(&root),
        vec![
            MountType::Mount {
                source: "/first".into(),
                target: "/one".into()
            },
            MountType::Ignore {
                source: "/inside".into()
            },
            MountType::Mount {
                source: "/last".into(),
                target: "/end".into()
            },
        ]
    );
}

#[test]
fn parser_custom_canonicalizes_include_aliases() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root");
    let child = dir.path().join("child");
    fs::write(&child, "ignore /once\n").unwrap();
    fs::write(
        &root,
        format!(
            "file {}\nadd {}\n",
            child.display(),
            dir.path().join("./child").display()
        ),
    )
    .unwrap();
    assert_eq!(
        parser_custom(&root),
        vec![MountType::Ignore {
            source: "/once".into()
        }]
    );
}

#[test]
fn mount_type_display() {
    assert_eq!(
        MountType::Mount {
            source: "s".into(),
            target: "t".into()
        }
        .to_string(),
        "s -> t"
    );
    assert_eq!(
        MountType::Ignore { source: "x".into() }.to_string(),
        "ignored x"
    );
}
