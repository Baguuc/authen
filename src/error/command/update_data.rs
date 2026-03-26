/// Enum modelling every error that can happen during adding update data to a confirmation.
#[derive(thiserror::Error, Debug)]
pub enum UpdateDataAddError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error)
}