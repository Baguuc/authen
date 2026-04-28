use authen::auth::otp::generate_confirmation_code;
use fake::{Fake, faker::{internet::en::{Password, SafeEmail}, lorem::en::Word}};
use sqlx::Row;
use uuid::Uuid;

use crate::helpers::{app::TestApp, database::commands::create_active_user, init, mock::{get_login_confirmation_code_from_request, get_request_from_mock_server}};

#[derive(serde::Deserialize)]
struct LoginResponseBody {
    confirmation_id: Uuid
}

#[tokio::test]
async fn delete_confirmations_login_deletes_the_code() {
    let (app, mock_server, http_client, config, mock) = init().await;
    mock.mount(&mock_server).await;

    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    create_active_user(&app.db_pool, &config.argon2_instance(), &email, &password).await;

    // Act
    // register the user
    let response: LoginResponseBody = TestApp::post_session(&http_client, &app.address, Some(email), Some(password))
        .await
        .expect("Couldn't send the request to the API.")
        .json()
        .await
        .expect("Wrong response shape.");
    let confirmation_id = response.confirmation_id;
    let recieved_request = get_request_from_mock_server(&mock_server, 0).await;
    let confirmation_code = get_login_confirmation_code_from_request(recieved_request, config).await;

    let response = TestApp::delete_confirmations_login(&http_client, &app.address, confirmation_id.to_string(), Some(confirmation_code))
        .await
        .expect("Couldn't send the request to the API.");
    let status = response.status();

    let code_number: i64 = sqlx::query("SELECT COUNT(*) as row_count FROM confirmation_codes;")
        .fetch_one(&app.db_pool)
        .await
        .unwrap()
        .get(0);
    let code_exists = code_number > 0;

    assert_eq!(status, 200);
    assert!(!code_exists);
}

#[tokio::test]
async fn delete_confirmations_login_rejects_wrong_code() {
    let (app, mock_server, http_client, config, mock) = init().await;
    mock.mount(&mock_server).await;

    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    create_active_user(&app.db_pool, &config.argon2_instance(), &email, &password).await;

    // Act
    // register the user
    let response: LoginResponseBody = TestApp::post_session(&http_client, &app.address, Some(email), Some(password))
        .await
        .expect("Couldn't send the request to the API.")
        .json()
        .await
        .expect("Wrong response shape.");
    let confirmation_id = response.confirmation_id;

    // try confirming with wrong code
    let response = TestApp::delete_confirmations_login(&http_client, &app.address, confirmation_id.to_string(), Some(generate_confirmation_code().as_ref().to_string()))
        .await
        .expect("Couldn't send the request to the API.");
    let status = response.status();

    assert_eq!(status, 401);
}

#[tokio::test]
async fn delete_confirmations_login_rejects_request_with_code_missing() {
    let (app, _, http_client, _, _mock) = init().await;
    
    // try to delete with no code
    let status = {
        // no need to provide a existing id as its existance is checked after deserialization.
        let id = Uuid::new_v4().to_string();
        let response = TestApp::delete_confirmations_login(&http_client, &app.address, id, None)
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn delete_confirmations_login_rejects_invalid_login_code_id() {
    let (app, _, http_client, _, _mock) = init().await;

    // Act
    // register the user
    let status = {
        let response = TestApp::delete_confirmations_login(&http_client, &app.address, Word().fake(), None)
            .await
            .expect("Couldn't send the request to the API.");

        response.status()
    };

    assert_eq!(status, 404);
}

#[tokio::test]
async fn delete_confirmations_login_rejects_login_code_id_not_existing() {
    let (app, _, http_client, _, _mock) = init().await;

    // Act
    // register the user
    // random Uuid that doesn't exist
    let status = {
        let response = TestApp::delete_confirmations_login(&http_client, &app.address, Uuid::new_v4().to_string(), Some(String::from("123456")))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    assert_eq!(status, 404);
}

#[tokio::test]
async fn delete_confirmations_login_rejects_request_with_code_with_invalid_chars() {
    let (app, _, http_client, _, _mock) = init().await;

    // try to post with invalid code
    let status = {
        // no need to provide a existing id as its existance is checked after deserialization.
        let id = Uuid::new_v4().to_string();
        let response = TestApp::delete_confirmations_login(&http_client, &app.address, id, Some(String::from("*&#-()")))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn delete_confirmations_login_rejects_request_with_code_with_invalid_lenght() {
    let (app, _, http_client, _, _mock) = init().await;

    // try to post with invalid code lenght
    let status = {
        // no need to provide a existing id as its existance is checked after deserialization.
        let id = Uuid::new_v4().to_string();
        let response = TestApp::delete_confirmations_login(&http_client, &app.address, id, Some(String::from("123")))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}
