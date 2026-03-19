use argon2::Argon2;
use authen::{crypto::hash, settings::{Settings, database::DatabaseSettings}, startup::Application, telemetry::{get_tracing_subscriber, init_tracing_subscriber}};
use reqwest::{Client, Response};
use secrecy::Secret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use wiremock::{Mock, MockServer, Request, ResponseTemplate, matchers::{method, path}};
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
            .header("content-type", "application/json")
            .json(&body_map)
            .send()
            .await
    }

    /// Send the request to POST /api/confirmations/registration/{} route for testing purposes
    pub async fn post_confirmations_registration(
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

    /// Send the request to DELETE /api/confirmations/registration/{} route for testing purposes
    pub async fn delete_confirmations_registration(
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
            .delete(&format!("{}/api/confirmations/registration/{}", address, confirmation_id.to_string()))
            .header("content-type", "application/json")
            .json(&body_map)
            .send()
            .await
    }

    /// Send the request to POST /api/session route for testing purposes
    pub async fn post_session(http_client: &Client, address: &String, email: Option<String>, password: Option<String>) -> Result<Response, reqwest::Error> {
        let mut body_map = HashMap::new();
        if let Some(email) = email {
            body_map.insert("email", email);
        };
        if let Some(password) = password {
            body_map.insert("password", password);
        };
        
        http_client
            // Use the returned application address
            .post(&format!("{}/api/session", address))
            .header("content-type", "application/json")
            .json(&body_map)
            .send()
            .await
    }

    /// Send the request to POST /api/confirmations/login/{} route for testing purposes
    pub async fn post_confirmations_login(
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
            .post(&format!("{}/api/confirmations/login/{}", address, confirmation_id.to_string()))
            .header("content-type", "application/json")
            .json(&body_map)
            .send()
            .await
    }
}

/// Create a already activated user for testing purposes
pub async fn create_active_user<'a>(db_conn: &PgPool, argon2_instance: &Argon2<'a>, email: &String, password: &String) {
    let _ = sqlx::query("INSERT INTO users (id, email, password_hash, active) VALUES ($1, $2, $3, true);")
        .bind(Uuid::new_v4())
        .bind(&email)
        .bind(&hash(&password, argon2_instance).unwrap())
        .execute(db_conn)
        .await;
}

pub async fn spawn_app(override_email_server_url: Option<String>) -> TestApp {
    LazyLock::force(&TRACING);

    // Randomise configuration to ensure test isolation
    let mut configuration = {
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
    configure_database(&mut configuration).await;

    // Launch the application as a background task
    let application = Application::configure(configuration.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();
    let database_connection = Application::database_connection(configuration);
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

pub async fn init() -> (TestApp, MockServer, reqwest::Client, Settings, Mock) {
    let mock_server = MockServer::start().await;
    let app = spawn_app(Some(mock_server.uri())).await;
    let http_client = reqwest::Client::new();
    let config = Settings::parse().unwrap();
    
    let mock = get_mock_email_api(&config, 1).await;

    (app, mock_server, http_client, config, mock)
}

async fn get_mock_email_api(config: &Settings, expected_email_count: u64) -> Mock {
    Mock::given(method(config.email.server.send_endpoint.method.clone()))
        .and(path(config.email.server.send_endpoint.route.clone()))
        .respond_with(ResponseTemplate::new(200))
        .expect(expected_email_count)
}

pub async fn get_request_from_mock_server(mock_server: &MockServer, request_index: usize) -> Request {
    mock_server.received_requests()
        .await
        .expect("Mock email server haven't got the request.")
        .get(request_index)
        .unwrap()
        .clone()
}

pub async fn get_registration_confirmation_code_from_request(mock_server: &MockServer, request_index: usize) -> String {
    let recieved_request = get_request_from_mock_server(mock_server, request_index).await;
    let recieved_request_body: HashMap<String, String> = recieved_request.body_json()
        .unwrap();
    let text_body: &String = recieved_request_body.get("TextBody")
        .expect("No TextBody in the request.");
    let confirmation_code = text_body.replace("Confirm your account using the code ", "");

    confirmation_code
}

pub async fn get_login_confirmation_code_from_request(mock_server: &MockServer, request_index: usize) -> String {
    let recieved_request = get_request_from_mock_server(mock_server, request_index).await;
    let recieved_request_body: HashMap<String, String> = recieved_request.body_json()
        .unwrap();
    let text_body: &String = recieved_request_body.get("TextBody")
        .expect("No TextBody in the request.");
    let confirmation_code = text_body.replace("Confirm your account using the code ", "");

    confirmation_code
}

async fn configure_database(config: &mut Settings) -> PgPool {
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
