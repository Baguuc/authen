use std::{collections::HashMap, sync::LazyLock};

use authen::{settings::Settings, startup::Application};
use reqwest::{Client, Response};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::helpers::{TRACING, database::configure_database};

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

    /// Send the request to GET /api/session route for testing purposes
    pub async fn get_session(http_client: &Client, address: &String, authorization_method: String, token: Option<String>, fields: Vec<String>) -> Result<Response, reqwest::Error> {
        let mut builder = http_client
            // Use the returned application address
            .get(&format!("{}/api/session?fields={}", address, fields.join(",")))
            .header("content-type", "application/json");

        if let Some(token) = token {
            builder = builder.header("authorization", format!("{} {}", authorization_method, token))
        }

        builder
            .send()
            .await
    }

    /// Send the request to GET /api/session route for testing purposes
    pub async fn put_session_user_password(http_client: &Client, address: &String, authorization_method: String, token: Option<String>, password: Option<String>, new_password: Option<String>) -> Result<Response, reqwest::Error> {
        let mut builder = http_client
            // Use the returned application address
            .patch(&format!("{}/api/session/user", address))
            .header("content-type", "application/json");

        if let Some(token) = token {
            builder = builder.header("authorization", format!("{} {}", authorization_method, token))
        }

        let body = json!({
            "password": password,
            "new_password": new_password
        });

        builder
            .json(&body)
            .send()
            .await
    }
    

    /// Send the requestw to POST /api/confirmations/login/{} route for testing purposes
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
