pub mod commands;

use authen::settings::{Settings, database::DatabaseSettings};
use secrecy::Secret;
use sqlx::{Connection, Executor, PgConnection, PgPool};

pub async fn configure_database(config: &mut Settings) -> PgPool {
    // Create database
    let maintenance_settings = DatabaseSettings {
        database_name: String::from("postgres"),
        username: String::from("postgres"),
        password: Secret::new(String::from("123")),
        ..config.database.clone()
    };
    let mut connection = PgConnection::connect_with(&maintenance_settings.connect_options())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.connect_options())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}
