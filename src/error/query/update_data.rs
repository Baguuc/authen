/// Enum modelling every error that can happen during retrieving confirmation's update data.
#[derive(thiserror::Error, Debug)]
pub enum GetUpdateDataError {
    #[error("Not exists.")]
    NotExists,
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error)
}