use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;
use crate::{crypto::hash, error::command::user::UserCreationError};

/// Command to hash the user's password and save the user in the database, returning its id.
#[instrument(name = "Inserting the user into database", skip(db_conn, password))]
pub async fn create_user<'a, A: Acquire<'a, Database = Postgres>>(db_conn: A, email: &String, password: &String) -> Result<Uuid, UserCreationError> {
    let mut db_conn = db_conn.acquire().await?;

    let id = Uuid::new_v4();
    // can unwrap because the argon errors are generally environment based rather than input based.
    let hashed_password = hash(password)?;
    
    // the rest should be filled out by postgres automatically
    let sql = "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3) RETURNING id;";
    let result: (Uuid,) = sqlx::query_as(sql)
        .bind(id)
        .bind(email)
        .bind(hashed_password)
        .fetch_one(&mut *db_conn)
        .await?;

    Ok(result.0)
}