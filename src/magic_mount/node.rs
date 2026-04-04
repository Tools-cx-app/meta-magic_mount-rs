// Copyright 2025 Magic Mount-rs Authors
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
    fmt,
    fs::{DirEntry, FileType},
    os::unix::fs::{FileTypeExt, MetadataExt},
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::Result;
use extattr::lgetxattr;
use rustix::path::Arg;

use crate::defs::{REPLACE_DIR_FILE_NAME, REPLACE_DIR_XATTR};

pub static IGNORE_LIST: OnceLock<Option<HashSet<String>>> = OnceLock::new();

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum NodeFileType {
    RegularFile,
    Directory,
    Symlink,
    Whiteout,
}

impl From<FileType> for NodeFileType {
    fn from(value: FileType) -> Self {
        if value.is_file() {
            Self::RegularFile
        } else if value.is_dir() {
            Self::Directory
        } else if value.is_symlink() {
            Self::Symlink
        } else {
            Self::Whiteout
        }
    }
}

#[derive(Clone)]
pub struct Node {
    pub name: String,
    pub file_type: NodeFileType,
    pub children: HashMap<String, Self>,
    // the module that owned this node
    pub module_path: Option<PathBuf>,
    pub replace: bool,
    pub skip: bool,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug_tree(f, 0)
    }
}

impl fmt::Display for NodeFileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::RegularFile => "RegularFile",
            Self::Directory => "Directory",
            Self::Symlink => "Symlink",
            Self::Whiteout => "Whiteout",
        })
    }
}

impl Node {
    fn debug_tree(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        let indent_str = "  ".repeat(indent);

        write!(f, "{}{} ({})", indent_str, self.name, self.file_type)?;
        if let Some(path) = &self.module_path {
            write!(f, " [{}]", path.display())?;
        }
        if self.replace {
            write!(f, " [REPLACE]")?;
        }
        if self.skip {
            write!(f, " [SKIP]")?;
        }
        writeln!(f)?;

        for child in self.children.values() {
            child.debug_tree(f, indent + 1)?;
        }
        Ok(())
    }
}

impl Node {
    pub fn collect_module_files<P>(&mut self, module_dir: P) -> Result<bool>
    where
        P: AsRef<Path>,
    {
        let dir = module_dir.as_ref();
        let mut has_file = false;
        for entry in dir.read_dir()?.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();

            let node = match self.children.entry(name.clone()) {
                Entry::Occupied(o) => Some(o.into_mut()),
                Entry::Vacant(v) => Self::new_module(&name, &entry).map(|it| v.insert(it)),
            };

            if let Some(node) = node {
                has_file |= if node.file_type == NodeFileType::Directory {
                    node.collect_module_files(dir.join(&node.name))? || node.replace
                } else {
                    true
                }
            }
        }

        Ok(has_file)
    }

    fn dir_is_replace<P>(path: P) -> bool
    where
        P: AsRef<Path>,
    {
        if let Ok(v) = lgetxattr(&path, REPLACE_DIR_XATTR)
            && String::from_utf8_lossy(&v) == "y"
        {
            return true;
        }

        path.as_ref().join(REPLACE_DIR_FILE_NAME).exists()
    }

    fn dir_is_skip<P>(path: P) -> bool
    where
        P: AsRef<Path>,
    {
        let list = IGNORE_LIST.get().unwrap();
        if let Some(f) = list
            && f.iter()
                .any(|s| s == path.as_ref().to_str().unwrap_or_default())
        {
            return true;
        }

        false
    }

    pub fn new_root<S>(name: S) -> Self
    where
        S: AsRef<str> + Into<String>,
    {
        Self {
            name: name.into(),
            file_type: NodeFileType::Directory,
            children: HashMap::default(),
            module_path: None,
            replace: false,
            skip: false,
        }
    }

    pub fn new_module<S>(name: &S, entry: &DirEntry) -> Option<Self>
    where
        S: ToString,
    {
        if let Ok(metadata) = entry.metadata() {
            let path = entry.path();
            let file_type = if metadata.file_type().is_char_device() && metadata.rdev() == 0 {
                Some(NodeFileType::Whiteout)
            } else {
                Some(NodeFileType::from(metadata.file_type()))
            };
            if let Some(file_type) = file_type {
                let replace = file_type == NodeFileType::Directory && Self::dir_is_replace(&path);
                let skip = Self::dir_is_skip(&path);
                if replace {
                    log::debug!("{} need replace", path.display());
                }
                if skip {
                    log::debug!("{} was skip", path.display());
                }
                return Some(Self {
                    name: name.to_string(),
                    file_type,
                    children: HashMap::default(),
                    module_path: Some(path),
                    replace,
                    skip,
                });
            }
        }

        None
    }
}
