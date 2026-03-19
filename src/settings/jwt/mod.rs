use jsonwebtoken::Algorithm;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct JwtSettings {
    pub algorithm: Algorithm,
    pub hashing_key: String,
    // value provided in minutes
    pub expires_in: i64
}