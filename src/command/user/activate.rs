use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;

/// Command to set the user's 'active' column to true (activating the user).
#[instrument(name = "Activating a user", skip(db_conn))]
pub async fn activate_user<'a, A: Acquire<'a, Database = Postgres>>(db_conn: A, id: Uuid) -> Result<(), sqlx::Error> {
    let mut db_conn = db_conn.acquire().await?;

    // the rest should be filled out by postgres automatically
    let sql = "UPDATE users SET active = true WHERE id = $1;";
    let _ = sqlx::query(sql)
        .bind(id)
        .execute(&mut *db_conn)
        .await?;

    Ok(())
}