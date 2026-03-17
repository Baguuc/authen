/// Enum modelling every error that can happen during verification of user password with the one saved in the database.
#[derive(thiserror::Error, Debug)]
pub enum UserPasswordVerificationError {
    #[error("Not exists.")]
    NotExists,
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error)
}