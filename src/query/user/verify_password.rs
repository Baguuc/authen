use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;
use crate::{crypto::verify, error::query::UserPasswordVerificationError};

/// Verify a user password with the one in the database.
#[instrument(name = "Verifing users password code", skip(db_conn))]
pub async fn verify_user_password<'a, A: Acquire<'a, Database = Postgres>>(db_conn: A, user_id: &Uuid, password: &String) -> Result<bool, UserPasswordVerificationError> {
    let mut db_conn = db_conn.acquire().await?;

    let sql = "SELECT password_hash FROM users WHERE id = $1;";
    let row: (String,) = sqlx::query_as(sql)
        .bind(user_id)
        .fetch_one(&mut *db_conn)
        .await
        .map_err(|_| UserPasswordVerificationError::NotExists)?;
    let hash = row.0;

    Ok(verify(password, &hash))
}
