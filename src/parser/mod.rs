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

mod mount;

use std::{fs, path::Path, sync::OnceLock};

pub use crate::parser::mount::Command;
use crate::parser::mount::parse_command;

pub static COMMAND_LIST: OnceLock<Vec<Command>> = OnceLock::new();

pub fn parser_custom<P>(path: P) -> Vec<Command>
where
    P: AsRef<Path>,
{
    fs::read_to_string(path.as_ref()).map_or_else(
        |_| Vec::new(),
        |s| {
            s.lines()
                .map(str::trim)
                .filter(|s| !s.starts_with('#'))
                .filter_map(parse_command)
                .collect()
        },
    )
}
