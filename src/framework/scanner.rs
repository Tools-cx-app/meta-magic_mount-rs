// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{collections::BTreeMap, fs, io::Cursor, path::Path};

use java_properties::PropertiesIter;
use regex_lite::Regex;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use super::error::{Error, Result};
use super::paths;

/// Validate `module_id` format and security
/// Module ID must match: ^[a-zA-Z][a-zA-Z0-9._-]+$
/// - Must start with a letter (a-zA-Z)
/// - Followed by one or more alphanumeric, dot, underscore, or hyphen characters
/// - Minimum length: 2 characters
pub fn validate_module_id(module_id: &str) -> Result<()> {
    let re = Regex::new(r"^[a-zA-Z][a-zA-Z0-9._-]+$")?;
    if re.is_match(module_id) {
        Ok(())
    } else {
        Err(Error::InvalidModuleID {
            module_id: module_id.to_string(),
        })
    }
}

#[derive(Debug)]
struct ModuleRecord {
    id: String,
    name: String,
    version: String,
    author: String,
    description: String,
    disabled: bool,
    skip_mount: bool,
    source_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModuleRules {
    default_mode: String,
    paths: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
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
    #[serde(skip)]
    scan_allowed: bool,
}

impl AppModule {
    pub fn mount_path(&self) -> &Path {
        Path::new(&self.source_path)
    }

    #[cfg(test)]
    pub const fn is_mountable(&self) -> bool {
        self.is_mounted
    }

    pub fn skip_mount(&mut self) {
        self.is_mounted = false;
        self.mode = "ignore".into();
        self.rules.default_mode = "ignore".into();
        self.rules.paths.clear();
    }

    pub const fn scan_allowed(&self) -> bool {
        self.scan_allowed
    }

    pub fn set_mount_state(&mut self, mounted: bool) {
        self.is_mounted = mounted;
        self.mode = if mounted { "magic" } else { "ignore" }.into();
        self.rules.default_mode = self.mode.clone();
    }

    pub fn record_rule(&mut self, path: &Path, mode: &str) {
        self.rules
            .paths
            .insert(path.to_string_lossy().into_owned(), mode.into());
    }
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

fn collect_modules<P>(module_dir: P) -> Vec<ModuleRecord>
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

            let disabled = path.join(paths::DISABLE_FILE_NAME).exists()
                || path.join(paths::REMOVE_FILE_NAME).exists();
            let skip_mount = path.join(paths::SKIP_MOUNT_FILE_NAME).exists();

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
                    source_path: path.to_str().unwrap_or_default().to_string(),
                });
            }
        }
    }
    modules.sort_by(|a, b| a.id.cmp(&b.id));

    modules
}

pub fn show_modules(mut modules: Vec<AppModule>, snapshot_path: &Path) -> Result<Vec<AppModule>> {
    let orgi = fs::read_to_string(snapshot_path)?;
    let orgi_modules: Vec<AppModule> = serde_json::from_str(&orgi)?;

    let mut mounted_modules: FxHashMap<_, _> = orgi_modules
        .into_iter()
        .map(|module| (module.id, (module.is_mounted, module.mode, module.rules)))
        .collect();

    for module in &mut modules {
        if let Some((is_mounted, mode, rules)) = mounted_modules.remove(&module.id) {
            module.is_mounted = is_mounted;
            module.mode = mode;
            module.rules = rules;
        } else {
            module.skip_mount();
        }
    }

    Ok(modules)
}

/// Reads identities and exclusion markers without inspecting mount content.
pub fn scan_modules<P>(module_dir: P) -> Vec<AppModule>
where
    P: AsRef<Path>,
{
    collect_modules(module_dir)
        .into_iter()
        .map(|module| AppModule {
            id: module.id,
            name: module.name,
            version: module.version,
            author: module.author,
            description: module.description,
            mode: "ignore".into(),
            is_mounted: false,
            enabled: !module.disabled,
            scan_allowed: !module.disabled && !module.skip_mount,
            source_path: module.source_path,
            rules: ModuleRules {
                default_mode: "ignore".into(),
                paths: BTreeMap::new(),
            },
        })
        .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/scanner.rs"]
mod tests;
