// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

mod api;
mod auth;
mod config;
mod defs;
mod scanner;

use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::Path,
};

use ::api::ConnectionInfo;
use anyhow::Result;
use tokio::{io::AsyncWriteExt, net::TcpListener};

fn init_logger() {
    #[cfg(not(target_os = "android"))]
    {
        let mut builder = env_logger::Builder::new();

        builder.format(|buf, record| {
            writeln!(
                buf,
                "[{}] [{}] {}",
                record.level(),
                record.target(),
                record.args()
            )
        });
        builder.filter_level(log::LevelFilter::Debug).init();
    }

    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("MagicMount"),
        );
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();

    run().await
}

async fn run() -> Result<()> {
    let _instance_lock = acquire_instance_lock(Path::new(defs::LOCK_FILE))?;
    remove_connection_file(defs::CONNECTION_FILE).await?;
    let token = auth::generate_token()?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    log::info!("daemon listening on 127.0.0.1:{port}");
    let router = api::router(
        api::AppState::production(token.clone())
            .initialize()
            .await?,
    );
    write_connection_file(
        Path::new(defs::CONNECTION_FILE),
        &ConnectionInfo { port, token },
    )
    .await?;
    log::info!("daemon discovery file published");

    let serve_result = axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await;
    let cleanup_result = remove_connection_file(defs::CONNECTION_FILE).await;
    match (serve_result, cleanup_result) {
        (Ok(()), Ok(())) => {
            log::info!("daemon stopped cleanly");
            Ok(())
        }
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error.into()),
        (Err(serve), Err(cleanup)) => Err(anyhow::anyhow!(
            "server failed: {serve}; failed to remove discovery file: {cleanup}"
        )),
    }
}

fn acquire_instance_lock(path: &Path) -> Result<File> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("lock file has no parent"))?;
    std::fs::create_dir_all(parent)?;
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(path)?;
    rustix::fs::flock(&file, rustix::fs::FlockOperation::NonBlockingLockExclusive)
        .map_err(|error| anyhow::anyhow!("another daemon instance is already running: {error}"))?;
    file.set_len(0)?;
    writeln!(file, "{}", std::process::id())?;
    file.sync_all()?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(file)
}

async fn write_connection_file(path: &Path, info: &ConnectionInfo) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("connection file has no parent"))?;
    tokio::fs::create_dir_all(parent).await?;
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(format!(".tmp-{:016x}", fastrand::u64(..)));
    let temporary = Path::new(&temporary);
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(temporary)
        .await?;
    let publish_result = async {
        file.write_all(&serde_json::to_vec(info)?).await?;
        file.sync_all().await?;
        drop(file);
        tokio::fs::rename(temporary, path).await
    }
    .await;
    if let Err(error) = publish_result {
        if let Err(cleanup) = remove_connection_file(temporary).await {
            return Err(anyhow::anyhow!(
                "failed to publish discovery file: {error}; failed to remove temporary file: {cleanup}"
            ));
        }
        return Err(error.into());
    }
    if let Err(error) =
        tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).await
    {
        if let Err(cleanup) = remove_connection_file(path).await {
            return Err(anyhow::anyhow!(
                "failed to secure discovery file: {error}; failed to remove it: {cleanup}"
            ));
        }
        return Err(error.into());
    }
    Ok(())
}

async fn remove_connection_file(path: impl AsRef<Path>) -> io::Result<()> {
    match tokio::fs::remove_file(path).await {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

#[cfg(unix)]
async fn shutdown_signal() {
    let mut terminate =
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(terminate) => terminate,
            Err(error) => {
                log::error!("failed to install SIGTERM handler: {error}");
                wait_for_ctrl_c().await;
                return;
            }
        };
    tokio::select! {
        () = wait_for_ctrl_c() => log::info!("received Ctrl-C shutdown signal"),
        _ = terminate.recv() => log::info!("received SIGTERM shutdown signal"),
    }
}

async fn wait_for_ctrl_c() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        log::error!("failed to install Ctrl-C handler: {error}");
        std::future::pending::<()>().await;
    }
}

#[cfg(not(unix))]
async fn shutdown_signal() {
    wait_for_ctrl_c().await;
}

#[cfg(test)]
#[path = "../../../tests/unit/daemon/main.rs"]
mod tests;
