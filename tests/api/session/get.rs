use std::collections::HashMap;

use authen::model::email::Email;
use fake::{Fake, faker::{internet::en::{Password, SafeEmail}, lorem::en::Word}};
use uuid::Uuid;

use crate::helpers::{app::TestApp, database::commands::create_active_user, init, mock::{get_login_confirmation_code_from_request, get_request_from_mock_server}};


#[derive(serde::Deserialize)]
struct LoginResponseBody {
    confirmation_id: Uuid
}

#[derive(serde::Deserialize)]
struct LoginConfirmationResponseBody {
    token: String
}

#[derive(serde::Deserialize)]
struct SessionInfoResponseBody {
    #[allow(unused)]
    id: Uuid,
    #[allow(unused)]
    email: Email,
    #[allow(unused)]
    password_hash: String
}

#[tokio::test]
pub async fn get_session_responds_with_desired_data() {
    let (app, mock_server, http_client, config, mock) = init().await;
    let argon2_instance = config.argon2_instance();
    mock.mount(&mock_server).await;

    let email: String = SafeEmail().fake();
    let password: String = Password(4..16).fake();

    // Act
    create_active_user(&app.db_pool, &argon2_instance, &email, &password).await;

    // login
    let confirmation_id = {
        let response: LoginResponseBody = TestApp::post_session(&http_client, &app.address, Some(email), Some(password))
            .await
            .expect("Couldn't send the request to the API.")
            .json()
            .await
            .unwrap();
        response.confirmation_id
    };

    // retrieve the confirmation code
    let request = get_request_from_mock_server(&mock_server, 0).await;
    let code = get_login_confirmation_code_from_request(request, config).await;

    // confirm the login, retrieve the token
    let token = {
        let response: LoginConfirmationResponseBody = TestApp::post_confirmations_login(&http_client, &app.address, confirmation_id.to_string(), Some(code))
            .await
            .expect("Couldn't send the request to the API.")
            .json()
            .await
            .unwrap();
        response.token
    };

    let response = TestApp::get_session(&http_client, &app.address, String::from("Bearer"), Some(token), vec![String::from("id"), String::from("email"), String::from("password_hash")])
        .await
        .expect("Couldn't send the request to the API.");
    let status = response.status();
    let _: SessionInfoResponseBody = response.json().await.expect("Wrong body in response");

    // Assert
    assert_eq!(status, 200);
}

#[tokio::test]
pub async fn get_session_narrows_the_data_to_desired_shape() {
    let (app, mock_server, http_client, config, mock) = init().await;
    let argon2_instance = config.argon2_instance();
    mock.mount(&mock_server).await;

    let email: String = SafeEmail().fake();
    let password: String = Password(4..16).fake();

    // Act
    create_active_user(&app.db_pool, &argon2_instance, &email, &password).await;

    // login
    let confirmation_id = {
        let response: LoginResponseBody = TestApp::post_session(&http_client, &app.address, Some(email), Some(password))
            .await
            .expect("Couldn't send the request to the API.")
            .json()
            .await
            .unwrap();
        response.confirmation_id
    };

    // retrieve the confirmation code
    let request = get_request_from_mock_server(&mock_server, 0).await;
    let code = get_login_confirmation_code_from_request(request, config).await;

    // confirm the login, retrieve the token
    let token = {
        let response: LoginConfirmationResponseBody = TestApp::post_confirmations_login(&http_client, &app.address, confirmation_id.to_string(), Some(code))
            .await
            .expect("Couldn't send the request to the API.")
            .json()
            .await
            .unwrap();
        response.token
    };

    let response = TestApp::get_session(&http_client, &app.address, String::from("Bearer"), Some(token), vec![String::from("id")])
        .await
        .expect("Couldn't send the request to the API.");
    let status = response.status();
    let body: HashMap<String, String> = response.json().await.expect("Wrong body in response");

    // Assert
    assert_eq!(status, 200);
    assert!(body.contains_key("id"));
    assert!(!body.contains_key("email"));
    assert!(!body.contains_key("password_hash"));
}

#[tokio::test]
pub async fn get_session_rejects_missing_token() {
    let (app, _, http_client, _, _) = init().await;

    // Act
    let status = {
        let response = TestApp::get_session(&http_client, &app.address, String::from("Bearer"), None, vec![])
            .await
            .expect("Couldn't send the request to the API.");
        response.status()
    };

    // Assert
    assert_eq!(status, 401);
}

#[tokio::test]
pub async fn get_session_rejects_invalid_authorization_type() {
    let (app, _, http_client, _, _) = init().await;

    // Act
    let status = {
        let authorization_type: String = Word().fake();

        // some token, wrong authorization type
        let response = TestApp::get_session(&http_client, &app.address, authorization_type, Some(String::new()), vec![])
            .await
            .expect("Couldn't send the request to the API.");
        response.status()
    };

    // Assert
    assert_eq!(status, 401);
}

#[tokio::test]
pub async fn get_session_rejects_invalid_token() {
    let (app, _, http_client, _, _) = init().await;

    // Act
    let status = {
        // some token, wrong authorization type
        let response = TestApp::get_session(&http_client, &app.address, String::from("Bearer"), Some(Word().fake()), vec![])
            .await
            .expect("Couldn't send the request to the API.");
        response.status()
    };

    // Assert
    assert_eq!(status, 401);
}