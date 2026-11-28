// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::collections::hash_map::Entry;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};

use extattr::lgetxattr;
use rustc_hash::FxHashMap;
use rustix::path::Arg;

use super::tree::{Node, NodeFileType, merge_nodes};
use crate::framework::error::{Error, Result};
use crate::framework::extension::{Extensions, ModulePlan};
use crate::framework::{paths, scanner::AppModule};

impl Node {
    fn from_path(name: &str, path: &Path) -> Result<Self> {
        let metadata = path.symlink_metadata()?;
        let file_type = if metadata.file_type().is_char_device() && metadata.rdev() == 0 {
            NodeFileType::Whiteout
        } else {
            NodeFileType::from(metadata.file_type())
        };
        let directory = file_type == NodeFileType::Directory;
        Ok(Self {
            name: name.into(),
            file_type,
            children: FxHashMap::default(),
            module_path: Some(path.into()),
            replace: directory && Self::dir_is_replace(path),
            remove: directory && path.join(paths::REMOVE_DIR_FILE_NAME).exists(),
            skip: Self::dir_is_skip(path),
        })
    }

    pub(super) fn collect_filtered(
        &mut self,
        source: &Path,
        relative: &Path,
        plan: &ModulePlan,
    ) -> Result<()> {
        if self.remove {
            return Ok(());
        }
        for entry in source.read_dir()? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let target = relative.join(&name);
            if plan.blocks(&target)
                || plan.removes(&target)
                || plan.replacements.contains_key(&target)
            {
                continue;
            }
            let mut node = Self::from_path(&name, &entry.path())?;
            if node.skip {
                continue;
            }
            if node.file_type == NodeFileType::Directory {
                node.collect_filtered(&entry.path(), &target, plan)?;
            }
            self.children.insert(name, node);
        }
        Ok(())
    }

    /// Creates logical parents using existing module directories or module-root metadata.
    fn insert_rule(&mut self, path: &Path, node: Self, module_dir: &Path) -> Result<()> {
        let mut parent = self;
        let mut relative = PathBuf::new();
        let mut parts = path.iter().peekable();
        while let Some(part) = parts.next() {
            let name = part.to_string_lossy().into_owned();
            if parts.peek().is_none() {
                parent.children.insert(name, node);
                return Ok(());
            }
            relative.push(part);
            parent = parent.children.entry(name.clone()).or_insert_with(|| {
                let mut directory = Self::new_root(name);
                let source = module_dir.join(&relative);
                directory.module_path = Some(if source.is_dir() {
                    source
                } else {
                    module_dir.into()
                });
                directory
            });
            if parent.remove {
                return Ok(());
            }
            if parent.file_type != NodeFileType::Directory {
                return Err(Error::RulePath {
                    path: relative.to_string_lossy().into_owned(),
                    reason: "rule parent is not a directory",
                });
            }
        }
        Ok(())
    }

    fn dir_is_replace<P>(path: P) -> bool
    where
        P: AsRef<Path>,
    {
        if lgetxattr(&path, paths::REPLACE_DIR_XATTR)
            .is_ok_and(|s| String::from_utf8_lossy(&s) == "y")
        {
            true
        } else {
            path.as_ref().join(paths::REPLACE_DIR_FILE_NAME).exists()
        }
    }

    fn dir_is_skip<P>(path: P) -> bool
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        path.to_string_lossy().ends_with(".replace")
            || path
                .file_name()
                .is_some_and(|name| name == paths::REMOVE_DIR_FILE_NAME)
    }
}

/// Builds one module independently so block cannot erase another module's contribution.
fn collect_contribution(
    module_dir: &Path,
    plan: &ModulePlan,
    extra_mount: &[String],
) -> Result<Node> {
    let mut contribution = Node::new_root("");
    for partition in std::iter::once("system").chain(extra_mount.iter().map(String::as_str)) {
        let relative = Path::new(partition);
        if plan.blocks(relative)
            || plan.removes(relative)
            || plan.replacements.contains_key(relative)
        {
            continue;
        }
        let path = module_dir.join(relative);
        if path.is_dir() {
            let mut node = Node::from_path(partition, &path)?;
            node.collect_filtered(&path, relative, plan)?;
            contribution.children.insert(partition.into(), node);
        }
    }
    for (target, source) in &plan.replacements {
        if plan.blocks(target) || plan.removes(target) {
            continue;
        }
        let metadata = source
            .symlink_metadata()
            .map_err(|source_error| Error::RuleSource {
                path: source.clone(),
                source: Box::new(source_error.into()),
            })?;
        if !metadata.is_file() && !metadata.is_dir() {
            return Err(Error::RulePath {
                path: source.to_string_lossy().into_owned(),
                reason: "replace source must be a regular file or directory",
            });
        }
        if target.components().count() == 1 && !metadata.is_dir() {
            return Err(Error::RulePath {
                path: target.to_string_lossy().into_owned(),
                reason: "partition replacement must be a directory",
            });
        }
        let name = target
            .file_name()
            .ok_or_else(|| Error::RulePath {
                path: target.to_string_lossy().into_owned(),
                reason: "replacement target missing name",
            })?
            .to_string_lossy();
        let mut node = Node::from_path(&name, source)?;
        node.skip = false;
        if metadata.is_dir() {
            node.replace = true;
            node.collect_filtered(source, target, plan)?;
        }
        contribution.insert_rule(target, node, module_dir)?;
    }
    Ok(contribution)
}

/// Applies a module's rules before merging its contribution into the shared tree.
pub fn collect_module_files(
    modules: &mut [AppModule],
    extra_partitions: &[String],
    extra_mount: &[String],
    hooks: &Extensions,
) -> Result<Option<Node>> {
    let mut root = Node::new_root("");
    let mut removals = Vec::new();
    let mut has_file = false;
    modules.sort_by(|a, b| a.id.cmp(&b.id));
    for module in modules {
        if !module.scan_allowed() {
            continue;
        }
        let plan = hooks.scan(module, extra_mount)?;
        if plan.skip {
            continue;
        }
        let contribution =
            collect_contribution(module.mount_path(), &plan, extra_mount).map_err(|source| {
                Error::ModuleRules {
                    module: module.id.clone(),
                    source: Box::new(source),
                }
            })?;
        let mounted = contribution.has_content() || !plan.removed.is_empty();
        module.set_mount_state(mounted);
        has_file |= mounted;
        merge_nodes(&mut root, contribution);
        removals.extend(
            plan.removed
                .into_iter()
                .map(|path| (path, module.mount_path().to_path_buf())),
        );
    }
    // Apply global hides before system partition relocation, including paths absent from modules.
    removals.sort_by(|a, b| a.0.cmp(&b.0));
    for (path, metadata_source) in removals {
        if path.components().count() <= 1 {
            return Err(Error::RulePath {
                path: path.to_string_lossy().into_owned(),
                reason: "cannot remove a partition root",
            });
        }
        let name = path.file_name().ok_or_else(|| Error::RulePath {
            path: path.to_string_lossy().into_owned(),
            reason: "remove target missing name",
        })?;
        let mut node = Node::new_root(name.to_string_lossy().into_owned());
        node.file_type = NodeFileType::Whiteout;
        node.remove = true;
        root.insert_rule(&path, node, &metadata_source)?;
    }
    let mut system = root
        .children
        .remove("system")
        .unwrap_or_else(|| Node::new_root("system"));

    if has_file {
        const BUILTIN_PARTITIONS: [(&str, bool); 4] = [
            ("vendor", true),
            ("system_ext", true),
            ("product", true),
            ("odm", false),
        ];

        for (partition, require_symlink) in BUILTIN_PARTITIONS {
            let path_of_root = Path::new("/").join(partition);
            let path_of_system = Path::new("/system").join(partition);
            if path_of_root.is_dir() && (!require_symlink || path_of_system.is_symlink()) {
                let name = partition.to_string();
                if let Some(node) = system.children.remove(&name) {
                    match root.children.entry(name) {
                        Entry::Occupied(mut entry) => merge_nodes(entry.get_mut(), node),
                        Entry::Vacant(entry) => {
                            entry.insert(node);
                        }
                    }
                }
            }
        }

        for partition in extra_partitions {
            if BUILTIN_PARTITIONS.iter().any(|(p, _)| p == partition) {
                continue;
            }
            if partition == "system" {
                continue;
            }

            let path_of_root = Path::new("/").join(partition);
            let path_of_system = Path::new("/system").join(partition);
            let require_symlink = false;

            if path_of_root.is_dir() && (!require_symlink || path_of_system.is_symlink()) {
                let name = partition.clone();
                if let Some(node) = system.children.remove(&name) {
                    log::debug!("attach extra partition '{name}' to root");
                    match root.children.entry(name) {
                        Entry::Occupied(mut entry) => merge_nodes(entry.get_mut(), node),
                        Entry::Vacant(entry) => {
                            entry.insert(node);
                        }
                    }
                }
            }
        }

        root.children.insert("system".to_string(), system);
        Ok(Some(root))
    } else {
        Ok(None)
    }
}
