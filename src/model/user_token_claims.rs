/// The model representing the claims of JWT tokens.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UserTokenClaims {
    pub sub: i32,
    pub exp: usize,
}