use rand::{TryRngCore, rngs::OsRng};

use crate::{consts::{REGISTRATION_CODE_CHAR_POOL, REGISTRATION_CODE_LENGTH}, model::confirmation_code::ConfirmationCode};

/// Generates a confirmation code from OsRng.
/// Returns None if there was an error generating the sequence.
pub fn generate_confirmation_code() -> ConfirmationCode {
    let mut rng = OsRng;
    let mut buf = vec![0u8; REGISTRATION_CODE_LENGTH];
    rng.try_fill_bytes(&mut buf); // fill buffer with random bytes

    // Map each byte to a character in the pool
    let code_raw: String = buf.iter()
        .map(|b| REGISTRATION_CODE_CHAR_POOL[(*b as usize) % REGISTRATION_CODE_CHAR_POOL.len()] as char)
        .collect();
    
    match ConfirmationCode::parse(code_raw) {
        Ok(code) => code,
        // retry
        Err(_) => generate_confirmation_code()
    }
}