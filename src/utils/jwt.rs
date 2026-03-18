use uuid::Uuid;

use crate::model::user_token_claims::UserTokenClaims;

/// Generate a jwt token based on the user_id and key
pub fn generate_user_token(user_id: Uuid, key: &String) -> Result<String, jsonwebtoken::errors::Error> {
    use jsonwebtoken::{EncodingKey, Header, encode};

    let claims = UserTokenClaims {
        sub: user_id,
        exp: (chrono::Utc::now() + chrono::Duration::days(7)).timestamp() as usize,
    };

    let encoded = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(key.as_ref()),
    )?;

    Ok(encoded)
}

/// Deserialize claims from a JWT token.
pub fn deserialize_claims_from_user_token(
    token: &str,
    key: &String,
) -> Result<UserTokenClaims, jsonwebtoken::errors::Error> {
    use jsonwebtoken::{DecodingKey, Validation, decode};

    let decoded = decode::<UserTokenClaims>(
        token,
        &DecodingKey::from_secret(key.as_ref()),
        &Validation::default(),
    )?;

    Ok(decoded.claims)
}