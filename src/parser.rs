// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{
    fmt, fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use rustc_hash::FxHashSet;

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
            Self::Ignore { source } => f.write_str(&format!("ignored {source}")),
        }
    }
}

pub fn parser_custom<P>(path: P) -> Vec<MountType>
where
    P: AsRef<Path>,
{
    Parser::default().parse_file(path.as_ref())
}

#[cfg(test)]
fn parse(content: &str) -> Vec<MountType> {
    Parser::default().parse_content(content)
}

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Bind { source: String, target: String },
    Ignore { source: String },
    Include { path: String },
}

struct LexParser;

impl LexParser {
    fn tokenize(input: &str) -> Option<Vec<String>> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut in_quote: Option<char> = None;
        let mut started = false;

        for ch in input.chars() {
            match in_quote {
                Some(quote) => {
                    if ch == quote {
                        in_quote = None;
                    } else {
                        current.push(ch);
                    }
                }
                None => {
                    if ch == '\'' || ch == '"' {
                        in_quote = Some(ch);
                        started = true;
                    } else if ch.is_ascii_whitespace() {
                        if started {
                            tokens.push(std::mem::take(&mut current));
                            started = false;
                        }
                    } else {
                        current.push(ch);
                        started = true;
                    }
                }
            }
        }
        if in_quote.is_some() {
            return None;
        }
        if started {
            tokens.push(current);
        }
        Some(tokens)
    }
}

#[derive(Default)]
struct Parser {
    seen: FxHashSet<PathBuf>,
}

impl Parser {
    // ponytail: add new commands here and in parse_content; the lexer stays command-agnostic.
    fn parse_line(line: &str) -> Option<Command> {
        let tokens = LexParser::tokenize(line)?;
        let path = |index: usize| -> Option<String> {
            let value: String = tokens
                .get(index)?
                .chars()
                .filter(|c| !c.is_control())
                .collect();
            (!value.is_empty()).then_some(value)
        };
        match tokens.first()?.as_str() {
            "bind" => Some(Command::Bind {
                source: path(1)?,
                target: path(2)?,
            }),
            "ignore" => Some(Command::Ignore { source: path(1)? }),
            "file" | "add" => Some(Command::Include { path: path(1)? }),
            _ => None,
        }
    }

    fn parse_content(&mut self, content: &str) -> Vec<MountType> {
        let mut types = Vec::new();
        for line in content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
        {
            match Self::parse_line(line) {
                Some(Command::Bind { source, target }) => {
                    types.push(MountType::Mount { source, target })
                }
                Some(Command::Ignore { source }) => types.push(MountType::Ignore { source }),
                Some(Command::Include { path }) => types.extend(self.parse_file(Path::new(&path))),
                None => log::debug!("failed to parse {line}"),
            }
        }
        types
    }

    fn parse_file(&mut self, path: &Path) -> Vec<MountType> {
        let identity = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        if !self.seen.insert(identity) {
            log::warn!(
                "detected same file, skip {} for solving loop",
                path.display()
            );
            return Vec::new();
        }
        match fs::read_to_string(path) {
            Ok(content) => self.parse_content(&content),
            Err(error) => {
                log::warn!("failed to read {}: {error}", path.display());
                Vec::new()
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/parser.rs"]
mod tests;
