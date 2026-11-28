// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use mlua::{Lua, LuaOptions, StdLib, Table, Value, chunk::ChunkMode};

use super::Extensions;
use super::api::Api;
use super::header::{Header, header};
use crate::framework::error::{Error, Result};

fn script_error(path: &Path, phase: &'static str, module: Option<&str>, source: Error) -> Error {
    Error::LuaScript {
        path: path.into(),
        phase,
        module: module.map(str::to_owned),
        source: Box::new(source),
    }
}

pub(super) struct Script<A> {
    path: PathBuf,
    pub(super) lua: Lua,
    pub(super) callbacks: A,
}

impl<A> Script<A>
where
    A: Api,
{
    fn new(path: &Path, module_dir: &Path, module: Option<&str>, body: &str) -> Result<Self> {
        let lua = Lua::new_with(
            StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8,
            LuaOptions::default(),
        )?;
        // Keep metadata in __index so every assignment passes through __newindex.
        for name in ["dofile", "loadfile", "load", "rawset", "setmetatable"] {
            lua.globals().set(name, Value::Nil)?;
        }
        let constants = lua.create_table()?;
        constants.set("API", A::VERSION)?;
        constants.set("MODULE", module)?;
        lua.globals()
            .set("moduledir", module_dir.to_string_lossy().as_ref())?;
        let meta = lua.create_table()?;
        meta.set("__index", constants)?;
        meta.set("__metatable", false)?;
        meta.set(
            "__newindex",
            lua.create_function(move |_, (table, key, value): (Table, Value, Value)| {
                if let Value::String(name) = &key {
                    let name = name.to_str()?;
                    if matches!(name.as_ref(), "API" | "MODULE") {
                        return Err(mlua::Error::external(Error::LuaHeader {
                            field: "API/MODULE",
                            reason: "immutable header constant",
                        }));
                    }
                    if A::UNAVAILABLE.contains(&name.as_ref()) {
                        return Err(mlua::Error::external(Error::LuaContract {
                            api: A::VERSION,
                            function: name.to_string(),
                            reason: "unavailable in this API version",
                        }));
                    }
                }
                table.raw_set(key, value)
            })?,
        )?;
        lua.globals().set_metatable(Some(meta))?;
        lua.load(body)
            .set_name(format!("@{}", path.display()))
            .set_mode(ChunkMode::Text)
            .exec()?;
        let callbacks = A::callbacks(&lua)?;
        Ok(Self {
            path: path.to_owned(),
            lua,
            callbacks,
        })
    }

    pub(super) fn error(&self, phase: &'static str, module: Option<&str>, source: Error) -> Error {
        script_error(&self.path, phase, module, source)
    }
}

impl Extensions {
    pub fn load(directory: &Path, module_dir: &Path) -> Result<Self> {
        let mut extensions = Self::default();
        let paths = script_paths(directory)
            .map_err(|error| script_error(directory, "directory read", None, error.into()))?;
        for path in paths {
            extensions
                .load_script(&path, module_dir)
                .map_err(|error| script_error(&path, "loading", None, error))?;
        }
        Ok(extensions)
    }

    fn load_script(&mut self, path: &Path, module_dir: &Path) -> Result<()> {
        let content = fs::read_to_string(path)?;
        let (header, body) = header(&content)?;
        match header {
            Header::V1 => self
                .bind_scripts
                .push(Script::new(path, module_dir, None, &body)?),
            Header::V2 { module } => {
                let script = Script::new(path, module_dir, Some(module), &body)?;
                self.module_scripts
                    .entry(module.to_owned())
                    .or_default()
                    .push(script);
            }
        }
        Ok(())
    }
}

fn script_paths(directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|extension| extension == "lua")
            && entry.file_type()?.is_file()
        {
            paths.push(path);
        }
    }
    paths.sort_unstable();
    Ok(paths)
}
