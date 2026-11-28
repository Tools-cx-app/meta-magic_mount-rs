// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::path::Path;

use mlua::{Function, Lua, MultiValue, Value};

use super::{Api, string_pair, validate_path};
use crate::errors::{Error, Result};
use crate::extension::core::Script;

#[derive(Debug, PartialEq, Eq)]
pub struct BindMount {
    pub source: String,
    pub target: String,
}

#[derive(Clone, Copy)]
pub enum BindPhase {
    Before,
    After,
}

pub(in crate::extension) struct ApiV1 {
    before: Option<Function>,
    after: Option<Function>,
}

impl Api for ApiV1 {
    const VERSION: u8 = 1;
    const UNAVAILABLE: &'static [&'static str] = &["load", "skip", "block", "replace", "remove"];

    fn callbacks(lua: &Lua) -> Result<Self> {
        Ok(Self {
            before: Self::callback(lua, "before")?,
            after: Self::callback(lua, "after")?,
        })
    }
}

impl Script<ApiV1> {
    pub(in crate::extension) fn bind_mounts(
        &self,
        phase: BindPhase,
        commands: &mut Vec<BindMount>,
    ) -> Result<()> {
        let (hook, stage) = match phase {
            BindPhase::Before => (&self.callbacks.before, "API 1 before"),
            BindPhase::After => (&self.callbacks.after, "API 1 after"),
        };
        let Some(hook) = hook else { return Ok(()) };
        self.lua
            .scope(|scope| {
                let bind = scope.create_function_mut(|_, args: MultiValue| {
                    let (source, target) = string_pair(args)?;
                    validate_path(&source).map_err(mlua::Error::external)?;
                    validate_path(&target).map_err(mlua::Error::external)?;
                    if !Path::new(&target).is_absolute() {
                        return Err(mlua::Error::external(Error::InvalidBindPath {
                            path: target,
                        }));
                    }
                    commands.push(BindMount { source, target });
                    Ok(())
                })?;
                self.lua.globals().raw_set("bind_mount", bind)?;
                let result = hook.call::<()>(());
                self.lua.globals().raw_set("bind_mount", Value::Nil)?;
                result
            })
            .map_err(|error| self.error(stage, None, error.into()))
    }
}
