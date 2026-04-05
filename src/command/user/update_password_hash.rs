use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;
use crate::{error::command::user::UserPasswordUpdateError, model::hashed_string::HashedString};

/// Command update user's password hash.
/// CAUTHION: The command itself doesn't hash the password, rather stores already created hash in place of the old one.
#[instrument(name = "Updating user's password hash.", skip(db_conn))]
pub async fn update_password_hash<'a, A: Acquire<'a, Database = Postgres>>(
    db_conn: A,
    user_id: &Uuid,
    password_hash: &HashedString
) -> Result<(), UserPasswordUpdateError> {
    let mut db_conn = db_conn.acquire().await?;
    
    // the rest should be filled out by postgres automatically
    let sql = "UPDATE users SET password_hash = $1 WHERE id = $2;";
    sqlx::query(sql)
        .bind(password_hash.as_ref())
        .bind(user_id)
        .execute(&mut *db_conn)
        .await?;

    Ok(())
}