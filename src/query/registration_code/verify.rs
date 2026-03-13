use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;
use crate::{crypto::verify, error::query::RegistrationCodeVerifyError, model::confirmation_code::ConfirmationCode};

/// Verify a registration code with the one in the database.
#[instrument(name = "Verifing a registration code", skip(db_conn, code))]
pub async fn verify_registration_code<'a, A: Acquire<'a, Database = Postgres>>(db_conn: A, id: Uuid, code: ConfirmationCode) -> Result<bool, RegistrationCodeVerifyError> {
    let mut db_conn = db_conn.acquire().await?;

    // the rest should be filled out by postgres automatically
    let sql = "SELECT code FROM registration_codes WHERE id = $1;";
    let row: (String,) = sqlx::query_as(sql)
        .bind(id)
        .fetch_one(&mut *db_conn)
        .await
        .map_err(|_| RegistrationCodeVerifyError::NotExists)?;
    let hash = row.0;

    Ok(verify(&code.as_ref().to_string(), &hash))
}
