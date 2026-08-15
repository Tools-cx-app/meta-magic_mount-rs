// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::io::Write;

use super::*;

#[test]
fn parse_path_no_quotes() {
    assert_eq!(parse_path("/home/user"), "/home/user");
}

#[test]
fn parse_path_single_quotes() {
    assert_eq!(parse_path("'/path/with spaces'"), "/path/with spaces");
}

#[test]
fn parse_path_single_quote_does_not_panic() {
    assert_eq!(parse_path("'"), "'");
}

#[test]
fn parse_path_double_quotes() {
    assert_eq!(parse_path("\"/another/path\""), "/another/path");
}

#[test]
fn parse_path_mixed_quotes_removed() {
    assert_eq!(parse_path("'/mixed\""), "");
    assert_eq!(parse_path("\"/mixed'"), "");
}

#[test]
fn parse_path_control_chars_filtered() {
    assert_eq!(parse_path("/path\x00with\x01control"), "/pathwithcontrol");
}

#[test]
fn parse_bind_valid() {
    assert_eq!(
        parse_bind("bind /src /dst"),
        Some(MountType::Mount {
            source: "/src".to_string(),
            target: "/dst".to_string(),
        })
    );
}

#[test]
fn parse_bind_with_quoted_paths() {
    assert_eq!(
        parse_bind("bind '/source path' \"/target path\""),
        Some(MountType::Mount {
            source: "/source path".to_string(),
            target: "/target path".to_string(),
        })
    );
}

#[test]
fn parse_bind_missing_target() {
    assert_eq!(parse_bind("bind /src"), None);
}

#[test]
fn parse_bind_missing_source() {
    assert_eq!(parse_bind("bind"), None);
}

#[test]
fn parse_bind_extra_args_ignored() {
    assert_eq!(
        parse_bind("bind /src /dst extra"),
        Some(MountType::Mount {
            source: "/src".to_string(),
            target: "/dst".to_string(),
        })
    );
}

#[test]
fn parse_ignore_valid() {
    assert_eq!(
        parse_ignore("ignore /path/to/ignore"),
        Some(MountType::Ignore {
            source: "/path/to/ignore".to_string(),
        })
    );
}

#[test]
fn parse_ignore_missing_source() {
    assert_eq!(parse_ignore("ignore"), None);
}

#[test]
fn parse_ignore_with_quoted_path() {
    assert_eq!(
        parse_ignore("ignore '/quoted path'"),
        Some(MountType::Ignore {
            source: "/quoted path".to_string(),
        })
    );
}

#[test]
fn parse_multiple_commands() {
    let content = "
        # this is a comment
        bind /src /dst
        ignore /ignored

        # another comment
        bind '/app/data' '/mnt/data'
    ";
    let result = parse(content);
    assert_eq!(result.len(), 3);
    assert_eq!(
        result[0],
        MountType::Mount {
            source: "/src".to_string(),
            target: "/dst".to_string(),
        }
    );
    assert_eq!(
        result[1],
        MountType::Ignore {
            source: "/ignored".to_string(),
        }
    );
    assert_eq!(
        result[2],
        MountType::Mount {
            source: "/app/data".to_string(),
            target: "/mnt/data".to_string(),
        }
    );
}

#[test]
fn parse_empty_content() {
    assert_eq!(parse(""), Vec::new());
}

#[test]
fn parser_custom_file_valid() {
    let mut tempfile = tempfile::Builder::new().tempfile().unwrap();
    tempfile.write_all(b"bind /a /b\nignore /c\n").unwrap();
    let result = parser_custom(tempfile.path());
    assert_eq!(result.len(), 2);
    assert_eq!(
        result[0],
        MountType::Mount {
            source: "/a".to_string(),
            target: "/b".to_string(),
        }
    );
    assert_eq!(
        result[1],
        MountType::Ignore {
            source: "/c".to_string(),
        }
    );
}

#[test]
fn parser_custom_file_not_found() {
    assert_eq!(parser_custom("/nonexistent/path/to/file"), Vec::new());
}

#[test]
fn parse_file_valid() {
    let tempfile = tempfile::Builder::new().tempfile().unwrap();
    assert_eq!(
        parse_file(&format!("file {}", tempfile.path().to_string_lossy())),
        Some(tempfile.path().to_string_lossy().to_string())
    );
}

#[test]
fn parse_file_with_quoted_path() {
    assert_eq!(
        parse_file("file '/path with spaces'"),
        Some("/path with spaces".to_string())
    );
    assert_eq!(
        parse_file("file \"/another/path\""),
        Some("/another/path".to_string())
    );
}

#[test]
fn parse_file_missing_path() {
    assert_eq!(parse_file("file"), None);
}

#[test]
fn parse_file_empty_quoted_path() {
    assert_eq!(parse_file("file ''"), None);
}

#[test]
fn parse_file_add_keyword_supported() {
    assert_eq!(
        parse_file("add /path/to/config"),
        Some("/path/to/config".to_string())
    );
}

#[test]
fn parse_file_with_quoted_path_for_add() {
    assert_eq!(
        parse_file("add '/quoted path'"),
        Some("/quoted path".to_string())
    );
}

#[test]
fn parse_file_and_add_command_inclusion() {
    let mut temp = tempfile::Builder::new().tempfile().unwrap();
    temp.write_all(b"bind /a /b\nignore /c\n").unwrap();
    for command in ["file", "add"] {
        let main_content = format!("{command} {}", temp.path().to_str().unwrap());
        FILES.lock().clear();
        let result = parse(&main_content);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0],
            MountType::Mount {
                source: "/a".to_string(),
                target: "/b".to_string(),
            }
        );
        assert_eq!(
            result[1],
            MountType::Ignore {
                source: "/c".to_string(),
            }
        );
    }
}

#[test]
fn mount_type_display() {
    let mount = MountType::Mount {
        source: "s".into(),
        target: "t".into(),
    };
    assert_eq!(format!("{mount}"), "s -> t");
    assert_eq!(
        format!("{}", MountType::Ignore { source: "x".into() }),
        "missing x"
    );
}
