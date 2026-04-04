/// Enum modelling every error that can happen during inserting a permission to the database.
#[derive(thiserror::Error, Debug)]
pub enum PermissionCreateError {
    #[error("ALREADY_EXISTS")]
    AlreadyExists,
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error)
}