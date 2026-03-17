use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;

use crate::{error::query::GetUserIdFromConfirmationCodeRecordError, model::confirmation_code_type::ConfirmationCodeType};

/// Get user id from registration confirmation record in the database.
#[instrument(name = "Retrieving user id from confirmation code", skip(db_conn))]
pub async fn get_user_id_from_registration_code<'a, A: Acquire<'a, Database = Postgres>>(db_conn: A, registration_id: Uuid, _type: ConfirmationCodeType) -> Result<Uuid, GetUserIdFromConfirmationCodeRecordError> {
    let mut db_conn = db_conn.acquire().await?;

    let sql = "SELECT user_id FROM confirmation_codes WHERE id = $1 AND _type = $2;";
    let row: (Uuid,) = sqlx::query_as(sql)
        .bind(registration_id)
        .bind(_type.as_ref())
        .fetch_one(&mut *db_conn)
        .await?;

    Ok(row.0)
}
