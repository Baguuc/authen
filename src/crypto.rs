use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::{Encoding, Error, SaltString, rand_core::OsRng}};

/// Utility to hash anything using Argon2.
/// Made with the specific purpose to have standard function across the codebase, so in case
/// of parameters change everything stays in sync.
pub fn hash(s: &String) -> Result<String, Error> {
    let s = s.as_bytes();
    let salt = SaltString::generate(&mut OsRng);

    let hash = Argon2::default().hash_password(s, &salt)?.to_string();

    Ok(hash)
}

/// Utility to validate a Argon2 hash.
/// Made with the specific purpose to have standard function across the codebase, so in case
/// of parameters change everything stays in sync.
pub fn verify(s: &String, hash: &str) -> bool {
    let password_hash = match PasswordHash::parse(hash, Encoding::B64) {
        Ok(hash) => hash,
        Err(_) => return false,
    };

    let _ = match Argon2::default().verify_password(s.as_bytes(), &password_hash) {
        Ok(_) => return true,
        Err(_) => return false,
    };

}