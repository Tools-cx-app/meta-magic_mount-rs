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
