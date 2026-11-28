// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

mod config;
mod error;
mod extension;
mod mount;
pub(crate) mod paths;
mod runtime;
mod scanner;

pub(crate) use config::{handle_gen_config, handle_save_config, handle_show_config};
pub(crate) use error::Result;
pub(crate) use mount::emulated_soft_reboot;
pub(crate) use runtime::Runtime;
