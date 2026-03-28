use argon2::Argon2;
use secrecy::{ExposeSecret, Secret};
use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;
use crate::{auth::hash::verify_string_with_hash, error::query::user::UserPasswordVerificationError};

/// Verify a user password with the one in the database.
#[instrument(name = "Verifing users password code", skip(db_conn))]
pub async fn verify_user_password<'a, A: Acquire<'a, Database = Postgres>>(
    db_conn: A,
    argon2_instance: &Argon2<'a>,
    user_id: &Uuid,
    password: &Secret<String>
) -> Result<bool, UserPasswordVerificationError> {
    let mut db_conn = db_conn.acquire().await?;

    let sql = "SELECT password_hash FROM users WHERE id = $1;";
    let row: (String,) = sqlx::query_as(sql)
        .bind(user_id)
        .fetch_one(&mut *db_conn)
        .await
        .map_err(|_| UserPasswordVerificationError::NotExists)?;
    let hash = row.0;

    Ok(verify_string_with_hash(password.expose_secret(), &hash, argon2_instance))
}
