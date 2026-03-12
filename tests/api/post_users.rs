use authen::configuration::Settings;
use fake::{Fake, faker::internet::en::{Password, SafeEmail}};
use sqlx::Row;
use uuid::Uuid;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::{method, path}};

use crate::helpers::{TestApp, spawn_app};

#[derive(serde::Deserialize)]
struct ResponseBody {
    confirmation_id: Uuid
}

#[tokio::test]
async fn post_users_returns_200_for_valid_data() {
    let mock_server = MockServer::start().await;
    let app = spawn_app(Some(mock_server.uri())).await;
    let http_client = reqwest::Client::new();
    let config = Settings::parse().unwrap();

    Mock::given(method(config.email.server.send_endpoint.method))
        .and(path(config.email.server.send_endpoint.route))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    // Act
    let response = TestApp::post_users(&http_client, &app.address, Some(email), Some(password))
        .await
        .expect("Couldn't send the request to POST /api/users");
    let status = response.status();

    println!("Status: {}", status);
    // Assert
    assert!(status == 200);

}

#[tokio::test]
async fn post_users_persists_valid_data() {
    let mock_server = MockServer::start().await;
    let app = spawn_app(Some(mock_server.uri())).await;
    let http_client = reqwest::Client::new();
    let config = Settings::parse().unwrap();

    Mock::given(method(config.email.server.send_endpoint.method))
        .and(path(config.email.server.send_endpoint.route))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    // Act
    let response = TestApp::post_users(&http_client, &app.address, Some(email), Some(password))
        .await
        .expect("Couldn't send the request to POST /api/users");
    let response_body: ResponseBody = response.json()
        .await
        .unwrap();

    let users_row_count: i64 = sqlx::query("SELECT COUNT(*) as row_count FROM users;")
        .fetch_one(&app.db_pool)
        .await
        .unwrap()
        .get("row_count");
    assert_eq!(users_row_count, 1);

    let registration_codes_count: i64 = sqlx::query("SELECT COUNT(*) as row_count FROM registration_codes;")
        .fetch_one(&app.db_pool)
        .await
        .unwrap()
        .get("row_count");
    assert_eq!(registration_codes_count, 1);

    let registration_code_id: Uuid = sqlx::query("SELECT id FROM registration_codes;")
        .fetch_one(&app.db_pool)
        .await
        .unwrap()
        .get("id");
    assert_eq!(response_body.confirmation_id, registration_code_id);
}

#[tokio::test]
async fn post_users_sends_the_link() {
    // Arrange
    let mock_server = MockServer::start().await;
    let app = spawn_app(Some(mock_server.uri())).await;
    let http_client = reqwest::Client::new();
    let config = Settings::parse().unwrap();

    Mock::given(method(config.email.server.send_endpoint.method))
        .and(path(config.email.server.send_endpoint.route))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    // Act
    let _ = TestApp::post_users(&http_client, &app.address, Some(email), Some(password))
        .await
        .expect("Couldn't send the request to POST /api/users");

    // Assert
    // mock will verify that the email has been sent
}

#[tokio::test]
async fn post_users_returns_400_for_invalid_email() {
    // Arrange
    let mock_server = MockServer::start().await;
    let app = spawn_app(Some(mock_server.uri())).await;
    let http_client = reqwest::Client::new();

    // no mock because it shouldn't send any email anyways

    let email: String = String::from("invali-email.com");
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    // Act
    let response = TestApp::post_users(&http_client, &app.address, Some(email), Some(password))
        .await
        .expect("Couldn't send the request to POST /api/users");
    let status = response.status();

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn post_users_returns_400_for_empty_email() {
    let mock_server = MockServer::start().await;
    let app = spawn_app(Some(mock_server.uri())).await;
    let http_client = reqwest::Client::new();

    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    // Act
    let response = TestApp::post_users(&http_client, &app.address, None, Some(password))
        .await
        .expect("Couldn't send the request to POST /api/users");
    let status = response.status();

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn post_users_returns_400_for_missing_password() {
    let mock_server = MockServer::start().await;
    let app = spawn_app(Some(mock_server.uri())).await;
    let http_client = reqwest::Client::new();

    let email: String = SafeEmail().fake();

    // Act
    let response = TestApp::post_users(&http_client, &app.address, Some(email), None)
        .await
        .expect("Couldn't send the request to POST /api/users");
    let status = response.status();

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn post_users_returns_400_for_both_fields_missing() {
    let mock_server = MockServer::start().await;
    let app = spawn_app(Some(mock_server.uri())).await;
    let http_client = reqwest::Client::new();

    // Act
    let response = TestApp::post_users(&http_client, &app.address, None, None)
        .await
        .expect("Couldn't send the request to POST /api/users");
    let status = response.status();

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn subscribe_fails_if_there_is_a_fatal_database_error() {
    // Arrange
    let mock_server = MockServer::start().await;
    let app = spawn_app(Some(mock_server.uri())).await;
    let http_client = reqwest::Client::new();

    // Sabotage the database
    let _ = sqlx::query("ALTER TABLE users DROP COLUMN email;",)
        .execute(&app.db_pool)
        .await;

    let email: String = SafeEmail().fake();
    let password: String = Password(1..16).fake();

    // Act
    let response = TestApp::post_users(&http_client, &app.address, Some(email), Some(password))
        .await
        .expect("Couldn't send the request to POST /api/users");
    let status = response.status();

    // Assert
    assert_eq!(status, 500);
}