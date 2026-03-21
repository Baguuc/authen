use fake::{Fake, faker::internet::en::{Password, SafeEmail}};
use sqlx::Row;
use uuid::Uuid;
use crate::helpers::{app::TestApp, init};

#[derive(serde::Deserialize)]
struct ResponseBody {
    confirmation_id: Uuid
}

#[tokio::test]
async fn post_users_returns_200_for_valid_data() {
    let (app, mock_server, http_client, _, mock) = init().await;
    mock.mount(&mock_server).await;

    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    // Act
    let status = {
        let response = TestApp::post_users(&http_client, &app.address, Some(email), Some(password))
            .await
            .expect("Couldn't send the request to the API.");
        response.status()
    };

    // Assert
    assert_eq!(status, 200);

}

#[tokio::test]
async fn post_users_persists_valid_data() {
    let (app, mock_server, http_client, _, mock) = init().await;
    mock.mount(&mock_server).await;

    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    // Act
    let response = TestApp::post_users(&http_client, &app.address, Some(email), Some(password))
        .await
        .expect("Couldn't send the request to the API.");
    let response_body: ResponseBody = response.json()
        .await
        .unwrap();

    let users_row_count: i64 = sqlx::query("SELECT COUNT(*) as row_count FROM users;")
        .fetch_one(&app.db_pool)
        .await
        .unwrap()
        .get("row_count");

    let registration_codes_count: i64 = sqlx::query("SELECT COUNT(*) as row_count FROM confirmation_codes;")
        .fetch_one(&app.db_pool)
        .await
        .unwrap()
        .get("row_count");

    let registration_code_id: Uuid = sqlx::query("SELECT id FROM confirmation_codes;")
        .fetch_one(&app.db_pool)
        .await
        .unwrap()
        .get("id");

    assert_eq!(users_row_count, 1);
    assert_eq!(registration_codes_count, 1);
    assert_eq!(response_body.confirmation_id, registration_code_id);
}

#[tokio::test]
async fn post_users_sends_the_link() {
    let (app, mock_server, http_client, _, mock) = init().await;
    mock.mount(&mock_server).await;

    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    // Act
    let _ = TestApp::post_users(&http_client, &app.address, Some(email), Some(password))
        .await
        .expect("Couldn't send the request to the API.");

    // Assert
    // mock will verify that the email has been sent
}

#[tokio::test]
async fn post_users_returns_400_for_invalid_email() {
    // Arrange
    let (app, _, http_client, _, _mock) = init().await;

    // no mock because it shouldn't send any email anyways

    let email: String = String::from("invali-email.com");
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    // Act
    let status = {
        let response = TestApp::post_users(&http_client, &app.address, Some(email), Some(password))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn post_users_returns_400_for_empty_email() {
    let (app, _, http_client, _, _mock) = init().await;

    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    // Act
    let status = {
        let response = TestApp::post_users(&http_client, &app.address, None, Some(password))
            .await
            .expect("Couldn't send the request to the API.");
    
        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn post_users_returns_400_for_missing_password() {
    let (app, _, http_client, _, _mock) = init().await;

    let email: String = SafeEmail().fake();

    // Act
    let status = {let response = TestApp::post_users(&http_client, &app.address, Some(email), None)
        .await
        .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn post_users_returns_400_for_both_fields_missing() {
    let (app, _, http_client, _, _mock) = init().await;

    // Act
    let status = {
         let response = TestApp::post_users(&http_client, &app.address, None, None)
        .await
        .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn post_users_returns_409_and_error_message_when_user_already_registered() {
    let (app, _, http_client, _, _mock) = init().await;

    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    let status1 = {
        // Act
        let response = TestApp::post_users(&http_client, &app.address, Some(email.clone()), Some(password.clone()))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // send the same email
    // password doesn't matter obviously
    let status2 = {
        // Act
        let response = TestApp::post_users(&http_client, &app.address, Some(email), Some(password))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status1, 200);
    assert_eq!(status2, 409);
}

#[tokio::test]
async fn post_users_returns_500_on_database_fail() {
    // Arrange
    let (app, _, http_client, _, _mock) = init().await;

    // Sabotage the database
    let _ = sqlx::query("ALTER TABLE users DROP COLUMN email;",)
        .execute(&app.db_pool)
        .await;

    let email: String = SafeEmail().fake();
    let password: String = Password(1..16).fake();

    // Act
    let status = {
        let response = TestApp::post_users(&http_client, &app.address, Some(email), Some(password))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status, 500);
}