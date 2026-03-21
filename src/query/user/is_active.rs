use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;
use crate::error::query::user::UserCheckIsActiveError;

/// Check if user is active.
#[instrument(name = "Checking user active status", skip(db_conn))]
pub async fn is_user_active<'a, A: Acquire<'a, Database = Postgres>>(db_conn: A, user_id: &Uuid) -> Result<bool, UserCheckIsActiveError> {
    let mut db_conn = db_conn.acquire().await?;

    let sql = "SELECT active FROM users WHERE id = $1;";
    let row: (bool,) = sqlx::query_as(sql)
        .bind(user_id)
        .fetch_one(&mut *db_conn)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => UserCheckIsActiveError::NotExists,
            err => UserCheckIsActiveError::Sqlx(err)
        })?;
    let is_active = row.0;

    Ok(is_active)
}
