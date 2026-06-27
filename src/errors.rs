// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: Apache-2.0

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
    #[error("missing required --payload argument")]
    MissingArgment,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Glob(#[from] glob::PatternError),
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
    Hex(#[from] hex::FromHexError),
}
