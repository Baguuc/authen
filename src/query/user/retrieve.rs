use sqlx::{Acquire, Postgres};
use tracing::instrument;
use uuid::Uuid;
use crate::{error::query::user::RetrieveUserError, model::user::User};

/// Retrieve a user from the database.
#[instrument(name = "Retrieving a user from the database.", skip(db_conn))]
pub async fn retrieve_user<'a, A: Acquire<'a, Database = Postgres>>(db_conn: A, id: Uuid) -> Result<User, RetrieveUserError> {
    let mut db_conn = db_conn.acquire().await?;

    let sql = "SELECT id, email, password_hash FROM users WHERE id = $1;";
    let user: User = sqlx::query_as(sql)
        .bind(id)
        .fetch_one(&mut *db_conn)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => RetrieveUserError::NotExists,
            err => RetrieveUserError::Sqlx(err)
        })?;
    
    Ok(user)
}
