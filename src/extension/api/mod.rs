// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

mod v1;
mod v2;

use std::path::{Component, Path};

use mlua::{Function, Lua, MultiValue, Value};

use crate::errors::{Error, Result};

pub(super) use v1::ApiV1;
pub use v1::{BindMount, BindPhase};
pub(super) use v2::ApiV2;
pub use v2::ModulePlan;

pub(super) trait Api: Sized {
    const VERSION: u8;
    const UNAVAILABLE: &'static [&'static str];

    fn callbacks(lua: &Lua) -> Result<Self>;

    fn callback(lua: &Lua, name: &str) -> Result<Option<Function>> {
        match lua.globals().get::<Value>(name)? {
            Value::Nil => Ok(None),
            Value::Function(function) => Ok(Some(function)),
            _ => Err(Error::LuaContract {
                api: Self::VERSION,
                function: name.into(),
                reason: "expected a function",
            }),
        }
    }
}

fn validate_path(path: &str) -> Result<()> {
    if path.is_empty() || path.chars().any(char::is_control) {
        return Err(Error::RulePath {
            path: path.into(),
            reason: "empty path or control character",
        });
    }
    if Path::new(path)
        .components()
        .any(|part| part == Component::ParentDir)
    {
        return Err(Error::RulePath {
            path: path.into(),
            reason: "parent components are forbidden",
        });
    }
    Ok(())
}

fn string_pair(mut args: MultiValue) -> mlua::Result<(String, String)> {
    match (args.pop_front(), args.pop_front(), args.pop_front()) {
        (Some(Value::String(source)), Some(Value::String(target)), None) => {
            Ok((source.to_str()?.to_owned(), target.to_str()?.to_owned()))
        }
        _ => Err(mlua::Error::runtime("expected two path strings")),
    }
}
