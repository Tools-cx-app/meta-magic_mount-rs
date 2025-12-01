use std::collections::HashSet;
use std::fs;
use std::path::Path;
use anyhow::Result;
use crate::{defs, utils, core::inventory::Module};

/// Synchronizes enabled modules from source to the prepared storage.
/// Implements a smart sync strategy to avoid unnecessary I/O.
pub fn perform_sync(modules: &[Module], target_base: &Path) -> Result<()> {
    log::info!("Starting smart module sync to {}", target_base.display());

    prune_orphaned_modules(modules, target_base)?;

    for module in modules {
        let dst = target_base.join(&module.id);
        
        let has_content = defs::BUILTIN_PARTITIONS.iter().any(|p| {
            let part_path = module.source_path.join(p);
            part_path.exists() && has_files_recursive(&part_path)
        });

        if has_content {
            if should_sync(&module.source_path, &dst) {
                log::info!("Syncing module: {} (Updated/New)", module.id);
                
                if dst.exists() {
                    if let Err(e) = fs::remove_dir_all(&dst) {
                        log::warn!("Failed to clean target dir for {}: {}", module.id, e);
                    }
                }

                if let Err(e) = utils::sync_dir(&module.source_path, &dst) {
                    log::error!("Failed to sync module {}: {}", module.id, e);
                } else {
                    repair_module_contexts(&dst, &module.id);
                }
            } else {
                log::debug!("Skipping module: {} (Up-to-date)", module.id);
            }
        } else {
            log::debug!("Skipping empty module: {}", module.id);
        }
    }
    
    Ok(())
}

/// Removes directories in the target base that do not correspond to any active module.
fn prune_orphaned_modules(modules: &[Module], target_base: &Path) -> Result<()> {
    if !target_base.exists() { return Ok(()); }

    let active_ids: HashSet<&str> = modules.iter().map(|m| m.id.as_str()).collect();

    for entry in fs::read_dir(target_base)? {
        let entry = entry?;
        let path = entry.path();
        let name_os = entry.file_name();
        let name = name_os.to_string_lossy();

        if name == "lost+found" || name == "meta-hybrid" { continue; }

        if !active_ids.contains(name.as_ref()) {
            log::info!("Pruning orphaned module storage: {}", name);
            if path.is_dir() {
                if let Err(e) = fs::remove_dir_all(&path) {
                    log::warn!("Failed to remove orphan dir {}: {}", name, e);
                }
            } else {
                if let Err(e) = fs::remove_file(&path) {
                    log::warn!("Failed to remove orphan file {}: {}", name, e);
                }
            }
        }
    }
    Ok(())
}

/// Determines if a module needs to be synced.
/// Compares `module.prop` content as a heuristic for version/content changes.
fn should_sync(src: &Path, dst: &Path) -> bool {
    if !dst.exists() {
        return true;
    }

    let src_prop = src.join("module.prop");
    let dst_prop = dst.join("module.prop");

    if !src_prop.exists() || !dst_prop.exists() {
        return true;
    }

    match (fs::read(&src_prop), fs::read(&dst_prop)) {
        (Ok(s), Ok(d)) => s != d,
        _ => true,
    }
}

fn repair_module_contexts(module_root: &Path, module_id: &str) {
    for part in defs::BUILTIN_PARTITIONS {
        let part_root = module_root.join(part);
        if part_root.exists() {
            if let Err(e) = recursive_context_repair(module_root, &part_root) {
                log::warn!("Context repair failed for {}/{}: {}", module_id, part, e);
            }
        }
    }
}

fn recursive_context_repair(base: &Path, current: &Path) -> Result<()> {
    if !current.exists() { return Ok(()); }
    
    let relative = current.strip_prefix(base)?;
    let system_path = Path::new("/").join(relative);

    if system_path.exists() {
        let _ = utils::copy_path_context(&system_path, current);
    }

    if current.is_dir() {
        for entry in fs::read_dir(current)? {
            let entry = entry?;
            recursive_context_repair(base, &entry.path())?;
        }
    }
    Ok(())
}

fn has_files_recursive(path: &Path) -> bool {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    if has_files_recursive(&entry.path()) { return true; }
                } else {
                    return true;
                }
            }
        }
    }
    false
}
