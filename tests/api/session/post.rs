use authen::settings::Settings;
use fake::{Fake, faker::internet::en::{Password, SafeEmail}};
use sqlx::Row;
use uuid::Uuid;
use crate::helpers::{TestApp, create_active_user, create_inactive_user, init};

#[derive(serde::Deserialize)]
struct ResponseBody {
    confirmation_id: Uuid
}

#[tokio::test]
async fn post_session_returns_200_for_valid_data() {
    let (app, mock_server, http_client, config, mock) = init().await;
    let argon2_instance = config.argon2_instance();
    mock.mount(&mock_server).await;

    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    // lets assume that there is already a user in the database,
    // we are already testing the registration in its own tests
    create_active_user(&app.db_pool, &argon2_instance, &email, &password).await;

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
    let (app, mock_server, http_client, config, mock) = init().await;
    let argon2_instance = config.argon2_instance();
    mock.mount(&mock_server).await;

    let email = SafeEmail().fake();
    let password: String = Password(8..16).fake();

    // lets assume that there is already a user in the database,
    // we are already testing the registration in its own tests
    create_active_user(&app.db_pool, &argon2_instance, &email, &password).await;

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
    let (app, mock_server, http_client, config, mock) = init().await;
    let argon2_instance = config.argon2_instance();
    mock.mount(&mock_server).await;

    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    // lets assume that there is already a user in the database,
    // we are already testing the registration in its own tests
    create_active_user(&app.db_pool, &argon2_instance, &email, &password).await;

    // Act
    let _ = TestApp::post_session(&http_client, &app.address, Some(email), Some(password))
        .await
        .expect("Couldn't send the request to the API.");

    // Assert
    // mock will verify that the email has been sent
}

#[tokio::test]
async fn post_session_rejects_if_the_user_is_inactive() {
    // Arrange
    let (app, _, http_client, config, _) = init().await;
    let argon2_instance = config.argon2_instance();

    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    create_inactive_user(&app.db_pool, &argon2_instance, &email, &password).await;

    // Act
    let response = TestApp::post_session(&http_client, &app.address, Some(email), Some(password))
        .await
        .expect("Couldn't send the request to the API.");
    let status = response.status();
    let text_body = response.text().await.unwrap();

    // Assert
    assert_eq!(status, 403);
    assert_eq!(text_body, String::from("USER_INACTIVE"));
}

#[tokio::test]
async fn post_session_returns_400_for_invalid_email() {
    // Arrange
    let (app, _, http_client, _, _mock) = init().await;

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
    let (app, _, http_client, config, _mock) = init().await;
    let argon2_instance = config.argon2_instance();

    // no mock because it shouldn't send any email anyways

    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let original_password: String = Password(1..16).fake();
    let fake_password: String = Password(1..16).fake();

    create_active_user(&app.db_pool, &argon2_instance, &email, &original_password).await;

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
    let (app, _, http_client, _, _mock) = init().await;

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
   let (app, _, http_client, _, _mock) = init().await;

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
    let (app, _, http_client, _, _mock) = init().await;

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
    let (app, _, http_client, _, _mock) = init().await;

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
    let (app, _, http_client, _, _mock) = init().await;

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