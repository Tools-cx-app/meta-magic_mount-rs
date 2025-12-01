use std::path::{Path, PathBuf};
use anyhow::Result;
use crate::{
    conf::config, 
    mount::{magic, overlay}, 
    utils,
    core::planner::MountPlan
};

pub struct ExecutionResult {
    pub overlay_module_ids: Vec<String>,
    pub magic_module_ids: Vec<String>,
}

fn extract_id(path: &Path) -> Option<String> {
    path.parent()
        .and_then(|p| p.file_name())
        .map(|s| s.to_string_lossy().to_string())
}

fn extract_module_root(partition_path: &Path) -> Option<PathBuf> {
    partition_path.parent().map(|p| p.to_path_buf())
}

pub fn execute(plan: &MountPlan, config: &config::Config) -> Result<ExecutionResult> {
    let mut magic_queue = plan.magic_module_paths.clone();
    
    let mut final_overlay_ids = plan.overlay_module_ids.clone();
    let mut fallback_ids = Vec::new();

    for op in &plan.overlay_ops {
        let lowerdir_strings: Vec<String> = op.lowerdirs.iter()
            .map(|p| p.display().to_string())
            .collect();
            
        log::info!("Mounting {} [OVERLAY] ({} layers)", op.target, lowerdir_strings.len());
        
        if let Err(e) = overlay::mount_overlay(&op.target, &lowerdir_strings, None, None, config.disable_umount) {
            log::warn!("OverlayFS failed for {}: {}. Triggering fallback.", op.target, e);
            
            for layer_path in &op.lowerdirs {
                if let Some(root) = extract_module_root(layer_path) {
                    magic_queue.push(root.clone());
                    if let Some(id) = extract_id(layer_path) {
                        fallback_ids.push(id);
                    }
                }
            }
        }
    }

    if !fallback_ids.is_empty() {
        final_overlay_ids.retain(|id| !fallback_ids.contains(id));
        log::info!("{} modules fell back to Magic Mount.", fallback_ids.len());
    }

    magic_queue.sort();
    magic_queue.dedup();

    let mut final_magic_ids = Vec::new();

    if !magic_queue.is_empty() {
        let tempdir = if let Some(t) = &config.tempdir { 
            t.clone() 
        } else { 
            utils::select_temp_dir()? 
        };
        
        for path in &magic_queue {
            if let Some(name) = path.file_name() {
                final_magic_ids.push(name.to_string_lossy().to_string());
            }
        }
        
        log::info!("Executing Magic Mount for {} modules...", magic_queue.len());
        
        utils::ensure_temp_dir(&tempdir)?;
        
        if let Err(e) = magic::mount_partitions(
            &tempdir, 
            &magic_queue, 
            &config.mountsource, 
            &config.partitions, 
            config.disable_umount
        ) {
            log::error!("Magic Mount critical failure: {:#}", e);
            final_magic_ids.clear();
        }
        
        utils::cleanup_temp_dir(&tempdir);
    }

    final_overlay_ids.sort();
    final_overlay_ids.dedup();
    final_magic_ids.sort();
    final_magic_ids.dedup();

    Ok(ExecutionResult {
        overlay_module_ids: final_overlay_ids,
        magic_module_ids: final_magic_ids,
    })
}
