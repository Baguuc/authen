use authen::auth::{jwt::generate_user_token, otp::generate_confirmation_code};
use fake::{Fake, faker::{internet::en::{Password, SafeEmail}, lorem::en::Word}};
use uuid::Uuid;
use crate::helpers::{app::TestApp, database::{commands::{create_active_user, get_user_password_hash}, queries::{get_confirmation_code_count, get_update_data_count}}, init, mock::{get_request_from_mock_server, get_user_password_update_confirmation_code_from_request}};

#[derive(serde::Deserialize)]
struct RegistrationResponseBody {
    confirmation_id: Uuid
}

#[tokio::test]
async fn delete_confirmations_user_update_password_not_changes_user_password() {
    let (app, mock_server, http_client, config, mock) = init().await;
    mock.mount(&mock_server).await;
    let argon2_instance = config.argon2_instance();
    
    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let original_password: String = Password(4..16).fake();
    let new_password: String = Password(4..16).fake();

    let user_id = create_active_user(&app.db_pool, &argon2_instance, &email, &original_password).await;
    let user_token = generate_user_token(&config.jwt.hashing_key, &config.jwt_header(), config.jwt_expires_in(), user_id).unwrap();

    let original_hash = get_user_password_hash(&app.db_pool, &user_id).await;

    // Act
    let response: RegistrationResponseBody = TestApp::put_session_user_password(&http_client, &app.address, String::from("Bearer"), Some(user_token), Some(String::from(original_password)), Some(String::from(new_password)))
        .await
        .expect("Couldn't send the request to the API.")
        .json()
        .await
        .expect("Wrong response shape.");
    let confirmation_id = response.confirmation_id;

    let recieved_request = get_request_from_mock_server(&mock_server, 0).await;
    let confirmation_code = get_user_password_update_confirmation_code_from_request(recieved_request, config).await;
    
    // try confirming with wrong code
    let response = TestApp::delete_confirmations_user_update_password(&http_client, &app.address, confirmation_id.to_string(), Some(confirmation_code))
        .await
        .expect("Couldn't send the request to the API.");
    let status = response.status();

    let new_hash = get_user_password_hash(&app.db_pool, &user_id).await;
    
    // Assert
    assert_eq!(status, 200);
    assert_eq!(original_hash, new_hash);
}

#[tokio::test]
async fn delete_confirmations_user_update_password_deletes_the_code_and_update_data() {
    let (app, mock_server, http_client, config, mock) = init().await;
    mock.mount(&mock_server).await;
    let argon2_instance = config.argon2_instance();
    
    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let original_password: String = Password(4..16).fake();
    let new_password: String = Password(4..16).fake();

    let user_id = create_active_user(&app.db_pool, &argon2_instance, &email, &original_password).await;
    let user_token = generate_user_token(&config.jwt.hashing_key, &config.jwt_header(), config.jwt_expires_in(), user_id).unwrap();

    // Act
    let response: RegistrationResponseBody = TestApp::put_session_user_password(&http_client, &app.address, String::from("Bearer"), Some(user_token), Some(String::from(original_password)), Some(String::from(new_password)))
        .await
        .expect("Couldn't send the request to the API.")
        .json()
        .await
        .expect("Wrong response shape.");
    let confirmation_id = response.confirmation_id;

    let recieved_request = get_request_from_mock_server(&mock_server, 0).await;
    let confirmation_code = get_user_password_update_confirmation_code_from_request(recieved_request, config).await;

    // try confirming with wrong code
    let _ = TestApp::delete_confirmations_user_update_password(&http_client, &app.address, confirmation_id.to_string(), Some(confirmation_code))
        .await
        .expect("Couldn't send the request to the API.");

    let updates_data_count = get_update_data_count(&app.db_pool).await;
    let confirmation_code_count = get_confirmation_code_count(&app.db_pool).await;

    assert_eq!(updates_data_count, 0);
    assert_eq!(confirmation_code_count, 0);
}

#[tokio::test]
async fn delete_confirmations_user_update_password_rejects_wrong_code() {
    let (app, mock_server, http_client, config, mock) = init().await;
    mock.mount(&mock_server).await;
    let argon2_instance = config.argon2_instance();
    
    let email: String = SafeEmail().fake();
    // any password length should be fine, testing only up to 16 characters as further is not needed.
    let original_password: String = Password(4..16).fake();
    let new_password: String = Password(4..16).fake();

    let user_id = create_active_user(&app.db_pool, &argon2_instance, &email, &original_password).await;
    let user_token = generate_user_token(&config.jwt.hashing_key, &config.jwt_header(), config.jwt_expires_in(), user_id).unwrap();

    // Act
    let response: RegistrationResponseBody = TestApp::put_session_user_password(&http_client, &app.address, String::from("Bearer"), Some(user_token), Some(String::from(original_password)), Some(String::from(new_password)))
        .await
        .expect("Couldn't send the request to the API.")
        .json()
        .await
        .expect("Wrong response shape.");
    let confirmation_id = response.confirmation_id;

    // try confirming with wrong code
    let response = TestApp::delete_confirmations_user_update_password(&http_client, &app.address, confirmation_id.to_string(), Some(generate_confirmation_code().as_ref().to_string()))
        .await
        .expect("Couldn't send the request to the API.");
    let status = response.status();

    assert_eq!(status, 401);
}

#[tokio::test]
async fn delete_confirmations_user_update_password_rejects_request_with_no_code() {
    let (app, _, http_client, _, _mock) = init().await;
    
    // try to delete with no code
    let status = {
        // no need to provide a existing id as its existance is checked after deserialization.
        let id = Uuid::new_v4().to_string();
        let response = TestApp::delete_confirmations_user_update_password(&http_client, &app.address, id, None)
            .await
            .expect("Couldn't send the request to the API.");

        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn delete_confirmations_user_update_password_rejects_invalid_confirmation_id() {
    let (app, _, http_client, _, _mock) = init().await;

    // Act
    // register the user
    // a word is not a valid Uuid obviously
    let status = {
        let response = TestApp::delete_confirmations_user_update_password(&http_client, &app.address, Word().fake(), None)
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    assert_eq!(status, 404);
}

#[tokio::test]
async fn delete_confirmations_user_update_password_rejects_confirmation_id_not_existing() {
    let (app, _, http_client, _, _mock) = init().await;

    // Act
    // register the user
    let status = {
        // random Uuid that doesn't exist
        let id = Uuid::new_v4().to_string();
        let response = TestApp::delete_confirmations_user_update_password(&http_client, &app.address, id, Some(String::from("123456")))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    assert_eq!(status, 404);
}

#[tokio::test]
async fn delete_confirmations_user_update_password_rejects_code_with_invalid_chars() {
    let (app, _, http_client, _, _mock) = init().await;

    // try to delete with invalid code
    let status = {
        // no need to provide a existing id as its existance is checked after deserialization.
        let id = Uuid::new_v4().to_string();
        let response = TestApp::delete_confirmations_user_update_password(&http_client, &app.address, id, Some(String::from("*&#-()")))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}

#[tokio::test]
async fn delete_confirmations_user_update_password_rejects_code_with_invalid_lenght() {
    let (app, _, http_client, _, _mock) = init().await;

    // try to delete with invalid code lenght
    let status = {
        // no need to provide a existing id as its existance is checked after deserialization.
        let id = Uuid::new_v4().to_string();
        let response = TestApp::delete_confirmations_user_update_password(&http_client, &app.address, id, Some(String::from("123")))
            .await
            .expect("Couldn't send the request to the API.");
        
        response.status()
    };

    // Assert
    assert_eq!(status, 400);
}