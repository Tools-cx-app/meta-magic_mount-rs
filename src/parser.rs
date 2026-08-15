// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{fmt, fs, path::Path, sync::OnceLock};

use parking_lot::Mutex;

pub static COMMAND_LIST: OnceLock<Vec<MountType>> = OnceLock::new();
static FILES: Mutex<Vec<String>> = Mutex::new(Vec::new());

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
        } else if line.starts_with("file") || line.starts_with("add") {
            match parse_file(line) {
                Some(s) => {
                    if FILES.lock().contains(&s) {
                        log::warn!("detected same file, skip {line} for solving loop");
                    } else {
                        log::debug!("new file: {s}");
                        FILES.lock().push(s.clone());
                        match fs::read_to_string(&s) {
                            Ok(s) => types.extend(parse(&s)),
                            Err(e) => log::warn!("failed to read {s}: {e}"),
                        }
                    }
                }
                _ => {
                    log::debug!("failed to parse {line}");
                }
            }
        }
    }

    types
}

fn parse_path(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }
    let first = input.as_bytes()[0] as char;
    let last = input.as_bytes()[input.len() - 1] as char;

    let strings = if (first == '\'' && last == '"') || (first == '"' && last == '\'') {
        log::error!("mixed quotes detected in path: {input}");
        String::new()
    } else if input.len() > 1 && (first == '\'' || first == '"') && first == last {
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

fn parse_file(input: &str) -> Option<String> {
    let tokens = tokenize(input);
    if tokens.len() < 2 || (tokens[0] != "file" && tokens[0] != "add") {
        return None;
    }
    let path = parse_path(&tokens[1]);
    if path.is_empty() {
        log::debug!("missing path, skip");
        None
    } else {
        Some(path)
    }
}

#[cfg(test)]
#[path = "../tests/unit/parser.rs"]
mod tests;
