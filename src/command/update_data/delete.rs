use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;
use crate::error::command::update_data::UpdateDataAddError;

/// Command to generate a new registration code and save it in the database, returning its and itself.
#[instrument(name = "Creating a registration code", skip(db_conn))]
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
