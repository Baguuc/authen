use sqlx::{Acquire, Postgres, error::ErrorKind};
use tracing::instrument;
use crate::error::command::permission::PermissionCreateError;

/// Command to insert a permission into the database.
#[instrument(name = "Inserting a permission into the database.", skip(db_conn))]
pub async fn create_permission<'a, A: Acquire<'a, Database = Postgres>>(
    db_conn: A,
    name: &String
) -> Result<(), PermissionCreateError> {
    let mut db_conn = db_conn.acquire().await?;

    let sql = "INSERT INTO permissions (name) VALUES ($1);";
    let result = sqlx::query(sql)
        .bind(name)
        .execute(&mut *db_conn)
        .await;
    
    match result {
        Ok(_) => Ok(()),
        Err(err) => match err.as_database_error() {
            Some(derr) => match derr.kind() {
                ErrorKind::UniqueViolation => Err(PermissionCreateError::AlreadyExists),
                _ => Err(PermissionCreateError::Sqlx(err))
            },
            None => Err(PermissionCreateError::Sqlx(err))
        }
    }
}