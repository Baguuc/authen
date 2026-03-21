use argon2::Argon2;
use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;
use crate::{auth::{hash::hash_string, otp::generate_confirmation_code}, error::command::confirmation_code::ConfirmationCodeCreationError, model::confirmation_code_type::ConfirmationCodeType};

/// Command to generate a new registration code and save it in the database, returning its and itself.
#[instrument(name = "Creating a registration code", skip(db_conn))]
pub async fn create_confirmation_code<'a, A: Acquire<'a, Database = Postgres>>(
    db_conn: A,
    argon2_instance: &Argon2<'a>,
    user_id: Uuid,
    _type: ConfirmationCodeType
) -> Result<(Uuid, String), ConfirmationCodeCreationError> {
    let mut db_conn = db_conn.acquire().await?;

    let id = Uuid::new_v4();
    let code = generate_confirmation_code()
        .as_ref()
        .to_string();
    // can unwrap because the argon errors are generally environment based rather than input based.
    let hashed = hash_string(&code, argon2_instance).unwrap();
    
    // the rest should be filled out by postgres automatically
    let sql = "INSERT INTO confirmation_codes (id, code, user_id, _type) VALUES ($1, $2, $3, $4) RETURNING id, code;";
    let result: (Uuid, String) = sqlx::query_as(sql)
        .bind(id)
        .bind(hashed)
        .bind(user_id)
        .bind(_type.as_ref())
        .fetch_one(&mut *db_conn)
        .await?;

    Ok((result.0, code))
}
