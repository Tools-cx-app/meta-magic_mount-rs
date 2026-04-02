// Copyright 2025 Magic Mount-rs Authors
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    io::Cursor,
    path::Path,
};

use anyhow::Result;
use java_properties::PropertiesIter;
use serde::Serialize;

use crate::{
    defs::{DISABLE_FILE_NAME, REMOVE_FILE_NAME, SKIP_MOUNT_FILE_NAME},
    utils::validate_module_id,
};

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
    source_path: String,
}

#[derive(Debug, Serialize)]
pub struct ModuleRules {
    default_mode: String,
    paths: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct AppModule {
    pub id: String,
    name: String,
    version: String,
    author: String,
    description: String,
    mode: String,
    is_mounted: bool,
    enabled: bool,
    source_path: String,
    rules: ModuleRules,
}

fn read_prop<P>(path: P) -> Result<HashMap<String, String>>
where
    P: AsRef<Path>,
{
    let buffer = fs::read_to_string(path)?;
    let mut map = HashMap::new();
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
            let mut partitions = HashSet::new();
            partitions.insert("system".to_string());
            partitions.extend(extra.iter().cloned());

            for partition in &partitions {
                if entry.path().join(partition).is_dir() {
                    modified = true;
                    break;
                }
            }

            let disabled =
                path.join(DISABLE_FILE_NAME).exists() || path.join(REMOVE_FILE_NAME).exists();
            let skip_mount = path.join(SKIP_MOUNT_FILE_NAME).exists();

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
                    source_path: path.display().to_string(),
                });
            }
        }
    }
    modules.sort_by(|a, b| a.id.cmp(&b.id));

    modules
}

pub fn list_modules<P>(module_dir: P, extra: &[String]) -> Vec<AppModule>
where
    P: AsRef<Path>,
{
    collect_modules(module_dir, extra)
        .into_iter()
        .map(|module| {
            let is_mounted = module.has_mount_files && !module.disabled && !module.skip_mount;
            let mode = if is_mounted { "magic" } else { "ignore" }.to_string();
            let default_mode = mode.clone();

            AppModule {
                id: module.id,
                name: module.name,
                version: module.version,
                author: module.author,
                description: module.description,
                mode,
                is_mounted,
                enabled: !module.disabled,
                source_path: module.source_path,
                rules: ModuleRules {
                    default_mode,
                    paths: BTreeMap::new(),
                },
            }
        })
        .collect()
}
