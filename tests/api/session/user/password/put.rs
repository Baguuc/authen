use authen::auth::jwt::generate_user_token;
use fake::{Fake, faker::{internet::ar_sa::{Password, SafeEmail}, lorem::en::Word}};
use serde::Deserialize;
use uuid::Uuid;

use crate::helpers::{app::TestApp, database::commands::create_active_user, init};

#[derive(Deserialize)]
pub struct Response {
    #[allow(unused)]
    confirmation_id: Uuid
}

#[tokio::test]
async fn put_session_user_password_returns_200_for_valid_data() {
    // arrange
    let (app, mock_server, http_client, config, mock) = init().await;
    let argon2_instance = config.argon2_instance();
    mock.mount(&mock_server).await;

    // step 1 - create a user
    let email: String = SafeEmail().fake();
    let password: String = Password(1..16).fake();
    let user_id = create_active_user(&app.db_pool, &argon2_instance, &email, &password).await;

    // step 2 - log in
    // intentionally skipping the register and login routes
    // to keep the tests atomic
    let user_token = generate_user_token(&config.jwt.hashing_key, &config.jwt_header(), config.jwt_expires_in(), user_id).unwrap();

    // act
    let new_password: String = Password(4..16).fake();
    let response = TestApp::put_session_user_password(
        &http_client,
        &app.address,
        String::from("Bearer"),
        Some(String::from(user_token)),
        Some(password),
        Some(new_password)
    )
    .await
    .unwrap();

    assert_eq!(response.status(), 200);
    // if the deserialization failes it means that wrong body shape was sent back
    let _: Response = response.json().await.expect("Wrong body shape");
}

#[tokio::test]
async fn put_session_user_password_stores_confirmation_with_valid_data() {
    // arrange
    let (app, mock_server, http_client, config, mock) = init().await;
    let argon2_instance = config.argon2_instance();
    mock.mount(&mock_server).await;

    // step 1 - create a user
    let email: String = SafeEmail().fake();
    let password: String = Password(1..16).fake();
    let user_id = create_active_user(&app.db_pool, &argon2_instance, &email, &password).await;

    // step 2 - log in
    // intentionally skipping the register and login routes
    // to keep the tests atomic
    let user_token = generate_user_token(&config.jwt.hashing_key, &config.jwt_header(), config.jwt_expires_in(), user_id).unwrap();

    // act
    let new_password: String = Password(4..16).fake();
    TestApp::put_session_user_password(
        &http_client,
        &app.address,
        String::from("Bearer"),
        Some(String::from(user_token)),
        Some(password),
        Some(new_password)
    )
    .await
    .unwrap();

    // assert
    let result = sqlx::query("SELECT 1 FROM confirmation_codes;")
        .execute(&app.db_pool)
        .await
        .unwrap();

    assert_eq!(result.rows_affected(), 1);
}

#[tokio::test]
async fn put_session_user_password_sends_confirmation_email_for_valid_data() {
    // arrange
    let (app, mock_server, http_client, config, mock) = init().await;
    let argon2_instance = config.argon2_instance();
    mock.mount(&mock_server).await;

    // step 1 - create a user
    let email: String = SafeEmail().fake();
    let password: String = Password(1..16).fake();
    let user_id = create_active_user(&app.db_pool, &argon2_instance, &email, &password).await;

    // step 2 - log in
    // intentionally skipping the register and login routes
    // to keep the tests atomic
    let user_token = generate_user_token(&config.jwt.hashing_key, &config.jwt_header(), config.jwt_expires_in(), user_id).unwrap();

    // act
    let new_password: String = Password(4..16).fake();
    TestApp::put_session_user_password(
        &http_client,
        &app.address,
        String::from("Bearer"),
        Some(String::from(user_token)),
        Some(password),
        Some(new_password)
    )
    .await
    .unwrap();

    // assert
    // mock asserts on drop
}

#[tokio::test]
async fn put_session_user_password_rejects_wrong_auth_type() {
    // arrange
    let (app, _, http_client, config, _) = init().await;
    let _ = config.argon2_instance();

    // act 
    let response = TestApp::put_session_user_password(
        &http_client,
        &app.address,
        // invalid auth type
        Word().fake(),
        None,
        Some(Password(4..16).fake()),
        Some(Password(4..16).fake())
    )
    .await
    .unwrap();

    // assert
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn put_session_user_password_rejects_no_token() {
    // arrange
    let (app, _, http_client, config, _) = init().await;
    let _ = config.argon2_instance();

    // act
    let response = TestApp::put_session_user_password(
        &http_client,
        &app.address,
        String::from("Bearer"),
        None,
        Some(Password(4..16).fake()),
        Some(Password(4..16).fake())
    )
    .await
    .unwrap();

    // assert
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn put_session_user_password_rejects_invalid_token() {
    // arrange
    let (app, _, http_client, config, _) = init().await;

    // act
    let response = TestApp::put_session_user_password(
        &http_client,
        &app.address,
        String::from("Bearer"),
        Some(Word().fake()),
        Some(Password(4..16).fake()),
        Some(Password(4..16).fake())
    )
    .await
    .unwrap();

    // assert
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn put_session_user_password_rejects_no_password() {
    // arrange
    let (app, _, http_client, config, _) = init().await;
    let argon2_instance = config.argon2_instance();

     // step 1 - create a user
    let email: String = SafeEmail().fake();
    let password: String = Password(1..16).fake();
    let user_id = create_active_user(&app.db_pool, &argon2_instance, &email, &password).await;

    // step 2 - log in
    // intentionally skipping the register and login routes
    // to keep the tests atomic
    let user_token = generate_user_token(&config.jwt.hashing_key, &config.jwt_header(), config.jwt_expires_in(), user_id).unwrap();

    // act
    let response = TestApp::put_session_user_password(
        &http_client,
        &app.address,
        String::from("Bearer"),
        Some(user_token),
        None,
        Some(Password(4..16).fake())
    )
    .await
    .unwrap();

    // assert
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn put_session_user_password_rejects_invalid_password() {
    // arrange
    let (app, _, http_client, config, _) = init().await;
    let argon2_instance = config.argon2_instance();

     // step 1 - create a user
    let email: String = SafeEmail().fake();
    let password: String = Password(1..16).fake();
    let user_id = create_active_user(&app.db_pool, &argon2_instance, &email, &password).await;

    // step 2 - log in
    // intentionally skipping the register and login routes
    // to keep the tests atomic
    let user_token = generate_user_token(&config.jwt.hashing_key, &config.jwt_header(), config.jwt_expires_in(), user_id).unwrap();


    // act
    let response = TestApp::put_session_user_password(
        &http_client,
        &app.address,
        String::from("Bearer"),
        Some(user_token),
        Some(Password(4..16).fake()),
        Some(Password(4..16).fake())
    )
    .await
    .unwrap();

    // assert
    assert_eq!(response.status(), 403);
}
