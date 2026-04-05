use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::{Encoding, Error, SaltString, rand_core::OsRng}};
use secrecy::{ExposeSecret, SecretString};

/// Utility to hash anything using Argon2.
/// Made with the specific purpose to have standard function across the codebase, so in case
/// of parameters change everything stays in sync.
#[tracing::instrument(name = "Hashing a string.")]
pub fn hash_string(s: &SecretString, argon2_instance: &Argon2) -> Result<String, Error> {
    let s = s.expose_secret().as_bytes();
    let salt = SaltString::generate(&mut OsRng);

    let hash = argon2_instance.hash_password(s, &salt)?.to_string();

    Ok(hash)
}

/// Utility to validate a Argon2 hash.
/// Made with the specific purpose to have standard function across the codebase, so in case
/// of parameters change everything stays in sync.
#[tracing::instrument(name = "Testing a string against a password hash.")]
pub fn verify_string_with_hash(s: &String, hash: &str, argon2_instance: &Argon2) -> bool {
    let password_hash = match PasswordHash::parse(hash, Encoding::B64) {
        Ok(hash) => hash,
        Err(_) => return false,
    };

    let _ = match argon2_instance.verify_password(s.as_bytes(), &password_hash) {
        Ok(_) => return true,
        Err(_) => return false,
    };

}