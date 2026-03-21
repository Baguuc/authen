use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use uuid::Uuid;

use crate::model::user_token_claims::UserTokenClaims;

/// Generate a jwt token based on the user_id and key
pub fn generate_user_token(
    key: &String,
    jwt_header: &Header,
    expires_in: chrono::Duration,
    user_id: Uuid
) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = UserTokenClaims {
        sub: user_id,
        exp: (chrono::Utc::now() + expires_in).timestamp() as usize,
    };
    
    let encoded = encode(
        jwt_header,
        &claims,
        &EncodingKey::from_secret(key.as_ref()),
    )?;

    Ok(encoded)
}

/// Deserialize claims from a JWT token.
pub fn deserialize_claims_from_user_token(
    key: &String,
    jwt_validation: &Validation,
    token: &str
) -> Result<UserTokenClaims, jsonwebtoken::errors::Error> {
    let decoded = decode::<UserTokenClaims>(
        token,
        &DecodingKey::from_secret(key.as_ref()),
        jwt_validation
    )?;

    Ok(decoded.claims)
}