use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use anyhow::Result;
use serde::Serialize;
use crate::{conf::config, defs, core::state};

#[derive(Serialize)]
struct ModuleInfo {
    id: String,
    name: String,
    version: String,
    author: String,
    description: String,
    mode: String,
}

fn read_prop(path: &Path, key: &str) -> Option<String> {
    if let Ok(file) = fs::File::open(path) {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            if line.starts_with(key) && line.chars().nth(key.len()) == Some('=') {
                return Some(line[key.len() + 1..].to_string());
            }
        }
    }
    None
}

fn has_files_recursive(path: &Path) -> bool {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            if file_type.is_dir() {
                if has_files_recursive(&entry.path()) {
                    return true;
                }
            } else {
                return true;
            }
        }
    }
    false
}

pub fn update_description(storage_mode: &str, nuke_active: bool, overlay_count: usize, magic_count: usize) {
    let path = Path::new(defs::MODULE_PROP_FILE);
    if !path.exists() { 
        log::warn!("module.prop not found at {}, skipping description update", path.display());
        return; 
    }

    let mode_str = if storage_mode == "tmpfs" { "Tmpfs" } else { "Ext4" };
    let status_emoji = if storage_mode == "tmpfs" { "🐾" } else { "💿" };
    
    let nuke_str = if nuke_active { " | 肉垫: 开启 ✨" } else { "" };
    
    let new_desc = format!(
        "description=😋 运行中喵～ ({}) {} | Overlay: {} | Magic: {}{}", 
        mode_str, status_emoji, overlay_count, magic_count, nuke_str
    );

    let mut new_lines = Vec::new();
    match fs::read_to_string(path) {
        Ok(content) => {
            for line in content.lines() {
                if line.starts_with("description=") {
                    new_lines.push(new_desc.clone());
                } else {
                    new_lines.push(line.to_string());
                }
            }
            if let Err(e) = fs::write(path, new_lines.join("\n")) {
                log::error!("Failed to update module.prop: {}", e);
            } else {
                log::info!("Updated module.prop description (Meow!).");
            }
        },
        Err(e) => log::error!("Failed to read module.prop: {}", e),
    }
}

pub fn print_list(config: &config::Config) -> Result<()> {
    let module_modes = config::load_module_modes();
    let modules_dir = &config.moduledir;
    let mut modules = Vec::new();

    let state = state::RuntimeState::load().unwrap_or_default();
    
    let mut mnt_base = PathBuf::from(defs::FALLBACK_CONTENT_DIR);
    if !state.mount_point.as_os_str().is_empty() {
        mnt_base = state.mount_point;
    }

    if modules_dir.exists() {
        for entry in fs::read_dir(modules_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() { continue; }
            let id = entry.file_name().to_string_lossy().to_string();
            if id == "meta-hybrid" || id == "lost+found" { continue; }
            if path.join(defs::DISABLE_FILE_NAME).exists() || path.join(defs::REMOVE_FILE_NAME).exists() || path.join(defs::SKIP_MOUNT_FILE_NAME).exists() { continue; }

            let has_content = defs::BUILTIN_PARTITIONS.iter().any(|p| {
                let p_src = path.join(p);
                let p_dst = mnt_base.join(&id).join(p);
                (p_src.exists() && has_files_recursive(&p_src)) || 
                (p_dst.exists() && has_files_recursive(&p_dst))
            });

            if has_content {
                let prop_path = path.join("module.prop");
                let name = read_prop(&prop_path, "name").unwrap_or_else(|| id.clone());
                let version = read_prop(&prop_path, "version").unwrap_or_default();
                let author = read_prop(&prop_path, "author").unwrap_or_default();
                let description = read_prop(&prop_path, "description").unwrap_or_default();
                let mode = module_modes.get(&id).cloned().unwrap_or_else(|| "auto".to_string());
                modules.push(ModuleInfo { id, name, version, author, description, mode });
            }
        }
    }
    modules.sort_by(|a, b| a.name.cmp(&b.name));
    println!("{}", serde_json::to_string(&modules)?);
    Ok(())
}
