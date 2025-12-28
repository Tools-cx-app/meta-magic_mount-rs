use std::{fmt, fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::defs::CONFIG_FILE;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_moduledir")]
    pub moduledir: PathBuf,
    #[serde(default = "default_mountsource")]
    pub mountsource: String,
    pub verbose: bool,
    pub partitions: Vec<String>,
    pub tmpfsdir: Option<String>,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub umount: bool,
}

fn default_moduledir() -> PathBuf {
    PathBuf::from("/data/adb/modules/")
}

fn default_mountsource() -> String {
    String::from("KSU")
}

impl Default for Config {
    fn default() -> Self {
        Self {
            moduledir: default_moduledir(),
            mountsource: default_mountsource(),
            verbose: false,
            tmpfsdir: None,
            umount: false,
            partitions: Vec::new(),
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "module real path: {}", self.moduledir.display())?;
        writeln!(f, "mount source: {}", self.mountsource)?;
        if self.verbose {
            writeln!(f, "u enable debug mode!!")?;
        }
        if self.partitions.is_empty() {
            write!(f, "no extra partitions")
        } else {
            write!(f, "extra partitions is {:?}", self.partitions)
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let content = fs::read_to_string(CONFIG_FILE).context("failed to read config file")?;

        let config: Self = toml::from_str(&content).context("failed to parse config file")?;

        Ok(config)
    }
}
