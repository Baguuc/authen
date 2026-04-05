use argon2::Argon2;
use secrecy::{ExposeSecret, SecretString};
use sqlx::{Acquire, Postgres, error::ErrorKind};
use tracing::instrument;
use uuid::Uuid;
use crate::{auth::hash::hash_string, error::command::user::UserCreationError};

/// Command to hash the user's password and save the user in the database, returning its id.
#[instrument(name = "Inserting the user into database", skip(db_conn))]
pub async fn create_user<'a, A: Acquire<'a, Database = Postgres>>(
    db_conn: A,
    argon2_instance: &Argon2<'a>,
    email: &String,
    password: &SecretString
) -> Result<Uuid, UserCreationError> {
    let mut db_conn = db_conn.acquire().await?;

    let id = Uuid::new_v4();
    // can unwrap because the argon errors are generally environment based rather than input based.
    let hashed_password = hash_string(&password.expose_secret().to_string(), &argon2_instance)?;
    
    // the rest should be filled out by postgres automatically
    let sql = "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3) RETURNING id;";
    let result = sqlx::query_as(sql)
        .bind(id)
        .bind(email)
        .bind(hashed_password)
        .fetch_one(&mut *db_conn)
        .await;

    match result {
        Ok((id,)) => Ok(id),
        Err(err) => {
            match err.as_database_error() {
                Some(derr) => match derr.kind() {
                    ErrorKind::UniqueViolation => Err(UserCreationError::UserExists),
                    _ => Err(UserCreationError::Sqlx(err))
                },
                None => Err(UserCreationError::Sqlx(err))
            }
        }
    }
}