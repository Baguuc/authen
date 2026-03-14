/// Enum modelling every error that can happen during generation and saving of a registration code.
#[derive(thiserror::Error, Debug)]
pub enum ConfirmationCodeCreationError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error)
}

/// Enum modelling every error that can happen during deletion of a registration code.
#[derive(thiserror::Error, Debug)]
pub enum ConfirmationCodeDeletionError {
    #[error("Code not exists.")]
    NotExists,
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error)
}
