// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use crate::framework::error::{Error, Result};
use crate::framework::scanner::validate_module_id;

pub(super) enum Header<'a> {
    V1,
    V2 { module: &'a str },
}

/// Skips Lua whitespace and comments without interpreting strings as comments.
fn skip_comments(mut text: &str) -> Result<&str> {
    loop {
        text = text.trim_start();
        let Some(comment) = text.strip_prefix("--") else {
            return Ok(text);
        };
        if let Some(long) = comment.strip_prefix('[') {
            let equals = long.bytes().take_while(|&byte| byte == b'=').count();
            if long.as_bytes().get(equals) == Some(&b'[') {
                let end = format!("]{}]", "=".repeat(equals));
                let content = &long[equals + 1..];
                let offset = content.find(&end).ok_or(Error::LuaHeader {
                    field: "comment",
                    reason: "unclosed long comment",
                })?;
                text = &content[offset + end.len()..];
                continue;
            }
        }
        text = comment.split_once('\n').map_or("", |(_, rest)| rest);
    }
}

/// Reads one literal declaration and returns the remaining lines.
fn declaration<'a>(text: &'a str, name: &'static str) -> Result<(&'a str, &'a str)> {
    let text = skip_comments(text)?;
    let (line, rest) = text.split_once('\n').unwrap_or((text, ""));
    let (key, value) = line.split_once('=').ok_or(Error::LuaHeader {
        field: name,
        reason: "expected a literal declaration at file header",
    })?;
    if key.trim() != name {
        return Err(Error::LuaHeader {
            field: name,
            reason: "expected declaration at file header",
        });
    }
    Ok((value.trim(), rest))
}

/// Replaces consumed header bytes with whitespace to preserve Lua line numbers.
pub(super) fn header(content: &str) -> Result<(Header<'_>, String)> {
    let (api, mut rest) = declaration(content, "API")?;
    let header = match api {
        "1" => Header::V1,
        "2" => {
            let (literal, remaining) = declaration(rest, "MODULE")?;
            rest = remaining;
            let quote = literal.as_bytes().first().copied();
            if !(matches!(quote, Some(b'\'' | b'"'))
                && literal.len() >= 2
                && literal.as_bytes().last().copied() == quote)
            {
                return Err(Error::LuaHeader {
                    field: "MODULE",
                    reason: "expected a quoted literal module ID",
                });
            }
            let id = &literal[1..literal.len() - 1];
            validate_module_id(id)?;
            Header::V2 { module: id }
        }
        _ => return Err(Error::LuaApi { value: api.into() }),
    };
    let prefix = &content[..content.len() - rest.len()];
    let mut body: String = prefix
        .chars()
        .map(|c| if c == '\n' { '\n' } else { ' ' })
        .collect();
    body.push_str(rest);
    Ok((header, body))
}
