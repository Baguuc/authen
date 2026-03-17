use authen::configuration::Settings;
use fake::{Fake, faker::internet::en::{Password, SafeEmail}};
use sqlx::Row;
use uuid::Uuid;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::{method, path}};
use crate::helpers::{TestApp, create_active_user, spawn_app};

#[derive(serde::Deserialize)]
struct ResponseBody {
    confirmation_id: Uuid
}

#[tokio::test]
async fn post_session_returns_200_for_valid_data() {
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

    // lets assume that there is already a user in the database,
    // we are already testing the registration in its own tests
    create_active_user(&app.db_pool, &email, &password).await;

    // Act
    let status = {
        let response = TestApp::post_session(&http_client, &app.address, Some(email), Some(password))
            .await
            .expect("Couldn't send the request to the API.");
        response.status()
    };

    // Assert
    assert_eq!(status, 200);

}

#[tokio::test]
async fn post_session_persists_valid_data() {
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

    let email = SafeEmail().fake();
    let password: String = Password(8..16).fake();

    // lets assume that there is already a user in the database,
    // we are already testing the registration in its own tests
    create_active_user(&app.db_pool, &email, &password).await;

    // Act
    let response = TestApp::post_session(&http_client, &app.address, Some(email), Some(password))
        .await
        .expect("Couldn't send the request to the API.");
    let response_body: ResponseBody = response.json()
        .await
        .unwrap();

    let registration_codes_count: i64 = sqlx::query("SELECT COUNT(*) as row_count FROM confirmation_codes;")
        .fetch_one(&app.db_pool)
        .await
        .unwrap()
        .get("row_count");

    let login_code_id: Uuid = sqlx::query("SELECT id FROM confirmation_codes;")
        .fetch_one(&app.db_pool)
        .await
        .unwrap()
        .get("id");
    
    assert_eq!(registration_codes_count, 1);
    assert_eq!(response_body.confirmation_id, login_code_id);
}

#[tokio::test]
async fn post_session_sends_the_link() {
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

    // lets assume that there is already a user in the database,
    // we are already testing the registration in its own tests
    create_active_user(&app.db_pool, &email, &password).await;

    // Act
    let _ = TestApp::post_session(&http_client, &app.address, Some(email), Some(password))
        .await
        .expect("Couldn't send the request to the API.");

    // Assert
    // mock will verify that the email has been sent
}

#[tokio::test]
async fn post_session_returns_400_for_invalid_email() {
    // Arrange
    let mock_server = MockServer::start().await;
    let app = spawn_app(Some(mock_server.uri())).await;
    let http_client = reqwest::Client::new();

    // no mock because it shouldn't send any email anyways

    let email: String = String::from("invali-email.com");
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    // Act
    let status = {
        let response = TestApp::post_session(&http_client, &app.address, Some(email), Some(password))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn post_session_returns_400_for_wrong_password() {
    // Arrange
    let mock_server = MockServer::start().await;
    let app = spawn_app(Some(mock_server.uri())).await;
    let http_client = reqwest::Client::new();

    // no mock because it shouldn't send any email anyways

    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let original_password: String = Password(1..16).fake();
    let fake_password: String = Password(1..16).fake();

    create_active_user(&app.db_pool, &email, &original_password).await;

    // Act
    let status = {
        let response = TestApp::post_session(&http_client, &app.address, Some(email), Some(fake_password))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status, 401);
}


#[tokio::test]
async fn post_session_returns_400_for_missing_email() {
    let mock_server = MockServer::start().await;
    let app = spawn_app(Some(mock_server.uri())).await;
    let http_client = reqwest::Client::new();

    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    // Act
    let status = {
        let response = TestApp::post_session(&http_client, &app.address, None, Some(password))
            .await
            .expect("Couldn't send the request to the API.");
    
        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn post_session_returns_400_for_missing_password() {
    let mock_server = MockServer::start().await;
    let app = spawn_app(Some(mock_server.uri())).await;
    let http_client = reqwest::Client::new();

    let email: String = SafeEmail().fake();

    // Act
    let status = {let response = TestApp::post_session(&http_client, &app.address, Some(email), None)
        .await
        .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn post_session_returns_400_for_both_fields_missing() {
    let mock_server = MockServer::start().await;
    let app = spawn_app(Some(mock_server.uri())).await;
    let http_client = reqwest::Client::new();

    // Act
    let status = {
         let response = TestApp::post_session(&http_client, &app.address, None, None)
        .await
        .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn post_session_returns_409_when_user_do_not_exists() {
    let mock_server = MockServer::start().await;
    let app = spawn_app(Some(mock_server.uri())).await;
    let http_client = reqwest::Client::new();

    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    let status = {
        // user does not yet exists in the database at this point
        let response = TestApp::post_session(&http_client, &app.address, Some(email.clone()), Some(password.clone()))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status, 401);
}

#[tokio::test]
async fn post_session_returns_500_on_database_fail() {
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
    let status = {
        let response = TestApp::post_session(&http_client, &app.address, Some(email), Some(password))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status, 500);
}