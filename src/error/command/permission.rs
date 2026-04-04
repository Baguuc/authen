/// Enum modelling every error that can happen during inserting a permission to the database.
#[derive(thiserror::Error, Debug)]
pub enum PermissionCreateError {
    #[error("ALREADY_EXISTS")]
    AlreadyExists,
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error)
}

/// Enum modelling every error that can happen during deleting a permission from the database.
#[derive(thiserror::Error, Debug)]
pub enum PermissionDeleteError {
    #[error("NOT_EXISTS")]
    NotExists,
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error)
}

/// Enum modelling every error that can happen during syncing permissions with the config.
#[derive(thiserror::Error, Debug)]
pub enum PermissionSyncError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error)
}