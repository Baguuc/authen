/// Enum modelling every error that can happen during creation of user.
#[derive(thiserror::Error, Debug)]
pub enum UserCreationError {
    #[error("Argon2 error: {0}")]
    Argon2(#[from] argon2::password_hash::Error),
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error)
}


/// Enum modelling every error that can happen during deleteion of user.
#[derive(thiserror::Error, Debug)]
pub enum UserDeletionError {
    #[error("User not exists in the database.")]
    NotExists,
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error)
}