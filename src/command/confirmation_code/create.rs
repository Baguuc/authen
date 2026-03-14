use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;
use crate::{crypto::hash, error::command::RegistractionCodeCreationError, utils::generation::generate_confirmation_code};

/// Command to generate a new registration code and save it in the database, returning its and itself.
#[instrument(name = "Creating a registration code", skip(db_conn))]
pub async fn create_confirmation_code<'a, A: Acquire<'a, Database = Postgres>>(db_conn: A, user_id: Uuid, _type: String) -> Result<(Uuid, String), RegistractionCodeCreationError> {
    let mut db_conn = db_conn.acquire().await?;

    let id = Uuid::new_v4();
    let code = generate_confirmation_code()
        .as_ref()
        .to_string();
    // can unwrap because the argon errors are generally environment based rather than input based.
    let hashed = hash(&code).unwrap();
    
    // the rest should be filled out by postgres automatically
    let sql = "INSERT INTO confirmation_codes (id, code, user_id, _type) VALUES ($1, $2, $3, $4) RETURNING id, code;";
    let result: (Uuid, String) = sqlx::query_as(sql)
        .bind(id)
        .bind(hashed)
        .bind(user_id)
        .bind(_type)
        .fetch_one(&mut *db_conn)
        .await?;

    Ok((result.0, code))
}
