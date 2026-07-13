#![allow(dead_code)]

use crate::utils::email as email_utils;
use unicode_normalization::UnicodeNormalization;

pub fn normalize_email(email: &str) -> String {
    let normalized = email.trim().nfc().collect::<String>();
    email_utils::normalize_email(&normalized).unwrap_or_else(|| normalized.to_lowercase())
}

pub fn normalize_cpf(cpf: &str) -> String {
    cpf.chars()
        .filter(|character| character.is_ascii_digit())
        .collect()
}

pub fn normalize_phone(phone: &str) -> String {
    phone
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{normalize_cpf, normalize_phone};

    #[test]
    fn cpf_normalized_to_digits() {
        assert_eq!(normalize_cpf(" 123.456.789-00 "), "12345678900");
    }

    #[test]
    fn phone_normalized_to_digits() {
        assert_eq!(normalize_phone(" +55 (11) 99999-0000 "), "5511999990000");
    }
}
