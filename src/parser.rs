// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{
    cell::RefCell,
    collections::BTreeMap,
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

use mlua::{Function, Lua, LuaOptions, MultiValue, StdLib, Table, Value, chunk::ChunkMode};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    errors::{Error, Result},
    scanner::AppModule,
    utils::validate_module_id,
};

#[derive(Debug, PartialEq, Eq)]
pub struct BindMount {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Default)]
pub struct ModulePlan {
    pub skip: bool,
    pub blocked: FxHashSet<PathBuf>,
    pub removed: FxHashSet<PathBuf>,
    pub replacements: BTreeMap<PathBuf, PathBuf>,
}

impl ModulePlan {
    pub fn blocks(&self, path: &Path) -> bool {
        self.blocked.iter().any(|parent| path.starts_with(parent))
    }

    pub fn removes(&self, path: &Path) -> bool {
        self.removed.iter().any(|parent| path.starts_with(parent))
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

fn relative_path(path: &str, extra_mount: &[String]) -> Result<PathBuf> {
    validate_path(path)?;
    let normalized: PathBuf = Path::new(path)
        .components()
        .filter(|part| *part != Component::CurDir)
        .collect();
    let Some(Component::Normal(partition)) = normalized.components().next() else {
        return Err(Error::RulePath {
            path: path.into(),
            reason: "expected a module-relative partition path",
        });
    };
    if partition != "system" && !extra_mount.iter().any(|name| partition == name.as_str()) {
        return Err(Error::RulePath {
            path: path.into(),
            reason: "partition is not configured for mounting",
        });
    }
    Ok(normalized)
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

/// Reads a literal header declaration and leaves its newline in the script body.
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
fn header(content: &str) -> Result<(u8, Option<String>, String)> {
    let (api, mut rest) = declaration(content, "API")?;
    let api = match api {
        "1" => 1,
        "2" => 2,
        _ => return Err(Error::LuaApi { value: api.into() }),
    };
    let module = if api == 2 {
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
        Some(id.to_owned())
    } else {
        None
    };
    let prefix = &content[..content.len() - rest.len()];
    let mut body: String = prefix
        .chars()
        .map(|c| if c == '\n' { '\n' } else { ' ' })
        .collect();
    body.push_str(rest);
    Ok((api, module, body))
}

fn script_error(path: &Path, phase: &'static str, module: Option<&str>, source: Error) -> Error {
    Error::LuaScript {
        path: path.into(),
        phase,
        module: module.map(str::to_owned),
        source: Box::new(source),
    }
}

fn callback(lua: &Lua, api: u8, name: &str) -> Result<Option<Function>> {
    match lua.globals().get::<Value>(name)? {
        Value::Nil => Ok(None),
        Value::Function(function) => Ok(Some(function)),
        _ => Err(Error::LuaContract {
            api,
            function: name.into(),
            reason: "expected a function",
        }),
    }
}

struct LuaScript {
    path: PathBuf,
    lua: Lua,
    before: Option<Function>,
    after: Option<Function>,
    load: Option<Function>,
}

impl LuaScript {
    fn read(path: PathBuf, module_dir: &Path) -> Result<(Option<String>, Self)> {
        let content = fs::read_to_string(&path)?;
        let (api, module, body) = header(&content)?;
        let lua = Lua::new_with(
            StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8,
            LuaOptions::default(),
        )?;
        // Keep metadata in __index so every assignment passes through __newindex.
        for name in ["dofile", "loadfile", "load", "rawset", "setmetatable"] {
            lua.globals().set(name, Value::Nil)?;
        }
        let constants = lua.create_table()?;
        constants.set("API", api)?;
        constants.set("MODULE", module.clone())?;
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
                    let forbidden = match api {
                        1 => matches!(
                            name.as_ref(),
                            "load" | "skip" | "block" | "replace" | "remove"
                        ),
                        _ => matches!(name.as_ref(), "before" | "after" | "bind_mount"),
                    };
                    if forbidden {
                        return Err(mlua::Error::external(Error::LuaContract {
                            api,
                            function: name.to_string(),
                            reason: "unavailable in this API version",
                        }));
                    }
                }
                table.raw_set(key, value)
            })?,
        )?;
        lua.globals().set_metatable(Some(meta))?;
        lua.load(&body)
            .set_name(format!("@{}", path.display()))
            .set_mode(ChunkMode::Text)
            .exec()?;
        let before = callback(&lua, api, "before")?;
        let after = callback(&lua, api, "after")?;
        let load = callback(&lua, api, "load")?;
        if api == 2 {
            let info = load
                .as_ref()
                .ok_or_else(|| Error::LuaContract {
                    api,
                    function: "load".into(),
                    reason: "required callback missing",
                })?
                .info();
            if info.what != "Lua" || info.num_params != 0 || info.is_vararg {
                return Err(Error::LuaContract {
                    api,
                    function: "load".into(),
                    reason: "expected a Lua function with no parameters",
                });
            }
        }
        Ok((
            module,
            Self {
                path,
                lua,
                before,
                after,
                load,
            },
        ))
    }

    fn bind_hook(&self, hook: &Option<Function>, commands: &mut Vec<BindMount>) -> Result<()> {
        self.lua.scope(|scope| {
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
            let result = hook.as_ref().map_or(Ok(()), |hook| hook.call::<()>(()));
            self.lua.globals().raw_set("bind_mount", Value::Nil)?;
            result
        })?;
        Ok(())
    }

    fn module_hook(&self, plan: &mut ModulePlan, extra_mount: &[String]) -> Result<()> {
        let plan = RefCell::new(plan);
        self.lua.scope(|scope| {
            self.lua.globals().raw_set(
                "skip",
                scope.create_function_mut(|_, args: MultiValue| {
                    if !args.is_empty() {
                        return Err(mlua::Error::runtime("skip expects no arguments"));
                    }
                    **plan.borrow_mut() = ModulePlan {
                        skip: true,
                        ..ModulePlan::default()
                    };
                    Ok(())
                })?,
            )?;
            for (name, remove) in [("block", false), ("remove", true)] {
                let plan = &plan;
                self.lua.globals().raw_set(
                    name,
                    scope.create_function_mut(move |_, args: MultiValue| {
                        if plan.borrow().skip {
                            return Ok(());
                        }
                        let path = string_arg(args)?;
                        let path =
                            relative_path(&path, extra_mount).map_err(mlua::Error::external)?;
                        if remove {
                            plan.borrow_mut().removed.insert(path);
                        } else {
                            plan.borrow_mut().blocked.insert(path);
                        }
                        Ok(())
                    })?,
                )?;
            }
            self.lua.globals().raw_set(
                "replace",
                scope.create_function_mut(|_, args: MultiValue| {
                    if plan.borrow().skip {
                        return Ok(());
                    }
                    let (source, target) = string_pair(args)?;
                    validate_path(&source).map_err(mlua::Error::external)?;
                    if !Path::new(&source).is_absolute() {
                        return Err(mlua::Error::external(Error::RulePath {
                            path: source,
                            reason: "replace source must be absolute",
                        }));
                    }
                    let target =
                        relative_path(&target, extra_mount).map_err(mlua::Error::external)?;
                    plan.borrow_mut().replacements.insert(target, source.into());
                    Ok(())
                })?,
            )?;
            let result = self
                .load
                .as_ref()
                .expect("API 2 load checked during loading")
                .call::<()>(());
            for name in ["skip", "block", "remove", "replace"] {
                self.lua.globals().raw_set(name, Value::Nil)?;
            }
            result
        })?;
        Ok(())
    }
}

fn string_arg(mut args: MultiValue) -> mlua::Result<String> {
    if args.len() != 1 {
        return Err(mlua::Error::runtime("expected one path string"));
    }
    match args.pop_front() {
        Some(Value::String(path)) => Ok(path.to_str()?.to_owned()),
        _ => Err(mlua::Error::runtime("expected a path string")),
    }
}

fn string_pair(mut args: MultiValue) -> mlua::Result<(String, String)> {
    if args.len() != 2 {
        return Err(mlua::Error::runtime("expected two path strings"));
    }
    let target = string_arg(MultiValue::from_vec(vec![
        args.pop_back().expect("length checked"),
    ]))?;
    Ok((string_arg(args)?, target))
}

#[allow(non_snake_case)]
pub struct LuaParser {
    scripts: Vec<LuaScript>,
    LUAS: FxHashMap<String, Vec<LuaScript>>,
    visited: RefCell<FxHashSet<String>>,
}

impl LuaParser {
    pub fn load(path: &Path, module_dir: &Path) -> Result<Self> {
        let mut parser = Self {
            scripts: Vec::new(),
            LUAS: FxHashMap::default(),
            visited: RefCell::default(),
        };
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(parser),
            Err(error) => {
                return Err(script_error(path, "directory read", None, error.into()));
            }
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
        for path in paths {
            let (module, script) = LuaScript::read(path.clone(), module_dir)
                .map_err(|error| script_error(&path, "loading", None, error))?;
            if let Some(module) = module {
                parser.LUAS.entry(module).or_default().push(script);
            } else {
                parser.scripts.push(script);
            }
        }
        Ok(parser)
    }

    pub fn before(&self) -> Result<Vec<BindMount>> {
        let mut commands = Vec::new();
        for script in &self.scripts {
            script
                .bind_hook(&script.before, &mut commands)
                .map_err(|error| script_error(&script.path, "API 1 before", None, error))?;
        }
        Ok(commands)
    }

    /// Called at the scan boundary, before examining this module's mount content.
    pub fn scan(&self, module: &mut AppModule, extra_mount: &[String]) -> Result<ModulePlan> {
        let mut plan = ModulePlan::default();
        if !self.visited.borrow_mut().insert(module.id.clone()) {
            return Err(Error::DuplicateModuleScan {
                module: module.id.clone(),
            });
        }
        if let Some(scripts) = self.LUAS.get(&module.id) {
            for script in scripts {
                script
                    .module_hook(&mut plan, extra_mount)
                    .map_err(|error| {
                        script_error(&script.path, "API 2 load", Some(&module.id), error)
                    })?;
                if plan.skip {
                    module.skip_mount();
                    break;
                }
            }
        }
        if !plan.skip {
            for path in &plan.blocked {
                module.record_rule(path, "block");
            }
            for path in &plan.removed {
                module.record_rule(path, "remove");
            }
            for path in plan.replacements.keys() {
                if !plan.blocks(path) && !plan.removes(path) {
                    module.record_rule(path, "replace");
                }
            }
        }
        Ok(plan)
    }

    pub fn after(&self) -> Result<Vec<BindMount>> {
        let mut commands = Vec::new();
        for script in &self.scripts {
            script
                .bind_hook(&script.after, &mut commands)
                .map_err(|error| script_error(&script.path, "API 1 after", None, error))?;
        }
        Ok(commands)
    }
}

#[cfg(test)]
#[path = "../tests/unit/parser.rs"]
mod tests;
