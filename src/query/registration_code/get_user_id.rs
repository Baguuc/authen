use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum GetUserIdFromRegistrationIdError {
    #[error("Not exists.")]
    NotExists,
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error)
}

/// Query to verify a registration code with the one in the database the database
#[instrument(name = "Verifing a registration code", skip(db_conn))]
pub async fn get_user_id_from_registration_id<'a, A: Acquire<'a, Database = Postgres>>(db_conn: A, registration_id: Uuid) -> Result<Uuid, GetUserIdFromRegistrationIdError> {
    let mut db_conn = db_conn.acquire().await?;

    // the rest should be filled out by postgres automatically
    let sql = "SELECT user_id FROM registration_codes WHERE id = $1;";
    let row: (Uuid,) = sqlx::query_as(sql)
        .bind(registration_id)
        .fetch_one(&mut *db_conn)
        .await
        .map_err(|_| GetUserIdFromRegistrationIdError::NotExists)?;

    Ok(row.0)
}
