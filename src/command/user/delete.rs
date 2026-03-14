use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;
use crate::error::command::user::UserDeletionError;

/// Command to delete a user from the database.
#[instrument(name = "Deleting a user from the database", skip(db_conn))]
pub async fn delete_user<'a, A: Acquire<'a, Database = Postgres>>(db_conn: A, id: Uuid) -> Result<(), UserDeletionError> {
    let mut db_conn = db_conn.acquire().await?;

    // the rest should be filled out by postgres automatically
    let sql = "DELETE FROM users WHERE id = $1;";
    let result = sqlx::query(sql)
        .bind(id)
        .execute(&mut *db_conn)
        .await?;

    if result.rows_affected() == 0 {
        return Err(UserDeletionError::NotExists);
    }

    Ok(())
}