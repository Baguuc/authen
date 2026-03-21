/// Enum modelling every error that can happen during verification of user password with the one saved in the database.
#[derive(thiserror::Error, Debug)]
pub enum UserPasswordVerificationError {
    #[error("Not exists.")]
    NotExists,
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error)
}

/// Enum modelling every error that can happen during retrieving users id from email.
#[derive(thiserror::Error, Debug)]
pub enum GetUserIdError {
    #[error("Not exists.")]
    NotExists,
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error)
}

/// Enum modelling every error that can happen during checking if user is active.
#[derive(thiserror::Error, Debug)]
pub enum UserCheckIsActiveError {
    #[error("Not exists.")]
    NotExists,
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error)
}