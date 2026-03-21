use argon2::Argon2;
use authen::auth::hash::hash_string;
use sqlx::PgPool;
use uuid::Uuid;

/// Create a already activated user for testing purposes
pub async fn create_active_user<'a>(db_conn: &PgPool, argon2_instance: &Argon2<'a>, email: &String, password: &String) {
    let _ = sqlx::query("INSERT INTO users (id, email, password_hash, active) VALUES ($1, $2, $3, true);")
        .bind(Uuid::new_v4())
        .bind(&email)
        .bind(&hash_string(&password, argon2_instance).unwrap())
        .execute(db_conn)
        .await;
}

/// Create inactivated user for testing purposes
pub async fn create_inactive_user<'a>(db_conn: &PgPool, argon2_instance: &Argon2<'a>, email: &String, password: &String) {
    let _ = sqlx::query("INSERT INTO users (id, email, password_hash, active) VALUES ($1, $2, $3, false);")
        .bind(Uuid::new_v4())
        .bind(&email)
        .bind(&hash_string(&password, argon2_instance).unwrap())
        .execute(db_conn)
        .await;
}