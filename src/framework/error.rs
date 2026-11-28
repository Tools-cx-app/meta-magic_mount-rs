// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("cannot mount root symlink {path:?}!")]
    MountRootSymlink { path: String },
    #[error("dir {path:?} is declared as replaced but it is root!")]
    DirDeclared { path: String },
    #[error("cannot mount root file {path:?}!")]
    MountRootFile { path: String },
    #[error("{path:?} is not a regular directory")]
    RegularDirectory { path: String },
    #[error("Invalid module ID: '{module_id:?}'. Must match /^[a-zA-Z][a-zA-Z0-9._-]+$/")]
    InvalidModuleID { module_id: String },
    #[error("Invalid Bind path: '{path}'")]
    InvalidBindPath { path: String },
    #[error("invalid Lua header {field}: {reason}")]
    LuaHeader {
        field: &'static str,
        reason: &'static str,
    },
    #[error("unsupported Lua API {value:?}; expected literal integer 1 or 2")]
    LuaApi { value: String },
    #[error("invalid rule path {path:?}: {reason}")]
    RulePath { path: String, reason: &'static str },
    #[error("Lua API {api} function {function}: {reason}")]
    LuaContract {
        api: u8,
        function: String,
        reason: &'static str,
    },
    #[error("Lua {phase} failed in {path} (MODULE={module:?}): {source}")]
    LuaScript {
        path: std::path::PathBuf,
        phase: &'static str,
        module: Option<String>,
        #[source]
        source: Box<Error>,
    },
    #[error("module {module:?} scanned more than once")]
    DuplicateModuleScan { module: String },
    #[error("module {module:?} rule preparation failed: {source}")]
    ModuleRules {
        module: String,
        #[source]
        source: Box<Error>,
    },
    #[error("cannot read rule source {path}: {source}")]
    RuleSource {
        path: std::path::PathBuf,
        #[source]
        source: Box<Error>,
    },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Pproperties(#[from] java_properties::PropertiesError),
    #[error(transparent)]
    AnyHow(#[from] anyhow::Error),
    #[error(transparent)]
    SerJson(#[from] serde_json::Error),
    #[error(transparent)]
    Rustix(#[from] rustix::io::Errno),
    #[error(transparent)]
    Regex(#[from] regex_lite::Error),
    #[error(transparent)]
    Mlua(#[from] mlua::Error),
    #[error(transparent)]
    Hex(#[from] hex::FromHexError),
}
