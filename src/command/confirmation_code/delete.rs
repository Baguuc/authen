use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;

use crate::{error::command::confirmation_code::ConfirmationCodeDeletionError, model::confirmation_code_type::ConfirmationCodeType};

/// Command to delete a registration code from the database.
#[instrument(name = "Deleting a registration code", skip(db_conn))]
pub async fn delete_confirmation_code<'a, A: Acquire<'a, Database = Postgres>>(db_conn: A, id: Uuid, _type: ConfirmationCodeType) -> Result<(), ConfirmationCodeDeletionError> {
    let mut db_conn = db_conn.acquire().await?;
    
    // the rest should be filled out by postgres automatically
    let sql = "DELETE FROM confirmation_codes WHERE id = $1 AND _type = $2;";
    let result = sqlx::query(sql)
        .bind(id)
        .bind(_type.as_ref())
        .execute(&mut *db_conn)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ConfirmationCodeDeletionError::NotExists);
    }

    Ok(())
}
