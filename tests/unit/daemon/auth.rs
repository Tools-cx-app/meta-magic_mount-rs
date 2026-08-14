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
