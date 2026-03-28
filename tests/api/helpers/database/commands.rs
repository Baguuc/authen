use argon2::Argon2;
use authen::auth::hash::hash_string;
use sqlx::PgPool;
use uuid::Uuid;

/// Create a already activated user for testing purposes
pub async fn create_active_user<'a>(db_conn: &PgPool, argon2_instance: &Argon2<'a>, email: &String, password: &String) -> Uuid {
    let id = Uuid::new_v4();
    let _ = sqlx::query("INSERT INTO users (id, email, password_hash, active) VALUES ($1, $2, $3, true) RETURNING id;")
        .bind(&id)
        .bind(&email)
        .bind(&hash_string(&password, argon2_instance).unwrap())
        .execute(db_conn)
        .await;
    id
}

/// Create inactivated user for testing purposes
pub async fn create_inactive_user<'a>(db_conn: &PgPool, argon2_instance: &Argon2<'a>, email: &String, password: &String) -> Uuid {
    let id = Uuid::new_v4();
    let _ = sqlx::query("INSERT INTO users (id, email, password_hash, active) VALUES ($1, $2, $3, false) RETURNING id;")
        .bind(&id)
        .bind(&email)
        .bind(&hash_string(&password, argon2_instance).unwrap())
        .execute(db_conn)
        .await;
    id
}

/// Get user's password hash from the database for testing purposes
pub async fn get_user_password_hash<'a>(db_conn: &PgPool, id: &Uuid) -> String {
    let (password,) = sqlx::query_as("SELECT password_hash FROM users WHERE id = $1;")
        .bind(&id)
        .fetch_one(db_conn)
        .await
        .unwrap();
    
    password
}