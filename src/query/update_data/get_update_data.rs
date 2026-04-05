use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;
use crate::error::query::update_data::GetUpdateDataError;

/// Retrieve an update data row from database.
#[instrument(name = "Retrieving confirmation code's update data.", skip(db_conn))]
pub async fn get_update_data<'a, A: Acquire<'a, Database = Postgres>>(db_conn: A, confirmation_id: &Uuid) -> Result<serde_json::Value, GetUpdateDataError> {
    let mut db_conn = db_conn.acquire().await?;

    let sql = "SELECT data FROM updates_data WHERE confirmation_id = $1;";
    let (data,) = sqlx::query_as(sql)
        .bind(confirmation_id)
        .fetch_one(&mut *db_conn)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => GetUpdateDataError::NotExists,
            err => GetUpdateDataError::Sqlx(err)
        })?;

    Ok(data)
}
