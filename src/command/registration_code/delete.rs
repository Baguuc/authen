use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;

use crate::error::command::RegistrationCodeDeletionError;

/// Command to delete a registration code from the database.
#[instrument(name = "Creating a registration code", skip(db_conn))]
pub async fn delete_registration_code<'a, A: Acquire<'a, Database = Postgres>>(db_conn: A, id: Uuid) -> Result<(), RegistrationCodeDeletionError> {
    let mut db_conn = db_conn.acquire().await?;
    
    // the rest should be filled out by postgres automatically
    let sql = "DELETE FROM registration_codes WHERE id = $1;";
    let result = sqlx::query(sql)
        .bind(id)
        .execute(&mut *db_conn)
        .await?;

    if result.rows_affected() == 0 {
        return Err(RegistrationCodeDeletionError::NotExists);
    }

    Ok(())
}
