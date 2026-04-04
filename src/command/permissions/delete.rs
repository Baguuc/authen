use sqlx::{Acquire, Postgres};
use tracing::instrument;
use crate::error::command::permission::PermissionDeleteError;

/// Command to delete a permission from the database.
#[instrument(name = "Inserting a permission into the database.", skip(db_conn))]
pub async fn delete_permission<'a, A: Acquire<'a, Database = Postgres>>(
    db_conn: A,
    name: &String
) -> Result<(), PermissionDeleteError> {
    let mut db_conn = db_conn.acquire().await?;

    let sql = "DELETE FROM permissions WHERE name = $1;";
    let result = sqlx::query(sql)
        .bind(name)
        .execute(&mut *db_conn)
        .await;
    
    match result {
        Ok(result) => match result.rows_affected() {
            0 => Err(PermissionDeleteError::NotExists),
            _ => Ok(())
        },
        Err(err) => Err(PermissionDeleteError::Sqlx(err))
    }
}