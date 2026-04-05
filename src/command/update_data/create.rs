use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;
use crate::error::command::update_data::UpdateDataAddError;

/// Command to attach related update data to a confirmation code in the database.
/// Used for update routes.
#[instrument(name = "Attaching update data to confirmation code.", skip(db_conn))]
pub async fn add_update_data_to_confirmation_code<'a, A: Acquire<'a, Database = Postgres>>(
    db_conn: A,
    confirmation_id: &Uuid,
    data: &serde_json::Value
) -> Result<(), UpdateDataAddError> {
    let mut db_conn = db_conn.acquire().await?;
    
    // the rest should be filled out by postgres automatically
    let sql = "INSERT INTO updates_data (confirmation_id, data) VALUES ($1, $2);";
    let _ = sqlx::query(sql)
        .bind(confirmation_id)
        .bind(data)
        .execute(&mut *db_conn)
        .await?;

    Ok(())
}
