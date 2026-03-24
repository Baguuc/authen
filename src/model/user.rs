use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String
}