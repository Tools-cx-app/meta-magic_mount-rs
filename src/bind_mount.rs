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
