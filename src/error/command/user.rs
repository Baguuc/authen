/// Enum modelling every error that can happen during creation of user.
#[derive(thiserror::Error, Debug)]
pub enum UserCreationError {
    #[error("Argon2 error: {0}")]
    Argon2(#[from] argon2::password_hash::Error),
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error)
}
