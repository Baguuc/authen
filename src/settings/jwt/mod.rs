use jsonwebtoken::Algorithm;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct PartializedJwtSettings {
    pub algorithm: Option<Algorithm>,
    // value provided in minutes
    pub expires_in: Option<i64>,
    pub hashing_key: String,
}

#[derive(Deserialize, Clone)]
pub struct JwtSettings {
    pub algorithm: Algorithm,
    // value provided in minutes
    pub expires_in: i64,
    pub hashing_key: String,
}