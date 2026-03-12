use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;

use crate::crypto::hash;

/// Command to generate a new registration code and save it in the database, return the id of confirmation and the code.
#[instrument(name = "Creating a registration code", skip(db_conn))]
pub async fn create_registration_code<'a, A: Acquire<'a, Database = Postgres>>(db_conn: A, user_id: Uuid) -> Result<(Uuid, String), sqlx::Error> {
    let mut db_conn = db_conn.acquire().await?;

    tracing::info!("Generating a registration code.");
    
    let id = Uuid::new_v4();
    let code = Uuid::new_v4().to_string();
    // can unwrap because the argon errors are generally environment based rather than input based.
    let hashed = hash(&code).unwrap();

    tracing::info!("Saving the registration code.");
    
    // the rest should be filled out by postgres automatically
    let sql = "INSERT INTO registration_codes (id, code, user_id) VALUES ($1, $2, $3) RETURNING id, code;";
    let result: (Uuid, String) = sqlx::query_as(sql)
        .bind(id)
        .bind(hashed)
        .bind(user_id)
        .fetch_one(&mut *db_conn)
        .await?;
    
    tracing::info!("Saved the registration code.");

    Ok(result)
}