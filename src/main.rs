mod conf;
mod core;
mod defs;
mod mount;
mod utils;

use std::path::{Path, PathBuf};
use anyhow::Result;
use clap::Parser;
use mimalloc::MiMalloc;

use conf::{
    cli::{Cli, Commands},
    config::{Config, CONFIG_FILE_DEFAULT},
};
use core::{
    executor,
    inventory,
    planner,
    state::RuntimeState,
    storage,
    sync,
    modules,
};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn load_config(cli: &Cli) -> Result<Config> {
    if let Some(config_path) = &cli.config {
        return Config::from_file(config_path);
    }
    match Config::load_default() {
        Ok(config) => Ok(config),
        Err(e) => {
            if Path::new(CONFIG_FILE_DEFAULT).exists() {
                eprintln!("Error loading config: {:#}", e);
            }
            Ok(Config::default())
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    if let Some(command) = &cli.command {
        match command {
            Commands::GenConfig { output } => { 
                Config::default().save_to_file(output)?; 
                return Ok(()); 
            },
            Commands::ShowConfig => { 
                let config = load_config(&cli)?;
                println!("{}", serde_json::to_string(&config)?); 
                return Ok(()); 
            },
            Commands::Storage => { 
                storage::print_status()?; 
                return Ok(()); 
            },
            Commands::Modules => { 
                let config = load_config(&cli)?;
                modules::print_list(&config)?; 
                return Ok(()); 
            }
        }
    }

    let mut config = load_config(&cli)?;
    config.merge_with_cli(
        cli.moduledir.clone(), 
        cli.tempdir.clone(), 
        cli.mountsource.clone(), 
        cli.verbose, 
        cli.partitions.clone()
    );

    let _log_guard = utils::init_logging(config.verbose, Path::new(defs::DAEMON_LOG_FILE))?;

    if let Err(e) = utils::camouflage_process("kworker/u9:1") {
        log::warn!("Failed to camouflage process: {}", e);
    }

    log::info!("Meta-Hybrid Mount Daemon Starting...");

    if config.disable_umount {
        log::warn!("Namespace Detach (try_umount) is DISABLED.");
    }

    utils::ensure_dir_exists(defs::RUN_DIR)?;

    let mnt_base = PathBuf::from(defs::FALLBACK_CONTENT_DIR);
    let img_path = Path::new(defs::BASE_DIR).join("modules.img");
    
    let storage_handle = storage::setup(&mnt_base, &img_path, config.force_ext4)?;

    let module_list = inventory::scan(&config.moduledir, &config)?;
    log::info!("Scanned {} active modules.", module_list.len());

    sync::perform_sync(&module_list, &storage_handle.mount_point)?;

    log::info!("Generating mount plan...");
    let plan = planner::generate(&config, &module_list, &storage_handle.mount_point)?;
    
    log::info!("Plan: {} OverlayFS ops, {} Magic modules", 
        plan.overlay_ops.len(), 
        plan.magic_module_paths.len()
    );

    let exec_result = executor::execute(&plan, &config)?;

    let mut nuke_active = false;
    if storage_handle.mode == "ext4" && config.enable_nuke {
        log::info!("Attempting to deploy Paw Pad (Stealth) via KernelSU...");
        match utils::ksu_nuke_sysfs(storage_handle.mount_point.to_string_lossy().as_ref()) {
            Ok(_) => {
                log::info!("Success: Paw Pad active. Ext4 sysfs traces nuked.");
                nuke_active = true;
            },
            Err(e) => {
                log::warn!("Paw Pad failed (KSU ioctl error): {}", e);
            }
        }
    }

    modules::update_description(
        &storage_handle.mode, 
        nuke_active, 
        exec_result.overlay_module_ids.len(), 
        exec_result.magic_module_ids.len()
    );

    let state = RuntimeState::new(
        storage_handle.mode,
        storage_handle.mount_point,
        exec_result.overlay_module_ids,
        exec_result.magic_module_ids,
        nuke_active
    );
    
    if let Err(e) = state.save() {
        log::error!("Failed to save runtime state: {}", e);
    }

    log::info!("Meta-Hybrid Mount Completed.");
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        log::error!("Fatal Error: {:#}", e);
        eprintln!("Fatal Error: {:#}", e);
        std::process::exit(1);
    }
}
