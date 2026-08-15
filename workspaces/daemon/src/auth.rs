// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::io;

pub fn generate_token() -> io::Result<String> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(|error| io::Error::other(error.to_string()))?;
    Ok(hex::encode(bytes))
}

pub fn is_authorized(expected: &str, header: Option<&str>) -> bool {
    let Some(actual) = header.and_then(|value| value.strip_prefix("Bearer ")) else {
        return false;
    };
    actual.len() == expected.len()
        && actual
            .bytes()
            .zip(expected.bytes())
            .fold(0_u8, |difference, (left, right)| {
                difference | (left ^ right)
            })
            == 0
}

#[cfg(test)]
#[path = "../../tests/unit/daemon/auth.rs"]
mod tests;
