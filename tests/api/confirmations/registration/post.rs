use std::collections::HashMap;

use authen::{configuration::Settings, utils::generation::generate_confirmation_code};
use fake::{Fake, faker::{internet::en::{Password, SafeEmail}, lorem::en::Word}};
use sqlx::Row;
use uuid::Uuid;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::{method, path}};

use crate::helpers::{TestApp, spawn_app};

#[derive(serde::Deserialize)]
struct RegistrationResponseBody {
    confirmation_id: Uuid
}

#[tokio::test]
async fn post_confirmations_registration_changes_active_state_of_user() {
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
    // register the user
    let response: RegistrationResponseBody = TestApp::post_users(&http_client, &app.address, Some(email), Some(password))
        .await
        .expect("Couldn't send the request to the API.")
        .json()
        .await
        .expect("Wrong response shape.");
    let confirmation_id = response.confirmation_id;

    let user_is_active: bool = sqlx::query("SELECT active FROM users;")
        .fetch_one(&app.db_pool)
        .await
        .unwrap()
        .get(0);

    // check if user is initially inactive
    assert!(!user_is_active);

    let recieved_request_body: HashMap<String, String> = mock_server.received_requests()
        .await
        .expect("Mock email server haven't got the request.")
        .get(0)
        .unwrap()
        .body_json()
        .unwrap();
    let text_body: &String = recieved_request_body.get("TextBody")
        .expect("No TextBody in the request.");
    // for now we collect the code this way, I will change it in future when I will add the UI page
    // so that the email will be sending a link and the link will be scraped from the email.
    let confirmation_code = text_body.replace("Confirm your account using the code ", "");

    let response = TestApp::post_registrations_confirmation(&http_client, &app.address, confirmation_id.to_string(), Some(confirmation_code))
        .await
        .expect("Couldn't send the request to the API.");
    let status = response.status();

    let user_is_active: bool = sqlx::query("SELECT active FROM users;")
        .fetch_one(&app.db_pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(status, 200);
    assert!(user_is_active);
}

#[tokio::test]
async fn post_confirmations_registration_deletes_the_code() {
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
    // register the user
    let response: RegistrationResponseBody = TestApp::post_users(&http_client, &app.address, Some(email), Some(password))
        .await
        .expect("Couldn't send the request to the API.")
        .json()
        .await
        .expect("Wrong response shape.");
    let confirmation_id = response.confirmation_id;

    let recieved_request_body: HashMap<String, String> = mock_server.received_requests()
        .await
        .expect("Mock email server haven't got the request.")
        .get(0)
        .unwrap()
        .body_json()
        .unwrap();
    let text_body: &String = recieved_request_body.get("TextBody")
        .expect("No TextBody in the request.");
    // for now we collect the code this way, I will change it in future when I will add the UI page
    // so that the email will be sending a link and the link will be scraped from the email.
    let confirmation_code = text_body.replace("Confirm your account using the code ", "");

    let response = TestApp::post_registrations_confirmation(&http_client, &app.address, confirmation_id.to_string(), Some(confirmation_code))
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
    assert!(!code_exists)
}

#[tokio::test]
async fn post_confirmations_registration_rejects_wrong_code() {
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
    let original_password: String = Password(1..16).fake();

    // Act
    // register the user
    let response: RegistrationResponseBody = TestApp::post_users(&http_client, &app.address, Some(email), Some(original_password))
        .await
        .expect("Couldn't send the request to the API.")
        .json()
        .await
        .expect("Wrong response shape.");
    let confirmation_id = response.confirmation_id;

    // try confirming with wrong code
    let response = TestApp::post_registrations_confirmation(&http_client, &app.address, confirmation_id.to_string(), Some(generate_confirmation_code().as_ref().to_string()))
        .await
        .expect("Couldn't send the request to the API.");
    let status = response.status();

    assert_eq!(status, 401);
}


#[tokio::test]
async fn post_confirmations_registration_rejects_request_with_code_missing() {
    let app = spawn_app(Some(String::new())).await;
    let http_client = reqwest::Client::new();
    
    // try to post with no code
    let status = {
        // no need to provide a existing id as its existance is checked after deserialization.
        let id = Uuid::new_v4().to_string();
        let response = TestApp::post_registrations_confirmation(&http_client, &app.address, id, None)
            .await
            .expect("Couldn't send the request to the API.");

        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn post_confirmations_registration_rejects_invalid_registration_code_id() {
    let mock_server = MockServer::start().await;
    let app = spawn_app(Some(mock_server.uri())).await;
    let http_client = reqwest::Client::new();

    // Act
    // register the user
    // a word is not a valid Uuid obviously
    let status = {
        let response = TestApp::post_registrations_confirmation(&http_client, &app.address, Word().fake(), None)
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    assert_eq!(status, 404);
}

#[tokio::test]
async fn post_confirmations_registration_rejects_registration_code_id_not_existing() {
    let mock_server = MockServer::start().await;
    let app = spawn_app(Some(mock_server.uri())).await;
    let http_client = reqwest::Client::new();

    // Act
    // register the user
    let status = {
        // random Uuid that doesn't exist
        let id = Uuid::new_v4().to_string();
        let response = TestApp::post_registrations_confirmation(&http_client, &app.address, id, Some(String::from("123456")))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    assert_eq!(status, 404);
}

#[tokio::test]
async fn post_confirmations_registration_rejects_request_with_code_with_invalid_chars() {
    let app = spawn_app(Some(String::new())).await;
    let http_client = reqwest::Client::new();

    // try to post with invalid code
    let status = {
        // no need to provide a existing id as its existance is checked after deserialization.
        let id = Uuid::new_v4().to_string();
        let response = TestApp::post_registrations_confirmation(&http_client, &app.address, id, Some(String::from("*&#-()")))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn post_confirmations_registration_rejects_request_with_code_with_invalid_lenght() {
    let app = spawn_app(Some(String::new())).await;
    let http_client = reqwest::Client::new();

    // try to post with invalid code lenght
    let status = {
        // no need to provide a existing id as its existance is checked after deserialization.
        let id = Uuid::new_v4().to_string();
        let response = TestApp::post_registrations_confirmation(&http_client, &app.address, id, Some(String::from("123")))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}