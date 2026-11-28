// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::collections::hash_map::Entry;
use std::fmt;
use std::fs::FileType;
use std::path::PathBuf;

use rustc_hash::FxHashMap;

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
    pub children: FxHashMap<String, Self>,
    // the module that owned this node
    pub module_path: Option<PathBuf>,
    pub replace: bool,
    pub remove: bool,
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
        if self.remove {
            write!(f, " [REMOVE]")?;
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
    pub(super) fn has_content(&self) -> bool {
        self.remove
            || self.replace
            || self.file_type != NodeFileType::Directory
            || self.children.values().any(Self::has_content)
    }

    pub fn new_root<S>(name: S) -> Self
    where
        S: AsRef<str> + Into<String>,
    {
        Self {
            name: name.into(),
            file_type: NodeFileType::Directory,
            children: FxHashMap::default(),
            module_path: None,
            replace: false,
            remove: false,
            skip: false,
        }
    }
}

/// Keeps the first module's conflicting entries, matching the existing merge policy.
pub(super) fn merge_nodes(target: &mut Node, source: Node) {
    if target.remove || target.file_type != NodeFileType::Directory {
        return;
    }
    if target.module_path.is_none() {
        target.module_path = source.module_path.clone();
        target.replace = source.replace;
        target.remove = source.remove;
    }
    if source.file_type != NodeFileType::Directory {
        return;
    }
    for (name, child) in source.children {
        match target.children.entry(name) {
            Entry::Occupied(mut entry) => merge_nodes(entry.get_mut(), child),
            Entry::Vacant(entry) => {
                entry.insert(child);
            }
        }
    }
}
