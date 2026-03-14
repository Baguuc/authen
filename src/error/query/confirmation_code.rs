/// Enum modelling every error that can happen during getting user id from registration confirmation record.
#[derive(thiserror::Error, Debug)]
pub enum GetUserIdFromConfirmationCodeRecordError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error)
}

/// Enum modelling every error that can happen during verification of code with the one saved in the database.
#[derive(thiserror::Error, Debug)]
pub enum ConfirmationCodeVerificationError {
    #[error("Not exists.")]
    NotExists,
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error)
}