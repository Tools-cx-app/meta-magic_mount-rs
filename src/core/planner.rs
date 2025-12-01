use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;
use crate::{conf::config, defs, core::inventory::Module};

#[derive(Debug)]
pub struct OverlayOperation {
    pub target: String,
    pub lowerdirs: Vec<PathBuf>,
}

#[derive(Debug, Default)]
pub struct MountPlan {
    pub overlay_ops: Vec<OverlayOperation>,
    pub magic_module_paths: Vec<PathBuf>,
    
    pub overlay_module_ids: Vec<String>,
    pub magic_module_ids: Vec<String>,
}

/// Generates a mount plan based on the inventory and current storage state.
/// The storage_root contains the SYNCED module files.
pub fn generate(
    config: &config::Config, 
    modules: &[Module], 
    storage_root: &Path
) -> Result<MountPlan> {
    let mut plan = MountPlan::default();
    
    let mut partition_layers: HashMap<String, Vec<PathBuf>> = HashMap::new();
    let mut magic_paths = HashSet::new();
    let mut overlay_ids = HashSet::new();
    let mut magic_ids = HashSet::new();

    let mut target_partitions = defs::BUILTIN_PARTITIONS.to_vec();
    target_partitions.extend(config.partitions.iter().map(|s| s.as_str()));

    for module in modules {
        let content_path = storage_root.join(&module.id);
        
        if !content_path.exists() {
            log::debug!("Planner: Module {} content missing, skipping", module.id);
            continue;
        }

        if module.mode == "magic" {
            if has_meaningful_content(&content_path, &target_partitions) {
                magic_paths.insert(content_path);
                magic_ids.insert(module.id.clone());
            }
        } else {
            let mut participates_in_overlay = false;

            for part in &target_partitions {
                let part_path = content_path.join(part);
                
                if part_path.is_dir() && has_files(&part_path) {
                    partition_layers.entry(part.to_string())
                        .or_default()
                        .push(part_path);
                    participates_in_overlay = true;
                }
            }

            if participates_in_overlay {
                overlay_ids.insert(module.id.clone());
            } else {
                if has_meaningful_content(&content_path, &target_partitions) {
                }
            }
        }
    }

    for (part, layers) in partition_layers {
        let initial_target_path = format!("/{}", part);
        let target_path_obj = Path::new(&initial_target_path);
        let resolved_target = if target_path_obj.is_symlink() || target_path_obj.exists() {
            match target_path_obj.canonicalize() {
                Ok(p) => {
                    if p != target_path_obj {
                        log::debug!("Planner: Resolved symlink {} -> {}", initial_target_path, p.display());
                    }
                    p
                },
                Err(e) => {
                    log::warn!("Planner: Failed to resolve path {}: {}. Skipping.", initial_target_path, e);
                    continue;
                }
            }
        } else {
            continue;
        };

        if !resolved_target.is_dir() {
            log::warn!("Planner: Target {} is not a directory, skipping", resolved_target.display());
            continue;
        }

        plan.overlay_ops.push(OverlayOperation {
            target: resolved_target.to_string_lossy().to_string(),
            lowerdirs: layers,
        });
    }

    plan.magic_module_paths = magic_paths.into_iter().collect();
    plan.overlay_module_ids = overlay_ids.into_iter().collect();
    plan.magic_module_ids = magic_ids.into_iter().collect();

    plan.overlay_module_ids.sort();
    plan.magic_module_ids.sort();

    Ok(plan)
}

fn has_files(path: &Path) -> bool {
    if let Ok(entries) = fs::read_dir(path) {
        for _ in entries.flatten() {
            return true;
        }
    }
    false
}

fn has_meaningful_content(base: &Path, partitions: &[&str]) -> bool {
    for part in partitions {
        let p = base.join(part);
        if p.exists() && has_files(&p) {
            return true;
        }
    }
    false
}
