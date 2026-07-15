#![allow(dead_code)]

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};

pub async fn hash() {
    // placeholder
}

pub async fn verify_password() {
    // placeholder
}

/// Current Argon2 parameters for password hashing.
/// Update these when strengthening requirements.
const ARGON2_M_COST: u32 = 65536; // 64 MB
const ARGON2_T_COST: u32 = 3;
const ARGON2_P_COST: u32 = 1;

/// Hash a plaintext password using Argon2id with current parameters.
pub fn password_hash(password: String) -> String {
    let mut rng = rand::thread_rng();

    let salt = SaltString::generate(&mut rng);

    let params = argon2::Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, None)
        .expect("Invalid Argon2 parameters");
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Unable to hash password.")
        .to_string()
}

/// Verify a plaintext password against an Argon2id hash.
pub fn verify(password: String, hash: String) -> bool {
    let parsed_hash = PasswordHash::new(&hash).expect("Failed to parse hash");

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

/// Check if a password hash needs to be rehashed with current parameters.
/// Returns true if the hash was created with different parameters than the current policy.
pub fn needs_rehash(hash: &str) -> bool {
    let parsed = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return true, // Can't parse = needs rehash
    };

    let params = &parsed.params;

    let m_cost = params.get("m").and_then(|v| v.to_string().parse::<u32>().ok()).unwrap_or(0);
    let t_cost = params.get("t").and_then(|v| v.to_string().parse::<u32>().ok()).unwrap_or(0);
    let p_cost = params.get("p").and_then(|v| v.to_string().parse::<u32>().ok()).unwrap_or(0);

    m_cost != ARGON2_M_COST || t_cost != ARGON2_T_COST || p_cost != ARGON2_P_COST
}

/// Rehash a password with current parameters. Returns the new hash.
/// Should only be called after successful password verification.
pub fn rehash_password(password: &str) -> String {
    password_hash(password.to_string())
}
