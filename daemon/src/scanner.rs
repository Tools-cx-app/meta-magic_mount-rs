// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{io::Cursor, path::Path};

use anyhow::Result;
use java_properties::PropertiesIter;
use regex_lite::Regex;
use rustc_hash::{FxHashMap, FxHashSet};
use tokio::fs;

use crate::defs;

/// Validate `module_id` format and security
/// Module ID must match: ^[a-zA-Z][a-zA-Z0-9._-]+$
/// - Must start with a letter (a-zA-Z)
/// - Followed by one or more alphanumeric, dot, underscore, or hyphen characters
/// - Minimum length: 2 characters
fn validate_module_id(module_id: &str) -> Result<()> {
    let re = Regex::new(r"^[a-zA-Z][a-zA-Z0-9._-]+$")?;
    if re.is_match(module_id) {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Invalid module ID: '{module_id:?}'. Must match /^[a-zA-Z][a-zA-Z0-9._-]+$/"
        ))
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
    has_mount_files: bool,
}

async fn read_prop<P>(path: P) -> Result<FxHashMap<String, String>>
where
    P: AsRef<Path>,
{
    let buffer = fs::read_to_string(path).await?;
    let mut map = FxHashMap::default();
    PropertiesIter::new_with_encoding(Cursor::new(buffer), encoding_rs::UTF_8).read_into(
        |k, v| {
            map.insert(k, v);
        },
    )?;

    Ok(map)
}

async fn collect_modules<P>(module_dir: P, extra: &[String]) -> Vec<ModuleRecord>
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

            let Ok(prop) = read_prop(prop_path).await else {
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

pub async fn list_modules<P>(module_dir: P, extra: &[String]) -> Vec<api::Module>
where
    P: AsRef<Path>,
{
    collect_modules(module_dir, extra)
        .await
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
mod tests {
    use std::fs;

    use super::*;

    fn create_valid_prop_content(id: &str) -> String {
        format!(
            "id={}\nname=Test Module\nversion=1.0.0\nauthor=Tester\ndescription=A test module\n",
            id
        )
    }

    #[tokio::test]
    async fn test_read_prop_success() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let prop_path = tmp_dir.path().join("module.prop");

        let content = "id=test\nname=test\nversion=v1.0\n";
        fs::write(&prop_path, content).unwrap();

        let res = read_prop(&prop_path).await.unwrap();
        assert_eq!(res.get("id").unwrap(), "test");
        assert_eq!(res.get("name").unwrap(), "test");
        assert_eq!(res.get("version").unwrap(), "v1.0");
    }

    #[tokio::test]
    async fn test_read_prop_file_not_found() {
        let res = read_prop("non_existent_file.prop").await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_list_modules_integration() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let module_dir = tmp_dir.path();

        let test1 = module_dir.join("test1");
        fs::create_dir_all(test1.join("system")).unwrap();
        fs::write(
            test1.join("module.prop"),
            create_valid_prop_content("test1"),
        )
        .unwrap();

        let test2 = module_dir.join("test2");
        fs::create_dir_all(test2.join("system")).unwrap();
        fs::write(
            test2.join("module.prop"),
            create_valid_prop_content("test2"),
        )
        .unwrap();
        fs::File::create(test2.join(defs::DISABLE_FILE_NAME)).unwrap();

        let test3 = module_dir.join("test3");
        fs::create_dir_all(test3.join("system")).unwrap();
        fs::write(
            test3.join("module.prop"),
            create_valid_prop_content("test3"),
        )
        .unwrap();
        fs::File::create(test3.join(defs::SKIP_MOUNT_FILE_NAME)).unwrap();

        let test4 = module_dir.join("test4");
        fs::create_dir_all(&test4).unwrap();
        fs::write(test4.join("module.prop"), "id=test4\n").unwrap();

        let test5 = module_dir.join("test5");
        fs::create_dir_all(&test5).unwrap();
        fs::write(
            test5.join("module.prop"),
            create_valid_prop_content("test5"),
        )
        .unwrap();

        let test6 = module_dir.join("test6");
        fs::create_dir_all(test6.join("vendor")).unwrap();
        fs::write(
            test6.join("module.prop"),
            create_valid_prop_content("test6"),
        )
        .unwrap();

        let extra_partitions = vec!["vendor".to_string()];
        let result: Vec<api::Module> = list_modules(module_dir, &extra_partitions).await;

        assert_eq!(result.len(), 5);

        assert_eq!(result[0].id, "test1");
        assert!(result[0].is_mounted);
        assert!(!result[1].is_mounted);
        assert!(!result[2].is_mounted);
        assert!(!result[3].is_mounted);
        assert!(result[4].is_mounted);
    }

    #[tokio::test]
    async fn test_list_modules_empty_dir() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let result = list_modules(tmp_dir.path(), &[]).await;
        assert!(result.is_empty());
    }
}
