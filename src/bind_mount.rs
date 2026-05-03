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

use rustix::mount::mount_bind;

use crate::{errors::Result, parser::COMMAND_LIST};

pub fn bind_mount() -> Result<()> {
    let bind_mount_list: Vec<_> = COMMAND_LIST
        .get()
        .unwrap()
        .iter()
        .filter_map(|s| {
            if let crate::parser::Command::Mount { source, target } = s {
                Some((source.clone(), target.clone()))
            } else {
                None
            }
        })
        .collect();

    for (s, t) in bind_mount_list {
        mount_bind(s, t)?;
    }
    Ok(())
}
