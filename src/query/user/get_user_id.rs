use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;
use crate::{error::query::user::GetUserIdError, model::email::Email};

/// Retrieve the user's id from its email.
#[instrument(name = "Retrieving user id from email.", skip(db_conn))]
pub async fn get_user_id_from_email<'a, A: Acquire<'a, Database = Postgres>>(db_conn: A, email: &Email) -> Result<Uuid, GetUserIdError> {
    let mut db_conn = db_conn.acquire().await?;

    let sql = "SELECT id FROM users WHERE email = $1;";
    let row: (Uuid,) = sqlx::query_as(sql)
        .bind(email.as_ref())
        .fetch_one(&mut *db_conn)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => GetUserIdError::NotExists,
            err => GetUserIdError::Sqlx(err)
        })?;
    let id = row.0;

    Ok(id)
}
