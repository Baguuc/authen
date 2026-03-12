use rand::{TryRngCore, rngs::OsRng};

/// Generates a confirmation token from OsRng.
/// Returns None if there was an error generating the sequence.
pub fn generate_confirmation_token(length: usize, char_pool: &[u8]) -> Option<String> {
    let mut rng = OsRng;
    let mut buf = vec![0u8; length];
    rng.try_fill_bytes(&mut buf); // fill buffer with random bytes

    // Map each byte to a character in the pool
    let code = buf.iter()
        .map(|b| char_pool[(*b as usize) % char_pool.len()] as char)
        .collect();

    Some(code)
}