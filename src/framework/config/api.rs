// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use anyhow::Context;
use hex::decode;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::Config;
use crate::framework::{Result, paths};

#[derive(Debug, Serialize)]
struct ApiConfig {
    moduledir: String,
    mountsource: String,
    partitions: Vec<String>,
    extra_mount: Vec<String>,
    umount: bool,
    disable_umount: bool,
}

#[derive(Debug, Deserialize)]
struct ApiConfigPayload {
    mountsource: Option<String>,
    partitions: Option<Vec<String>>,
    extra_mount: Option<Vec<String>>,
    umount: Option<bool>,
    disable_umount: Option<bool>,
}

impl Config {
    fn into_api(self) -> ApiConfig {
        let umount_enabled = self.umount_enabled();

        ApiConfig {
            moduledir: paths::MODULE_PATH.trim_end_matches('/').to_string(),
            mountsource: self.mountsource,
            partitions: self.partitions,
            extra_mount: self.extra_mount,
            umount: umount_enabled,
            disable_umount: !umount_enabled,
        }
    }

    fn apply_api_payload(&mut self, payload: ApiConfigPayload) {
        if let Some(mountsource) = payload.mountsource {
            self.mountsource = mountsource;
        }

        if let Some(partitions) = payload.partitions {
            self.partitions = partitions;
        }

        if let Some(extra_mount) = payload.extra_mount {
            self.extra_mount = extra_mount;
        }

        if let Some(umount) = payload.umount {
            self.set_umount_enabled(umount);
        } else if let Some(disable_umount) = payload.disable_umount {
            self.set_umount_enabled(!disable_umount);
        }
    }
}

pub fn handle_show_config() -> Result<()> {
    let config = Config::load_or_default(paths::CONFIG_FILE);

    println!("{}", serde_json::to_string(&config.into_api())?);
    Ok(())
}

pub fn handle_save_config(payload_hex: &str) -> Result<()> {
    let payload_json =
        String::from_utf8(decode(payload_hex)?).context("decoded payload is not valid utf-8")?;
    let payload: ApiConfigPayload =
        serde_json::from_str(&payload_json).context("failed to parse config payload json")?;

    let mut config = Config::load_or_default(paths::CONFIG_FILE);
    config.apply_api_payload(payload);
    config.save(paths::CONFIG_FILE)?;

    println!("{}", json!({ "ok": true }));
    Ok(())
}

pub fn handle_gen_config() -> Result<()> {
    let config = Config::default();
    config.save(paths::CONFIG_FILE)?;
    println!("{}", json!({ "ok": true }));
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/unit/config_api.rs"]
mod tests;
