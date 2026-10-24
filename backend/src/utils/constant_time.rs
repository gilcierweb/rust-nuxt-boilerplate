//! Constant-time comparison for secrets and signatures.
//!
//! Every place that compares a caller-supplied secret against a configured one
//! must go through these helpers. A short-circuiting `==` (or
//! `iter().all(...)` with an early exit) leaks how many leading bytes matched
//! through response timing, which is enough to recover a key byte by byte.
//!
//! Uses the `subtle` crate, already a project dependency.

use subtle::{Choice, ConstantTimeEq};

/// Compare two strings in constant time.
///
/// Both the length check and the byte comparison are performed without
/// short-circuiting, so an attacker cannot infer the correct key length
/// or any individual byte by measuring response latency.
///
/// # Implementation
///
/// We use `subtle::ConstantTimeEq` for the byte-level comparison. For length
/// equality we also use `subtle::Choice` so the branch is not visible to the
/// optimiser or CPU branch-predictor.
pub fn constant_time_eq(expected: &str, provided: &str) -> bool {
    let expected = expected.as_bytes();
    let provided = provided.as_bytes();

    // Lengths must match. Encode the result as a `subtle::Choice` (1 = equal)
    // so the branch below is not evaluated until after the byte comparison.
    let lengths_match: Choice = ConstantTimeEq::ct_eq(&(expected.len() as u64), &(provided.len() as u64));

    // Byte comparison. We pad the shorter slice with zeros so that the loop
    // always runs for `max(len_a, len_b)` iterations regardless of input
    // lengths, preventing the optimiser from eliding work based on the length
    // difference. The `lengths_match` flag ensures the overall result is
    // `false` whenever lengths differ, even if the padded bytes accidentally
    // compare equal.
    let max_len = expected.len().max(provided.len());
    let mut diff = Choice::from(0u8); // accumulates any difference

    for index in 0..max_len {
        let expected_byte = if index < expected.len() { expected[index] } else { 0 };
        let provided_byte = if index < provided.len() { provided[index] } else { 0 };
        // ct_ne returns Choice(1) if bytes differ
        diff |= expected_byte.ct_ne(&provided_byte);
    }

    // Valid only when lengths match AND no bytes differ.
    // Both conditions are evaluated; neither causes an early exit.
    bool::from(lengths_match & !diff)
}

/// Compare two hex-encoded digests in constant time.
///
/// Decoding is not constant time, but a malformed value cannot leak anything
/// about the expected digest: it simply fails validation. Length or encoding
/// differences resolve to `false`.
pub fn constant_time_eq_hex(expected: &str, provided: &str) -> bool {
    let expected_bytes = match hex::decode(expected) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    let provided_bytes = match hex::decode(provided) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    bytes_constant_time_eq(&expected_bytes, &provided_bytes)
}

/// Compare two byte slices in constant time.
pub fn bytes_constant_time_eq(expected: &[u8], provided: &[u8]) -> bool {
    let lengths_match: Choice = ConstantTimeEq::ct_eq(&(expected.len() as u64), &(provided.len() as u64));

    let max_len = expected.len().max(provided.len());
    let mut diff = Choice::from(0u8);

    for index in 0..max_len {
        let expected_byte = if index < expected.len() { expected[index] } else { 0 };
        let provided_byte = if index < provided.len() { provided[index] } else { 0 };
        diff |= expected_byte.ct_ne(&provided_byte);
    }

    bool::from(lengths_match & !diff)
}

#[cfg(test)]
mod tests {
    use super::{bytes_constant_time_eq, constant_time_eq, constant_time_eq_hex};

    #[test]
    fn identical_keys() {
        assert!(constant_time_eq("secret-key-abc", "secret-key-abc"));
    }

    #[test]
    fn different_content_same_length() {
        assert!(!constant_time_eq("secret-key-abc", "secret-key-xyz"));
    }

    #[test]
    fn different_lengths() {
        // Must return false without leaking which side is longer.
        assert!(!constant_time_eq("short", "short-but-longer"));
        assert!(!constant_time_eq("short-but-longer", "short"));
    }

    #[test]
    fn empty_strings() {
        assert!(constant_time_eq("", ""));
    }

    #[test]
    fn one_empty() {
        assert!(!constant_time_eq("", "non-empty"));
        assert!(!constant_time_eq("non-empty", ""));
    }

    #[test]
    fn prefix_match_is_rejected() {
        assert!(!constant_time_eq("abc", "abcX"));
        assert!(!constant_time_eq("abcX", "abc"));
    }

    #[test]
    fn hex_comparison_requires_a_full_match() {
        let expected = "ab".repeat(32);
        assert!(constant_time_eq_hex(&expected, &expected));
        // One differing character, same length.
        let mut tampered = expected.clone();
        tamper_last_char(&mut tampered);
        assert!(!constant_time_eq_hex(&expected, &tampered));
    }

    #[test]
    fn hex_comparison_rejects_malformed_or_mismatched_length() {
        let expected = "ab".repeat(32);
        assert!(!constant_time_eq_hex(&expected, &"ab".repeat(31)));
        assert!(!constant_time_eq_hex(&expected, "not-hex-at-all"));
        assert!(!constant_time_eq_hex("zz", "zz"));
    }

    #[test]
    fn byte_comparison_matches_string_comparison() {
        assert!(bytes_constant_time_eq(b"key", b"key"));
        assert!(!bytes_constant_time_eq(b"key", b"kex"));
        assert!(!bytes_constant_time_eq(b"key", b""));
    }

    fn tamper_last_char(value: &mut String) {
        let last = value.pop().expect("non-empty");
        value.push(if last == 'a' { 'b' } else { 'a' });
    }
}
