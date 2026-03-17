use sqlx::{Acquire, Postgres};
use tracing::instrument;
use crate::{crypto::verify, error::query::UserPasswordVerificationError, model::email::Email};

/// Verify a user password with the one in the database.
#[instrument(name = "Verifing a registration code", skip(db_conn))]
pub async fn verify_user_password<'a, A: Acquire<'a, Database = Postgres>>(db_conn: A, email: Email, password: String) -> Result<bool, UserPasswordVerificationError> {
    let mut db_conn = db_conn.acquire().await?;

    let sql = "SELECT password_hash FROM users WHERE email = $1;";
    let row: (String,) = sqlx::query_as(sql)
        .bind(email.as_ref())
        .fetch_one(&mut *db_conn)
        .await
        .map_err(|_| UserPasswordVerificationError::NotExists)?;
    let hash = row.0;

    Ok(verify(&password, &hash))
}
