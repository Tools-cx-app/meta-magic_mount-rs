// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

mod zip_ext;

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use fs_extra::{dir, file};
use serde::{Deserialize, Serialize};
use zip::{CompressionMethod, write::FileOptions};

use sha2::{Digest, Sha256};
use std::io::{BufReader, Read};

use crate::zip_ext::zip_create_from_directory_with_options;

#[derive(Deserialize)]
struct Package {
    name: String,
}

#[derive(Deserialize)]
struct CargoConfig {
    package: Package,
    workspace: Workspace,
}

#[derive(Deserialize)]
struct Workspace {
    package: WorkspacePackage,
}

#[derive(Deserialize)]
struct WorkspacePackage {
    version: String,
}

impl CargoConfig {
    fn version(&self) -> &str {
        &self.workspace.package.version
    }
}

#[derive(Serialize)]
struct UpdateJson {
    version: String,
    #[serde(rename = "versionCode")]
    versioncode: usize,
    #[serde(rename = "zipUrl")]
    zipurl: String,
    changelog: String,
}

#[derive(Parser)]
#[command(version, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, ValueEnum, Copy, Clone)]
enum Targets {
    Arm64,
    Armv7,
    X86_64,
    Universal,
}

#[derive(Subcommand)]
enum Commands {
    /// Check the build of mmrs
    Check {
        /// Print detailed output (default: false)
        #[clap(short, long, default_value = "false")]
        verbose: bool,
    },

    /// Build mmrs
    Build {
        /// Build target (default: arm64)
        #[clap(short, long, default_value = "arm64")]
        target: Targets,
        /// Print detailed output (default: false)
        #[clap(short, long, default_value = "false")]
        verbose: bool,
    },

    /// Clean build artifacts
    Clean,

    /// Format source code
    Format {
        /// Print detailed output (default: false)
        #[clap(short, long, default_value = "false")]
        verbose: bool,
    },

    /// Run the Clippy linter
    Lint {
        /// Automatically fix lint issues (default: false)
        #[clap(short, long, default_value = "false")]
        fix: bool,
    },

    /// Update versionCode/url in update/update.json
    Update,
}

impl Targets {
    fn to_str(self) -> &'static str {
        match self {
            Self::Arm64 => "arm64",
            Self::Armv7 => "armv7",
            Self::X86_64 => "x86_64",
            Self::Universal => "universal",
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { verbose } => {
            check(verbose)?;
        }
        Commands::Build { verbose, target } => {
            match_build(verbose, target)?;
        }
        Commands::Clean => {
            clean()?;
        }
        Commands::Format { verbose } => {
            format(verbose)?;
        }
        Commands::Lint { fix } => {
            lint(fix)?;
        }
        Commands::Update => {
            update()?;
        }
    }

    Ok(())
}

fn cal_version_code(version: &str) -> Result<usize> {
    let manjor = version
        .split('.')
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid version format"))?;
    let manjor: usize = manjor.parse()?;
    let minor = version
        .split('.')
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Invalid version format"))?;
    let minor: usize = minor.parse()?;
    let patch = version
        .split('.')
        .nth(2)
        .ok_or_else(|| anyhow::anyhow!("Invalid version format"))?;
    let patch: usize = patch.parse()?;

    // Version code rule: Major * 100000 + Minor * 1000 + Patch
    Ok(manjor * 100000 + minor * 1000 + patch)
}

fn cal_git_code() -> Result<i32> {
    Ok(String::from_utf8(
        Command::new("git")
            .args(["rev-list", "--count", "HEAD"])
            .output()?
            .stdout,
    )?
    .trim()
    .parse::<i32>()?)
}

fn update() -> Result<()> {
    let toml = fs::read_to_string("Cargo.toml")?;
    let data: CargoConfig = toml::from_str(&toml)?;

    //build()?;

    let version = data.version();
    let json = UpdateJson {
        versioncode: cal_version_code(version)?,
        // Fixed typo here as well
        version: version.to_owned(),
        zipurl: format!(
            "https://github.com/Tools-cx-app/meta-magic_mount-rs/releases/download/v{}/magic_mount_rs-{}-{}-universal.zip",
            version,
            version,
            cal_git_code()?
        ),
        changelog: String::from(
            "https://github.com/Tools-cx-app/meta-magic_mount-rs/raw/master/update/changelog.md",
        ),
    };

    let raw_json = serde_json::to_string_pretty(&json)?;

    fs::write("update/update.json", raw_json)?;

    Ok(())
}

fn check(verbose: bool) -> Result<()> {
    let mut cargo = cargo_ndk(Targets::Universal);
    cargo.args([
        "check",
        "--workspace",
        "--exclude",
        "xtask",
        "-Z",
        "build-std",
        "-Z",
        "trim-paths",
    ]);
    cargo.env("RUSTFLAGS", "-C default-linker-libraries");

    if verbose {
        cargo.arg("--verbose");
    }

    ensure_success(cargo.spawn()?.wait()?, "cargo ndk check")?;

    Ok(())
}

fn clean() -> Result<()> {
    let temp_dir = temp_dir();
    let _ = fs::remove_dir_all(&temp_dir);

    ensure_success(
        Command::new("cargo").arg("clean").spawn()?.wait()?,
        "cargo clean",
    )?;

    Ok(())
}

fn lint(fix: bool) -> Result<()> {
    let command_builder = |fix: bool, release: bool| {
        let mut command = cargo_ndk(Targets::Universal);
        command.args(["clippy", "--workspace", "--exclude", "xtask"]);
        if release {
            command.arg("--release");
        }
        if fix {
            command.args(["--fix", "--allow-dirty", "--allow-staged", "--all"]);
        }
        command.args(["--", "-D", "warnings"]);
        command
    };

    ensure_success(
        command_builder(fix, false).spawn()?.wait()?,
        "cargo ndk clippy",
    )?;
    ensure_success(
        command_builder(fix, true).spawn()?.wait()?,
        "cargo ndk clippy --release",
    )?;

    Ok(())
}

fn format(verbose: bool) -> Result<()> {
    let mut command = Command::new("cargo");
    command.args(["fmt", "--all"]);
    if verbose {
        command.arg("--verbose");
    }
    ensure_success(command.spawn()?.wait()?, "cargo fmt")?;

    Ok(())
}

fn match_build(verbose: bool, target: Targets) -> Result<()> {
    let temp_dir = temp_dir();
    let bin_path = temp_dir.join("bin");
    let toml = fs::read_to_string("Cargo.toml")?;
    let data: CargoConfig = toml::from_str(&toml)?;

    if let Err(error) = fs::remove_dir_all(&temp_dir)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        return Err(error.into());
    }
    fs::create_dir_all(&bin_path)?;
    build(verbose, target, data.package.name.clone())?;
    let targets: &[(&str, &str)] = match target {
        Targets::Arm64 => &[("arm64-v8a", "aarch64-linux-android")],
        Targets::Armv7 => &[("armeabi-v7a", "armv7-linux-androideabi")],
        Targets::X86_64 => &[("x86_64", "x86_64-linux-android")],
        Targets::Universal => &[
            ("arm64-v8a", "aarch64-linux-android"),
            ("armeabi-v7a", "armv7-linux-androideabi"),
            ("x86_64", "x86_64-linux-android"),
        ],
    };
    for (abi, rust_target) in targets {
        let abi_dir = bin_path.join(abi);
        fs::create_dir_all(&abi_dir)?;
        for binary in ["magic_mount_rs", "daemon"] {
            file::copy(
                target_bin_path(rust_target, binary),
                abi_dir.join(binary),
                &file::CopyOptions::new().overwrite(true),
            )?;
        }
    }

    let mut vec_temp_dir: Vec<PathBuf> = vec![temp_dir.clone()];
    while let Some(current) = vec_temp_dir.pop() {
        for entry in fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                vec_temp_dir.push(path);
            } else {
                let mut hasher = Sha256::new();
                let file = fs::File::open(&path)?;
                let mut reader = BufReader::new(file);
                let mut buffer = [0; 8192];
                loop {
                    let n = reader.read(&mut buffer)?;
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]);
                }
                let hex: String = hasher
                    .finalize()
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect();
                let mut out_path = path.into_os_string();
                out_path.push(".sha256");
                fs::write(out_path, hex)?;
            }
        }
    }

    let options: FileOptions<'_, ()> = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(9));
    zip_create_from_directory_with_options(
        &Path::new("output").join(format!(
            "magic_mount_rs-{}-{}-{}.zip",
            data.version(),
            cal_git_code()?,
            target.to_str()
        )),
        &temp_dir,
        |_| options,
    )?;

    Ok(())
}

fn build(verbose: bool, target: Targets, name: String) -> Result<()> {
    let temp_dir = temp_dir();

    unsafe {
        std::env::set_var("MODULE_ID", name);
    }
    build_webui()?;
    unsafe {
        std::env::remove_var("MODULE_ID");
    }

    let mut cargo = cargo_ndk(target);
    let args = vec![
        "build",
        "-Z",
        "build-std=std,core,panic_abort",
        "-Z",
        "build-std-features=optimize_for_size",
        "-Z",
        "trim-paths",
        "-r",
        "-p",
        "magic_mount_rs",
        "-p",
        "daemon",
    ];

    if verbose {
        cargo.arg("--verbose");
    }

    cargo.args(args);

    ensure_success(cargo.spawn()?.wait()?, "cargo ndk build")?;

    let module_dir = module_dir();
    dir::copy(
        &module_dir,
        &temp_dir,
        &dir::CopyOptions::new().overwrite(true).content_only(true),
    )?;

    if temp_dir.join(".gitignore").exists() {
        fs::remove_file(temp_dir.join(".gitignore"))?;
    }

    Ok(())
}

fn module_dir() -> PathBuf {
    Path::new("module").to_path_buf()
}

fn temp_dir() -> PathBuf {
    Path::new("output").join(".temp")
}

fn target_bin_path(target: &str, binary: &str) -> PathBuf {
    Path::new("target")
        .join(target)
        .join("release")
        .join(binary)
}

fn ensure_success(status: std::process::ExitStatus, command: &str) -> Result<()> {
    anyhow::ensure!(status.success(), "{command} failed with {status}");
    Ok(())
}

fn cargo_ndk(target: Targets) -> Command {
    let mut command = Command::new("cargo");
    command
        .args([
            "+nightly",
            "ndk",
            "--platform",
            if matches!(target, Targets::Arm64 | Targets::X86_64) {
                "30"
            } else {
                "26"
            },
        ])
        .env("RUSTFLAGS", "-C default-linker-libraries");
    match target {
        Targets::Arm64 => {
            command.args(["-t", "arm64-v8a"]);
        }
        Targets::Armv7 => {
            command.args(["-t", "armeabi-v7a"]);
        }
        Targets::X86_64 => {
            command.args(["-t", "x86_64"]);
        }
        Targets::Universal => {
            command.args(["-t", "arm64-v8a", "-t", "x86_64", "-t", "armeabi-v7a"]);
        }
    }
    command
}

fn build_webui() -> Result<()> {
    let pnpm = || {
        let mut command = Command::new("pnpm");
        command.current_dir("webui");
        command
    };

    ensure_success(
        pnpm().args(["run", "build"]).spawn()?.wait()?,
        "pnpm run build",
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daemon_uses_the_same_target_directory_as_mount_binary() {
        assert_eq!(
            target_bin_path("aarch64-linux-android", "daemon"),
            Path::new("target/aarch64-linux-android/release/daemon")
        );
    }

    #[test]
    fn failed_cargo_ndk_status_is_an_error() {
        let status = Command::new("sh").args(["-c", "exit 7"]).status().unwrap();

        assert!(ensure_success(status, "cargo ndk build").is_err());
    }
}
