use std::{fmt, fs, path::Path, sync::OnceLock};

pub static COMMAND_LIST: OnceLock<Vec<MountType>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MountType {
    Mount { source: String, target: String },
    Ignore { source: String },
}

impl fmt::Display for MountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mount { source, target } => f.write_str(&format!("{source} -> {target}")),
            Self::Ignore { source } => f.write_str(&format!("missing {source}")),
        }
    }
}

pub fn parser_custom<P>(path: P) -> Vec<MountType>
where
    P: AsRef<Path>,
{
    fs::read_to_string(path.as_ref()).map_or_else(|_| Vec::new(), |s| parse(&s))
}

fn parse(content: &str) -> Vec<MountType> {
    let mut types = Vec::new();
    for line in content.lines() {
        let line = line.trim();

        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        if line.starts_with("bind") {
            match parse_bind(line) {
                Some(s) => {
                    log::debug!("new bind command: {s}");
                    types.push(s);
                }
                None => {
                    log::debug!("failed to parse {line}");
                }
            }
        } else if line.starts_with("ignore") {
            match parse_ignore(line) {
                Some(s) => {
                    log::debug!("new bind command: {s}");
                    types.push(s);
                }
                None => {
                    log::debug!("failed to parse {line}");
                }
            }
        }
    }

    types
}

fn parse_path(input: &str) -> String {
    let first = input.as_bytes()[0] as char;
    let last = input.as_bytes()[input.len() - 1] as char;

    let strings =
        if (first == '\'' && last == '\"') || (first == '\"' && last == '\'') || first == last {
            input[1..input.len() - 1].to_string()
        } else {
            input.to_string()
        };

    strings.chars().filter(|c| !c.is_control()).collect()
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote: Option<char> = None;

    for ch in input.chars() {
        match in_quote {
            Some(quote) => {
                current.push(ch);
                if ch == quote {
                    in_quote = None;
                }
            }
            None => {
                if ch == '\'' || ch == '"' {
                    in_quote = Some(ch);
                    current.push(ch);
                } else if ch.is_ascii_whitespace() {
                    if !current.is_empty() {
                        tokens.push(std::mem::take(&mut current));
                    }
                } else {
                    current.push(ch);
                }
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn parse_bind(input: &str) -> Option<MountType> {
    let tokens = tokenize(input);

    if tokens.len() < 3 || tokens[0] != "bind" {
        return None;
    }
    let source = parse_path(&tokens[1]);
    let target = parse_path(&tokens[2]);
    if source.is_empty() || target.is_empty() {
        log::debug!("missing source/target, skip");
        None
    } else {
        Some(MountType::Mount { source, target })
    }
}

fn parse_ignore(input: &str) -> Option<MountType> {
    let tokens = tokenize(input);
    if tokens.len() < 2 || tokens[0] != "ignore" {
        return None;
    }
    let source = parse_path(&tokens[1]);
    if source.is_empty() {
        log::debug!("missing source, skip");
        None
    } else {
        Some(MountType::Ignore { source })
    }
}

#[cfg(test)]
mod tests {
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
    fn parse_path_double_quotes() {
        assert_eq!(parse_path("\"/another/path\""), "/another/path");
    }

    #[test]
    fn parse_path_mixed_quotes_removed() {
        assert_eq!(parse_path("'/mixed\""), "/mixed");
        assert_eq!(parse_path("\"/mixed'"), "/mixed");
    }

    #[test]
    fn parse_path_control_chars_filtered() {
        let input = "/path\x00with\x01control";
        let expected = "/pathwithcontrol";
        assert_eq!(parse_path(input), expected);
    }

    #[test]
    fn parse_bind_valid() {
        let cmd = "bind /src /dst";
        let expected = Some(MountType::Mount {
            source: "/src".to_string(),
            target: "/dst".to_string(),
        });
        assert_eq!(parse_bind(cmd), expected);
    }

    #[test]
    fn parse_bind_with_quoted_paths() {
        let cmd = "bind '/source path' \"/target path\"";
        let expected = Some(MountType::Mount {
            source: "/source path".to_string(),
            target: "/target path".to_string(),
        });
        assert_eq!(parse_bind(cmd), expected);
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
        let cmd = "bind /src /dst extra";
        let expected = Some(MountType::Mount {
            source: "/src".to_string(),
            target: "/dst".to_string(),
        });
        assert_eq!(parse_bind(cmd), expected);
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
        let cmd = "ignore '/quoted path'";
        assert_eq!(
            parse_ignore(cmd),
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
        let result = parse("");
        assert!(result.is_empty());
    }

    #[test]
    fn parser_custom_file_valid() {
        let content = "bind /a /b\nignore /c\n";
        let mut tempfile = tempfile::Builder::new().tempfile().unwrap();
        tempfile.write_all(content.as_bytes()).unwrap();

        let result = parser_custom(&tempfile.path());
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
        let result = parser_custom("/nonexistent/path/to/file");
        assert!(result.is_empty());
    }

    #[test]
    fn mount_type_display() {
        let m = MountType::Mount {
            source: "s".into(),
            target: "t".into(),
        };
        assert_eq!(format!("{}", m), "s -> t");

        let i = MountType::Ignore { source: "x".into() };
        assert_eq!(format!("{}", i), "missing x");
    }
}
