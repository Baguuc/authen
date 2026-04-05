use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;
use crate::error::command::update_data::UpdateDataAddError;

/// Command remove a attached update data from a confirmation code in the database.
#[instrument(name = "Removing an update data from a confirmation code.", skip(db_conn))]
pub async fn delete_update_data<'a, A: Acquire<'a, Database = Postgres>>(
    db_conn: A,
    confirmation_id: &Uuid
) -> Result<(), UpdateDataAddError> {
    let mut db_conn = db_conn.acquire().await?;
    
    // the rest should be filled out by postgres automatically
    let sql = "DELETE FROM updates_data WHERE confirmation_id = $1;";
    let _ = sqlx::query(sql)
        .bind(confirmation_id)
        .execute(&mut *db_conn)
        .await?;

    Ok(())
}
