// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use mlua::{Function, Lua, MultiValue, Value};
use rustc_hash::FxHashSet;

use super::{Api, string_pair, validate_path};
use crate::errors::{Error, Result};
use crate::extension::core::Script;

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

fn string_arg(mut args: MultiValue) -> mlua::Result<String> {
    if args.len() != 1 {
        return Err(mlua::Error::runtime("expected one path string"));
    }
    match args.pop_front() {
        Some(Value::String(path)) => Ok(path.to_str()?.to_owned()),
        _ => Err(mlua::Error::runtime("expected a path string")),
    }
}

pub(in crate::extension) struct ApiV2 {
    load: Function,
}

impl Api for ApiV2 {
    const VERSION: u8 = 2;
    const UNAVAILABLE: &'static [&'static str] = &["before", "after", "bind_mount"];

    fn callbacks(lua: &Lua) -> Result<Self> {
        let load = Self::callback(lua, "load")?.ok_or_else(|| Error::LuaContract {
            api: Self::VERSION,
            function: "load".into(),
            reason: "required callback missing",
        })?;
        let info = load.info();
        if info.what != "Lua" || info.num_params != 0 || info.is_vararg {
            return Err(Error::LuaContract {
                api: Self::VERSION,
                function: "load".into(),
                reason: "expected a Lua function with no parameters",
            });
        }
        Ok(Self { load })
    }
}

impl Script<ApiV2> {
    pub(in crate::extension) fn load_module(
        &self,
        module: &str,
        plan: &mut ModulePlan,
        extra_mount: &[String],
    ) -> Result<()> {
        let plan = RefCell::new(plan);
        self.lua
            .scope(|scope| {
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
                let result = self.callbacks.load.call::<()>(());
                for name in ["skip", "block", "remove", "replace"] {
                    self.lua.globals().raw_set(name, Value::Nil)?;
                }
                result
            })
            .map_err(|error| self.error("API 2 load", Some(module), error.into()))
    }
}
