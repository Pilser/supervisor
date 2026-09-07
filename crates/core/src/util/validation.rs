//! Input validation helpers.

/// Validate a generic id `[a-zA-Z0-9_-]+`.
pub fn is_valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Validate a username.
pub fn is_valid_username(username: &str) -> bool {
    let len = username.len();
    (3..=64).contains(&len)
        && username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-')
}