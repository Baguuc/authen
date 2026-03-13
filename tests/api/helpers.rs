use actix_web::body;
use authen::{configuration::{DatabaseSettings, Settings}, startup::Application, telemetry::{get_tracing_subscriber, init_tracing_subscriber}};
use reqwest::{Client, Response};
use secrecy::Secret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::{collections::HashMap, sync::LazyLock};
use uuid::Uuid;

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_tracing_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_tracing_subscriber(subscriber);
    } else {
        let subscriber = get_tracing_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_tracing_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: PgPool,
    pub api_client: reqwest::Client
}

impl TestApp {
    /// Send the request to POST /api/users route for testing purposes
    pub async fn post_users(http_client: &Client, address: &String, email: Option<String>, password: Option<String>) -> Result<Response, reqwest::Error> {
        let mut body_map = HashMap::new();
        if let Some(email) = email {
            body_map.insert("email", email);
        };
        if let Some(password) = password {
            body_map.insert("password", password);
        };
        
        http_client
            // Use the returned application address
            .post(&format!("{}/api/users", address))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(serde_urlencoded::to_string(&body_map).unwrap())
            .send()
            .await
    }

    /// Send the request to POST /api/confirmations/registration/{} route for testing purposes
    pub async fn post_registrations_confirmation(
        http_client: &Client,
        address: &String,
        confirmation_id: String,
        confirmation_code: Option<String>
    ) -> Result<Response, reqwest::Error> {
        let mut body_map = HashMap::new();

        if let Some(confirmation_code) = confirmation_code {
            body_map.insert("code", confirmation_code);
        };
        
        http_client
            // Use the returned application address
            .post(&format!("{}/api/confirmations/registration/{}", address, confirmation_id.to_string()))
            .header("content-type", "application/json")
            .json(&body_map)
            .send()
            .await
    }
}

pub async fn spawn_app(override_email_server_url: Option<String>) -> TestApp {
    LazyLock::force(&TRACING);

    // Randomise configuration to ensure test isolation
    let configuration = {
        let mut c = Settings::parse().expect("Failed to read configuration.");
        // Use a different database for each test case
        c.database.database_name = Uuid::new_v4().to_string();
        // Use a random OS port
        c.application.port = 0;

        // override the email server config to use wiremock's
        if let Some(url) = override_email_server_url {
            c.email.server.base_url = url;
        }

        c
    };

    // Create and migrate the database
    configure_database(&configuration.database).await;

    // Launch the application as a background task
    let application = Application::configure(configuration.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();
    let database_connection = Application::database_connection(configuration.database);
    let _ = tokio::spawn(application.run());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    let test_app = TestApp {
        address: format!("http://localhost:{}", application_port),
        port: application_port,
        db_pool: database_connection,
        api_client: client
    };

    test_app
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let maintenance_settings = DatabaseSettings {
        database_name: String::from("postgres"),
        username: String::from("postgres"),
        password: Secret::new(String::from("123")),
        ..config.clone()
    };
    let mut connection = PgConnection::connect_with(&maintenance_settings.connect_options())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
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
