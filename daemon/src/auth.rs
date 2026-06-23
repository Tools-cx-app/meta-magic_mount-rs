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
mod tests {
    use super::{generate_token, is_authorized};

    #[test]
    fn generated_tokens_are_distinct_hex_secrets() {
        let first = generate_token().unwrap();
        let second = generate_token().unwrap();
        assert_eq!(first.len(), 64);
        assert!(first.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert_ne!(first, second);
    }

    #[test]
    fn bearer_token_must_match_exactly() {
        assert!(is_authorized("secret", Some("Bearer secret")));
        assert!(!is_authorized("secret", Some("Bearer Secret")));
        assert!(!is_authorized("secret", Some("Basic secret")));
        assert!(!is_authorized("secret", None));
    }
}
