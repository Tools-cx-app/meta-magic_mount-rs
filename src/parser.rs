// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

use anyhow::Context;
use mlua::{Function, Lua, LuaOptions, LuaSerdeExt, StdLib, Table, Value, chunk::ChunkMode};
use rustc_hash::FxHashSet;

use crate::{
    errors::{Error, Result},
    scanner::AppModule,
};

#[derive(Debug, PartialEq, Eq)]
pub struct BindMount {
    pub source: String,
    pub target: String,
}

fn validate_bind(source: &str, target: &str) -> Result<()> {
    for path in [source, target] {
        if path.is_empty() || path.chars().any(char::is_control) {
            return Err(Error::InvalidBindPath {
                path: path.to_string(),
            });
        }
    }
    if !Path::new(target).is_absolute()
        || Path::new(target)
            .components()
            .any(|part| part == Component::ParentDir)
    {
        Err(Error::InvalidBindPath {
            path: target.to_string(),
        })
    } else {
        Ok(())
    }
}

struct LuaScript {
    path: PathBuf,
    lua: Lua,
    modules: Table,
    before: Option<Function>,
    after: Option<Function>,
}

impl LuaScript {
    fn load(path: PathBuf, modules: &[AppModule], module_dir: &Path) -> Result<Self> {
        let lua = Lua::new_with(
            StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8,
            LuaOptions::default(),
        )?;
        for name in ["dofile", "loadfile"] {
            lua.globals().set(name, Value::Nil)?;
        }
        let module_info = lua.create_table()?;
        for module in modules {
            let Value::Table(info) = lua.to_value(module)? else {
                unreachable!("AppModule serializes as a table");
            };
            info.set("skip", !module.is_mountable())?;
            module_info.set(module.id.as_str(), info)?;
        }
        lua.globals().set("modules", module_info.clone())?;
        lua.globals()
            .set("moduledir", module_dir.to_string_lossy().as_ref())?;
        let content = fs::read_to_string(&path)?;
        lua.load(&content)
            .set_name(format!("@{}", path.display()))
            .set_mode(ChunkMode::Text)
            .exec()?;
        let before = lua
            .globals()
            .get::<Option<Function>>("before")
            .context("before must be a function")?;
        let after = lua
            .globals()
            .get::<Option<Function>>("after")
            .context("after must be a function")?;
        Ok(Self {
            path,
            lua,
            modules: module_info,
            before,
            after,
        })
    }

    fn before(&self, modules: &mut [AppModule]) -> Result<()> {
        if let Some(before) = &self.before {
            before.call::<()>(())?;
        }
        for module in modules {
            let info: Table = self
                .modules
                .raw_get(module.id.as_str())
                .with_context(|| format!("missing module info: {}", module.id))?;
            match info.raw_get::<Value>("skip")? {
                Value::Boolean(true) => module.skip_mount(),
                Value::Boolean(false) => {}
                _ => log::warn!("modules[{:?}].skip must be a boolean", module.id),
            }
        }
        Ok(())
    }

    fn sync_modules(&self, modules: &[AppModule]) -> anyhow::Result<()> {
        for module in modules {
            let info: Table = self.modules.raw_get(module.id.as_str())?;
            info.raw_set("skip", !module.is_mountable())?;
            info.raw_set("is_mounted", module.is_mountable())?;
        }
        Ok(())
    }

    fn after(&self, commands: &mut Vec<BindMount>) -> anyhow::Result<()> {
        self.lua.scope(|scope| {
            let bind = scope.create_function_mut(|_, (source, target): (String, String)| {
                validate_bind(&source, &target).map_err(mlua::Error::external)?;
                commands.push(BindMount { source, target });
                Ok(())
            })?;
            self.lua.globals().set("bind", bind)?;
            let result = self
                .after
                .as_ref()
                .map_or(Ok(()), |after| after.call::<()>(()));
            self.lua.globals().set("bind", Value::Nil)?;
            result
        })?;
        Ok(())
    }
}

pub struct LuaParser {
    scripts: Vec<LuaScript>,
}

impl LuaParser {
    pub fn load(path: &Path, modules: &[AppModule], module_dir: &Path) -> anyhow::Result<Self> {
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Ok(Self {
                    scripts: Vec::new(),
                });
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("reading Lua directory {}", path.display()));
            }
        };
        let mut paths = Vec::new();
        for entry in entries {
            let entry =
                entry.with_context(|| format!("reading Lua directory {}", path.display()))?;
            let script = entry.path();
            if script
                .extension()
                .is_some_and(|extension| extension == "lua")
                && entry.file_type()?.is_file()
            {
                paths.push(script);
            }
        }
        paths.sort_unstable();
        let mut scripts = Vec::with_capacity(paths.len());
        for path in paths {
            let script = LuaScript::load(path.clone(), modules, module_dir)
                .with_context(|| format!("loading Lua hooks {}", path.display()))?;
            scripts.push(script);
        }
        Ok(Self { scripts })
    }

    pub fn before(&self, modules: &mut [AppModule]) -> anyhow::Result<FxHashSet<PathBuf>> {
        for script in &self.scripts {
            script
                .before(modules)
                .with_context(|| format!("Lua before hook failed: {}", script.path.display()))?;
        }
        let mut skipped = FxHashSet::default();
        for module in modules.iter() {
            if !module.is_mountable() {
                skipped.insert(module.mount_path().to_path_buf());
            }
        }
        for script in &self.scripts {
            script
                .sync_modules(modules)
                .with_context(|| format!("updating Lua module info: {}", script.path.display()))?;
        }
        Ok(skipped)
    }

    pub fn after(&self) -> anyhow::Result<Vec<BindMount>> {
        let mut commands = Vec::new();
        for script in &self.scripts {
            script
                .after(&mut commands)
                .with_context(|| format!("Lua after hook failed: {}", script.path.display()))?;
        }
        Ok(commands)
    }
}

#[cfg(test)]
#[path = "../tests/unit/parser.rs"]
mod tests;
