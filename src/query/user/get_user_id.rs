use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;
use crate::{error::query::{GetUserIdError, UserPasswordVerificationError}, model::email::Email};

/// Retrieve the users id from email.
#[instrument(name = "Verifing a registration code", skip(db_conn))]
pub async fn get_user_id<'a, A: Acquire<'a, Database = Postgres>>(db_conn: A, email: &Email) -> Result<Uuid, GetUserIdError> {
    let mut db_conn = db_conn.acquire().await?;

    let sql = "SELECT email FROM users WHERE id = $1;";
    let row: (Uuid,) = sqlx::query_as(sql)
        .bind(email.as_ref())
        .fetch_one(&mut *db_conn)
        .await
        .map_err(|_| GetUserIdError::NotExists)?;
    let id = row.0;

    Ok(id)
}
