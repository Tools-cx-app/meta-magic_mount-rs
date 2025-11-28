use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub const CONFIG_FILE_DEFAULT: &str = "/data/adb/magic_mount/config.toml";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_moduledir")]
    pub moduledir: PathBuf,
    pub tempdir: Option<PathBuf>,
    #[serde(default = "default_mountsource")]
    pub mountsource: String,
    pub verbose: bool,
    pub partitions: Vec<String>,
    pub umount: bool,
}

fn default_moduledir() -> PathBuf {
    PathBuf::from("/data/adb/modules/")
}

fn default_mountsource() -> String {
    String::from("MaGIcMounT")
}

impl Default for Config {
    fn default() -> Self {
        Self {
            moduledir: default_moduledir(),
            tempdir: None,
            mountsource: default_mountsource(),
            verbose: false,
            umount: false,
            partitions: Vec::new(),
        }
    }
}

impl Config {
    pub fn load_default() -> Result<Self> {
        let content =
            fs::read_to_string(CONFIG_FILE_DEFAULT).context("failed to read config file")?;

        let config: Self = toml::from_str(&content).context("failed to parse config file")?;

        Ok(config)
    }
}
