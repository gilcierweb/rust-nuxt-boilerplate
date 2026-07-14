use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

/// Simulates the behavior of ActiveSupport's `String#parameterize` (Rails)
#[allow(dead_code)]
pub fn parameterize(text: &str) -> String {
    text.nfd()
        .filter(|c| c.is_ascii_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

/// parameterize + UUID suffix for guaranteed uniqueness
#[allow(dead_code)]
pub fn parameterize_unique(text: &str) -> String {
    let slug = parameterize(text);
    format!("{slug}-{}", Uuid::new_v4())
}

/// Normalize display name: remove control characters, preserve unicode
#[allow(dead_code)]
pub fn normalize_display_name(name: &str) -> String {
    name.trim()
        .nfc()
        .collect::<String>()
        .chars()
        .filter(|c| {
            c.is_alphanumeric()
                || c.is_whitespace()
                || matches!(c, '.' | ',' | '\'' | '-' | '(' | ')' | '!')
        })
        .collect()
}
