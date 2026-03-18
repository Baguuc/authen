use authen::utils::generation::generate_confirmation_code;
use fake::{Fake, faker::{internet::en::{Password, SafeEmail}, lorem::en::Word}};
use sqlx::Row;
use uuid::Uuid;
use crate::helpers::{TestApp, create_active_user, get_login_confirmation_code_from_request, init};

#[derive(serde::Deserialize)]
struct LoginResponseBody {
    confirmation_id: Uuid
}

#[derive(serde::Deserialize)]
struct LoginConfirmationResponseBody {
    #[allow(unused)]
    token: String
}

#[tokio::test]
async fn post_confirmations_login_returns_a_session_token() {
    let (app, mock_server, http_client, _, mock) = init().await;
    mock.mount(&mock_server).await;

    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    create_active_user(&app.db_pool, &email, &password).await;

    // Act
    // register the user
    let response = TestApp::post_session(&http_client, &app.address, Some(email), Some(password))
        .await
        .expect("Couldn't send the request to the API.");
    let json_body_data: LoginResponseBody = response.json()
        .await
        .expect("Wrong login response.");

    let confirmation_id = json_body_data.confirmation_id;
    let confirmation_code = get_login_confirmation_code_from_request(&mock_server, 0).await;

    let response = TestApp::post_confirmations_login(&http_client, &app.address, confirmation_id.to_string(), Some(confirmation_code))
        .await
        .expect("Couldn't send the request to the API.");
    let post_confirmations_status = response.status();
    let deserialization_result: Result<LoginConfirmationResponseBody, reqwest::Error> = response.json().await;

    assert_eq!(post_confirmations_status, 200);
    assert!(deserialization_result.is_ok())
}

#[tokio::test]
async fn post_confirmations_login_deletes_the_code() {
    let (app, mock_server, http_client, _, mock) = init().await;
    mock.mount(&mock_server).await;

    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    create_active_user(&app.db_pool, &email, &password).await;

    // Act
    // register the user
    let response = TestApp::post_session(&http_client, &app.address, Some(email), Some(password))
        .await
        .expect("Couldn't send the request to the API.");
    let json_body_data: LoginResponseBody = response.json()
        .await
        .expect("Wrong login response.");

    let confirmation_id = json_body_data.confirmation_id;
    let confirmation_code = get_login_confirmation_code_from_request(&mock_server, 0).await;

    let response = TestApp::post_confirmations_login(&http_client, &app.address, confirmation_id.to_string(), Some(confirmation_code))
        .await
        .expect("Couldn't send the request to the API.");
    let post_confirmations_status = response.status();

    let code_number: i64 = sqlx::query("SELECT COUNT(*) as row_count FROM confirmation_codes;")
        .fetch_one(&app.db_pool)
        .await
        .unwrap()
        .get(0);
    let code_exists = code_number > 0;

    assert_eq!(post_confirmations_status, 200);
    assert!(!code_exists);
}

#[tokio::test]
async fn post_confirmations_login_rejects_wrong_code() {
    let (app, mock_server, http_client, _, mock) = init().await;
    mock.mount(&mock_server).await;

    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let password: String = Password(1..16).fake();

    create_active_user(&app.db_pool, &email, &password).await;

    // Act
    // register the user
    let response = TestApp::post_session(&http_client, &app.address, Some(email), Some(password))
        .await
        .expect("Couldn't send the request to the API.");
    let post_session_status = response.status();
    let json_body_data: LoginResponseBody = response.json()
        .await
        .expect("Wrong login response.");
    let confirmation_id = json_body_data.confirmation_id;

    // send random code
    let response = TestApp::post_confirmations_login(&http_client, &app.address, confirmation_id.to_string(), Some(generate_confirmation_code().as_ref().to_string()))
        .await
        .expect("Couldn't send the request to the API.");
    let post_confirmations_status = response.status();

    assert_eq!(post_session_status, 200);
    assert_eq!(post_confirmations_status, 401);
}

#[tokio::test]
async fn post_confirmations_login_rejects_request_with_code_missing() {
    let (app, _, http_client, _, _mock) = init().await;
    
    // try to post with no code
    let status = {
        // no need to provide a existing id as its existance is checked after deserialization.
        let id = Uuid::new_v4().to_string();
        let response = TestApp::post_confirmations_login(&http_client, &app.address, id, None)
            .await
            .expect("Couldn't send the request to the API.");

        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn post_confirmations_login_rejects_invalid_login_code_id() {
    let (app, _, http_client, _, _mock) = init().await;
    
    // Act
    // register the user
    // a word is not a valid Uuid obviously
    let status = {
        let response = TestApp::post_confirmations_login(&http_client, &app.address, Word().fake(), None)
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    assert_eq!(status, 404);
}

#[tokio::test]
async fn post_confirmations_login_rejects_login_code_id_not_existing() {
    let (app, _, http_client, _, _mock) = init().await;

    // Act
    // register the user
    let status = {
        // random Uuid that doesn't exist
        let id = Uuid::new_v4().to_string();
        let response = TestApp::post_confirmations_login(&http_client, &app.address, id, Some(String::from("123456")))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    assert_eq!(status, 404);
}

#[tokio::test]
async fn post_confirmations_login_rejects_request_with_code_with_invalid_chars() {
    let (app, _, http_client, _, _mock) = init().await;

    // try to post with invalid code
    let status = {
        // no need to provide a existing id as its existance is checked after deserialization.
        let id = Uuid::new_v4().to_string();
        let response = TestApp::post_confirmations_login(&http_client, &app.address, id, Some(String::from("*&#-()")))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn post_confirmations_login_rejects_request_with_code_with_invalid_lenght() {
    let (app, _, http_client, _, _mock) = init().await;

    // try to post with invalid code lenght
    let status = {
        // no need to provide a existing id as its existance is checked after deserialization.
        let id = Uuid::new_v4().to_string();
        let response = TestApp::post_confirmations_login(&http_client, &app.address, id, Some(String::from("123")))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}