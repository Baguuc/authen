use argon2::password_hash::rand_core::{OsRng, RngCore};

use crate::{consts::{CONFIRMATION_CODE_CHAR_POOL, CONFIRMATION_CODE_LENGTH}, model::confirmation_code::ConfirmationCode};

/// Generates a confirmation code from OsRng.
/// Returns None if there was an error generating the sequence.
#[tracing::instrument(name = "Generating a confirmation code string.")]
pub fn generate_confirmation_code() -> ConfirmationCode {
    let mut rng = OsRng;
    let mut buf = vec![0u8; CONFIRMATION_CODE_LENGTH];
    let _ = rng.try_fill_bytes(&mut buf); // fill buffer with random bytes

    // Map each byte to a character in the pool
    let code_raw: String = buf.iter()
        .map(|b| CONFIRMATION_CODE_CHAR_POOL[(*b as usize) % CONFIRMATION_CODE_CHAR_POOL.len()])
        .collect();
    
    match ConfirmationCode::parse(code_raw) {
        Ok(code) => code,
        // retry
        Err(_) => generate_confirmation_code()
    }
}