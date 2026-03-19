use argon2::Argon2;
use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;
use crate::{crypto::verify, error::query::confirmation_code::ConfirmationCodeVerificationError, model::{confirmation_code::ConfirmationCode, confirmation_code_type::ConfirmationCodeType}};

/// Verify a registration code with the one in the database.
#[instrument(name = "Verifing a registration code", skip(db_conn, code))]
pub async fn verify_confirmation_code<'a, A: Acquire<'a, Database = Postgres>>(
    db_conn: A,
    argon2_instance: Argon2<'a>,
    id: Uuid,
    code: ConfirmationCode,
    _type: ConfirmationCodeType
) -> Result<bool, ConfirmationCodeVerificationError> {
    let mut db_conn = db_conn.acquire().await?;

    // the rest should be filled out by postgres automatically
    let sql = "SELECT code FROM confirmation_codes WHERE id = $1 AND _type = $2;";
    let row: (String,) = sqlx::query_as(sql)
        .bind(id)
        .bind(_type.as_ref())
        .fetch_one(&mut *db_conn)
        .await
        .map_err(|_| ConfirmationCodeVerificationError::NotExists)?;
    let hash = row.0;

    Ok(verify(&code.as_ref().to_string(), &hash, &argon2_instance))
}
