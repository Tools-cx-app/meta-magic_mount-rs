// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{fs, io::Cursor, path::Path};

use anyhow::Result;
use java_properties::PropertiesIter;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{defs, utils::validate_module_id};

#[derive(Debug)]
struct ModuleRecord {
    id: String,
    name: String,
    version: String,
    author: String,
    description: String,
    disabled: bool,
    skip_mount: bool,
    has_mount_files: bool,
}

fn read_prop<P>(path: P) -> Result<FxHashMap<String, String>>
where
    P: AsRef<Path>,
{
    let buffer = fs::read_to_string(path)?;
    let mut map = FxHashMap::default();
    PropertiesIter::new_with_encoding(Cursor::new(buffer), encoding_rs::UTF_8).read_into(
        |k, v| {
            map.insert(k, v);
        },
    )?;

    Ok(map)
}

fn collect_modules<P>(module_dir: P, extra: &[String]) -> Vec<ModuleRecord>
where
    P: AsRef<Path>,
{
    let mut modules = Vec::new();

    if let Ok(entries) = module_dir.as_ref().read_dir() {
        for entry in entries.flatten() {
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            if !path.join("module.prop").exists() {
                continue;
            }

            let mut modified = false;
            let mut partitions = FxHashSet::default();
            partitions.insert("system".to_string());
            partitions.extend(extra.iter().cloned());

            for partition in &partitions {
                if entry.path().join(partition).is_dir() {
                    modified = true;
                    break;
                }
            }

            let disabled = path.join(defs::DISABLE_FILE_NAME).exists()
                || path.join(defs::REMOVE_FILE_NAME).exists();
            let skip_mount = path.join(defs::SKIP_MOUNT_FILE_NAME).exists();

            let prop_path = path.join("module.prop");

            let Ok(prop) = read_prop(prop_path) else {
                continue;
            };
            let Some(id) = prop.get("id") else {
                log::warn!("{} missing module id", path.display());
                continue;
            };
            let Some(name) = prop.get("name") else {
                log::warn!("{} missing module name", path.display());
                continue;
            };
            let Some(version) = prop.get("version") else {
                log::warn!("{} missing module version", path.display());
                continue;
            };
            let Some(author) = prop.get("author") else {
                log::warn!("{} missing module author", path.display());
                continue;
            };
            let Some(description) = prop.get("description") else {
                log::warn!("{} missing module description", path.display());
                continue;
            };

            if validate_module_id(id).is_ok() {
                modules.push(ModuleRecord {
                    id: id.clone(),
                    name: name.clone(),
                    version: version.clone(),
                    author: author.clone(),
                    description: description.clone(),
                    disabled,
                    skip_mount,
                    has_mount_files: modified,
                });
            }
        }
    }
    modules.sort_by(|a, b| a.id.cmp(&b.id));

    modules
}

pub fn list_modules<P>(module_dir: P, extra: &[String]) -> Vec<api::Module>
where
    P: AsRef<Path>,
{
    collect_modules(module_dir, extra)
        .into_iter()
        .map(|module| api::Module {
            id: module.id,
            name: module.name,
            version: module.version,
            author: module.author,
            description: module.description,
            is_mounted: module.has_mount_files && !module.disabled && !module.skip_mount,
        })
        .collect()
}
#[cfg(test)]
#[path = "../tests/unit/magic_mount/scanner.rs"]
mod tests;
