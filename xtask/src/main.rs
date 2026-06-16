// Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
// SPDX-License-Identifier: Apache-2.0

mod zip_ext;

use std::{
    env,
    ffi::CString,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use ed25519_dalek::SigningKey;
use fs_extra::{dir, file};
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use zip::{CompressionMethod, write::FileOptions};

use crate::zip_ext::zip_create_from_directory_with_options;

type SignFunc = unsafe extern "C" fn(*const i8, *const i8) -> i32;

#[derive(Deserialize)]
struct Package {
    version: String,
}

#[derive(Deserialize)]
struct CargoConfig {
    package: Package,
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

#[derive(Debug, ValueEnum, Clone)]
enum Targets {
    Arm64,
    Armv7,
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
    fn to_str(&self) -> &'static str {
        match self {
            Self::Arm64 => "arm64",
            Self::Armv7 => "armv7",
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

    let json = UpdateJson {
        versioncode: cal_version_code(&data.package.version)?,
        // Fixed typo here as well
        version: data.package.version.clone(),
        zipurl: format!(
            "https://github.com/Tools-cx-app/meta-magic_mount-rs/releases/download/v{}/magic_mount_rs-{}-{}.zip",
            data.package.version.clone(),
            &data.package.version,
            &cal_git_code()?
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
    let mut cargo = cargo_ndk();
    cargo.args(["check", "-Z", "build-std", "-Z", "trim-paths"]);
    cargo.env("RUSTFLAGS", "-C default-linker-libraries");

    if verbose {
        cargo.arg("--verbose");
    }

    cargo.spawn()?.wait()?;

    Ok(())
}

fn clean() -> Result<()> {
    let temp_dir = temp_dir();
    let _ = fs::remove_dir_all(&temp_dir);

    Command::new("cargo").arg("clean").spawn()?.wait()?;

    Ok(())
}

fn lint(fix: bool) -> Result<()> {
    let command_builder = |fix: bool| {
        let mut command = cargo_ndk();
        command.arg("clippy");
        if fix {
            command.args(["--fix", "--allow-dirty", "--allow-staged", "--all"]);
        }
        command
    };

    command_builder(fix).spawn()?.wait()?;
    command_builder(fix).arg("--release").spawn()?.wait()?;

    Ok(())
}

fn format(verbose: bool) -> Result<()> {
    let mut command = Command::new("cargo");
    command.args(["fmt", "--all"]);
    if verbose {
        command.arg("--verbose");
    }
    command.spawn()?.wait()?;

    Ok(())
}

fn match_build(verbose: bool, target: Targets) -> Result<()> {
    let temp_dir = temp_dir();
    let bin_path = temp_dir.join("bin");
    let toml = fs::read_to_string("Cargo.toml")?;
    let data: CargoConfig = toml::from_str(&toml)?;
    let (priv_key, pub_key) = generate_key()?;

    unsafe {
        env::set_var("PUB_KEY", pub_key.to_string_lossy().to_string());
    }

    let _ = fs::remove_dir_all(&temp_dir);
    let _ = fs::create_dir_all(&temp_dir);
    let _ = fs::create_dir_all(&bin_path);
    build(verbose)?;
    match target {
        Targets::Arm64 => {
            file::copy(
                aarch64_bin_path(),
                bin_path.join("magic_mount_rs.aarch64"),
                &file::CopyOptions::new().overwrite(true),
            )?;
            fs::remove_dir_all(temp_dir.join("libs").join("arm"))?;
        }
        Targets::Armv7 => {
            file::copy(
                armv7_bin_path(),
                bin_path.join("magic_mount_rs.armv7"),
                &file::CopyOptions::new().overwrite(true),
            )?;
            fs::remove_dir_all(temp_dir.join("libs").join("arm64-v8a"))?;
        }
        Targets::Universal => {
            file::copy(
                armv7_bin_path(),
                bin_path.join("magic_mount_rs.armv7"),
                &file::CopyOptions::new().overwrite(true),
            )?;

            file::copy(
                aarch64_bin_path(),
                bin_path.join("magic_mount_rs.aarch64"),
                &file::CopyOptions::new().overwrite(true),
            )?;
        }
    }

    generate_sign(priv_key)?;

    let options: FileOptions<'_, ()> = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(9));
    zip_create_from_directory_with_options(
        &Path::new("output").join(format!(
            "magic_mount_rs-{}-{}-{}.zip",
            &data.package.version,
            &cal_git_code()?,
            target.to_str()
        )),
        &temp_dir,
        |_| options,
    )
    .unwrap();

    Ok(())
}

fn build(verbose: bool) -> Result<()> {
    let temp_dir = temp_dir();

    build_webui()?;

    let mut cargo = cargo_ndk();
    let args = vec![
        "build",
        "-Z",
        "build-std=std,core,panic_abort",
        "-Z",
        "build-std-features=optimize_for_size",
        "-Z",
        "trim-paths",
        "-r",
    ];

    if verbose {
        cargo.arg("--verbose");
    }

    cargo.args(args);

    cargo.spawn()?.wait()?;

    let module_dir = module_dir();
    dir::copy(
        &module_dir,
        &temp_dir,
        &dir::CopyOptions::new().overwrite(true).content_only(true),
    )
    .unwrap();

    if temp_dir.join(".gitignore").exists() {
        fs::remove_file(temp_dir.join(".gitignore")).unwrap();
    }
    if temp_dir.join("signature").exists() {
        fs::remove_file(temp_dir.join("signature")).unwrap();
    }
    Ok(())
}

fn module_dir() -> PathBuf {
    Path::new("module").to_path_buf()
}

fn generate_key() -> Result<(CString, CString)> {
    let mut seed = [0u8; 32];

    getrandom::fill(&mut seed)?;
    let priv_key = SigningKey::from_bytes(&seed);
    let pub_key = priv_key.verifying_key();

    let hex_cstring = |bytes: &[u8]| -> CString {
        let hex_string: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        CString::new(hex_string).unwrap()
    };
    let priv_key = hex_cstring(&priv_key.to_keypair_bytes());
    let pub_key = hex_cstring(&pub_key.to_bytes());

    Ok((priv_key, pub_key))
}

fn generate_sign(key: CString) -> Result<()> {
    #[cfg(target_arch = "x86_64")]
    let lib = unsafe { Library::new("libs/x86_64/libchecker.so")? };
    #[cfg(target_arch = "aarch64")]
    let lib = unsafe { Library::new("libs/arm64-v8a/libchecker.so")? };
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    compile_error!("unsupported host arch, please use arm64-v8a/x86_64");

    let generate_sign: Symbol<SignFunc> = unsafe { lib.get(b"GenerateSign")? };

    let path = CString::new(temp_dir().to_string_lossy().to_string())?;

    if unsafe { generate_sign(key.as_ptr() as *const i8, path.as_ptr() as *const i8) } < 0 {
        eprintln!("failed to generate sign");
    }

    Ok(())
}

fn temp_dir() -> PathBuf {
    Path::new("output").join(".temp")
}

fn aarch64_bin_path() -> PathBuf {
    Path::new("target")
        .join("aarch64-linux-android")
        .join("release")
        .join("magic_mount_rs")
}

fn armv7_bin_path() -> PathBuf {
    Path::new("target")
        .join("armv7-linux-androideabi")
        .join("release")
        .join("magic_mount_rs")
}

fn cargo_ndk() -> Command {
    let mut command = Command::new("cargo");
    command
        .args([
            "+nightly",
            "ndk",
            "--platform",
            "26",
            "-t",
            "arm64-v8a",
            "-t",
            "armeabi-v7a",
        ])
        .env("RUSTFLAGS", "-C default-linker-libraries");
    command
}

fn build_webui() -> Result<()> {
    let pnpm = || {
        let mut command = Command::new("pnpm");
        command.current_dir("webui");
        command
    };

    pnpm().args(["run", "build"]).spawn()?.wait()?;

    Ok(())
}
