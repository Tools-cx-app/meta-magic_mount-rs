// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

mod api;
mod core;
mod header;

use std::cell::RefCell;

use rustc_hash::{FxHashMap, FxHashSet};

use self::core::Script;
use crate::errors::{Error, Result};
use crate::scanner::AppModule;
use api::{ApiV1, ApiV2};
pub use api::{BindMount, BindPhase, ModulePlan};

#[derive(Default)]
pub struct Extensions {
    bind_scripts: Vec<Script<ApiV1>>,
    module_scripts: FxHashMap<String, Vec<Script<ApiV2>>>,
    visited: RefCell<FxHashSet<String>>,
}

impl Extensions {
    pub fn bind_mounts(&self, phase: BindPhase) -> Result<Vec<BindMount>> {
        let mut commands = Vec::new();
        for script in &self.bind_scripts {
            script.bind_mounts(phase, &mut commands)?;
        }
        Ok(commands)
    }

    /// Called at the scan boundary, before examining this module's mount content.
    pub fn scan(&self, module: &mut AppModule, extra_mount: &[String]) -> Result<ModulePlan> {
        let mut plan = ModulePlan::default();
        if !self.visited.borrow_mut().insert(module.id.clone()) {
            return Err(Error::DuplicateModuleScan {
                module: module.id.clone(),
            });
        }
        if let Some(scripts) = self.module_scripts.get(&module.id) {
            for script in scripts {
                script.load_module(&module.id, &mut plan, extra_mount)?;
                if plan.skip {
                    module.skip_mount();
                    return Ok(plan);
                }
            }
        }
        for path in &plan.blocked {
            module.record_rule(path, "block");
        }
        for path in &plan.removed {
            module.record_rule(path, "remove");
        }
        for path in plan.replacements.keys() {
            if !plan.blocks(path) && !plan.removes(path) {
                module.record_rule(path, "replace");
            }
        }
        Ok(plan)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/extension.rs"]
mod tests;
