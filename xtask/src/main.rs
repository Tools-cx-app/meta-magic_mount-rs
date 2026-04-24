// Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod zip_ext;

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use fs_extra::{dir, file};
use serde::{Deserialize, Serialize};
use zip::{CompressionMethod, write::FileOptions};

use crate::zip_ext::zip_create_from_directory_with_options;

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

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { verbose } => {
            check(verbose)?;
        }
        Commands::Build { verbose } => {
            build(verbose)?;
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

fn build(verbose: bool) -> Result<()> {
    let temp_dir = temp_dir();
    let toml = fs::read_to_string("Cargo.toml")?;
    let data: CargoConfig = toml::from_str(&toml)?;

    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir)?;

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

    let bin_path = temp_dir.join("bin");

    let _ = fs::create_dir_all(&bin_path);
    file::copy(
        aarch64_bin_path(),
        bin_path.join("magic_mount_rs.aarch64"),
        &file::CopyOptions::new().overwrite(true),
    )?;
    file::copy(
        armv7_bin_path(),
        bin_path.join("magic_mount_rs.armv7"),
        &file::CopyOptions::new().overwrite(true),
    )?;
    let options: FileOptions<'_, ()> = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(9));
    zip_create_from_directory_with_options(
        &Path::new("output").join(format!(
            "magic_mount_rs-{}-{}.zip",
            &data.package.version,
            &cal_git_code()?
        )),
        &temp_dir,
        |_| options,
    )
    .unwrap();

    Ok(())
}

fn module_dir() -> PathBuf {
    Path::new("module").to_path_buf()
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
