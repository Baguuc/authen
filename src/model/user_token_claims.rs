use uuid::Uuid;

/// The model representing the claims of JWT tokens.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UserTokenClaims {
    pub sub: Uuid,
    pub exp: usize,
}