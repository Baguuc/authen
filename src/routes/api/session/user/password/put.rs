use actix_web::{HttpResponse, web::{Data, Json}};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::{auth::{hash::hash_string, jwt::deserialize_claims_from_user_token}, clients::email::EmailClient, command::{confirmation_code::create::create_confirmation_code, update_data::create::add_update_data_to_confirmation_code}, error::{api::session::SessionUserUpdatePasswordError, query::user::RetrieveUserError}, extractor::user_token::UserTokenExtractor, model::{confirmation_code_type::ConfirmationCodeType, email::Email}, query::user::{retrieve::retrieve_user, verify_password::verify_user_password}, settings::Settings, utils::error::log_map};

#[derive(Deserialize, Debug)]
pub struct JsonBody {
    password: SecretString,
    new_password: SecretString
}

#[derive(Serialize, Debug)]
pub struct ResponseBody {
    confirmation_id: Uuid
}

/// Session user password updating endpoint
#[instrument(name = "Creating a session user's password update request", skip(db_conn, config, user_token, email_client))]
pub async fn put_session_user_password(
    body: Json<JsonBody>,
    user_token: UserTokenExtractor,
    db_conn: Data<PgPool>,
    config: Data<Settings>,
    email_client: Data<EmailClient>
) -> actix_web::Result<HttpResponse, SessionUserUpdatePasswordError> {
    let argon2_instance = config.argon2_instance();
    

    // 1. Begin a transaction
    tracing::debug!("Beginning a database transaction.");
    let mut transaction = db_conn.begin()
        .await
        .map_err(|err| log_map(format!("Cannot acquire the connection to the database.\n{}", err), SessionUserUpdatePasswordError::UnexpectedError))?;

    
    // 2. Decode the user token
    let claims = match deserialize_claims_from_user_token(&
        config.jwt.hashing_key,
        &config.jwt_validation(),
        user_token.as_ref()
    ) {
        Ok(claims) => claims,
        Err(_) => return Err(SessionUserUpdatePasswordError::InvalidToken)
    };
    let user_id = claims.sub;
    

    // 3. Verify the user's password
    // this is done although requiring session token which implies user already logged in
    // to prevent a token-jacking scenario.
    match verify_user_password(&mut transaction, &argon2_instance, &user_id, &body.password).await {
        Ok(false) => return Err(SessionUserUpdatePasswordError::InvalidPassword),
        Err(err) => return Err(log_map(err, SessionUserUpdatePasswordError::UnexpectedError)),
        _ => ()
    };


    // 4. Retrieve user's data from the database.
    let user = match retrieve_user(&mut *transaction, user_id).await {
        Ok(user) => user,
        // when the user from the token do not exist it implies that the token is invalid
        Err(RetrieveUserError::NotExists) => return Err(SessionUserUpdatePasswordError::UnexpectedError),
        Err(RetrieveUserError::Sqlx(err)) => return log_map(err, Err(SessionUserUpdatePasswordError::UnexpectedError))
    };


    // 5. Parse the user email
    // Done in case user somehow registered with invalid email address (for example because of invalid parsing rules).
    let user_email = match Email::parse(user.email) {
        Ok(email) => email,
        Err(err) => return log_map(
            format!("An invalid email was found inside the database.\nDetails from parsing error:\n{}", err), 
            Err(SessionUserUpdatePasswordError::UnexpectedError)
        )
    };


    // 6. Generate a confirmation code
    let (confirmation_id, code) = create_confirmation_code(&mut *transaction, &argon2_instance, user_id, ConfirmationCodeType::UpdateUserPassword)
        .await
        // unexpected because no error should happen
        .map_err(|err| log_map(err, SessionUserUpdatePasswordError::UnexpectedError))?;

    
    // 7. Hash the password
    let hashed_new_password = match hash_string(&body.new_password.expose_secret().to_string(), &argon2_instance) {
        Ok(hash) => hash,
        Err(err) => return Err(log_map(err, SessionUserUpdatePasswordError::UnexpectedError)),
    };


    // 8. Attach update data to the confirmation code
    match add_update_data_to_confirmation_code(&mut *transaction, &confirmation_id, &json!({ "password_hash": hashed_new_password, "user_email": user_email })).await {
        Ok(_) => (),
        Err(err) => return Err(log_map(err, SessionUserUpdatePasswordError::UnexpectedError)),
    };

    
    // 9. Send the confirmation code email
    tracing::info!("Sending the confirmation code.");
    let email_config = config.user_password_update_confirmation_email();
    let subject = String::from(email_config.subject.clone());
    let text_body = String::from(email_config.text_body.as_ref().replace("%code%", &code));
    let html_body = String::from(email_config.html_body.as_ref().replace("%code%", &code));

    let sender_email = config.email.sender.clone();
    let result = email_client.send_email(
        sender_email,
        user_email,
        // content customization and link will be implemented in the near future
        subject,
        text_body,
        html_body
    )
    .await;

    // 10. Match the result
    match result {
        Ok(_) => {
            // only commit when email was successful.
            // if not commited sqlx will rollback automatically on drop.
            transaction.commit()
                .await
                .map_err(|err| log_map(
                    format!("Unexpected error occured while commiting changes to the database.\n{}", err),
                    SessionUserUpdatePasswordError::UnexpectedError
                ))?;
            Ok(HttpResponse::Ok().json(ResponseBody {
                confirmation_id 
            }))
        },
        Err(err) => {
            log_map(
                format!("Email server has wrong configuration, couldn't send a confirmation email.\n{}", err),
                Err(SessionUserUpdatePasswordError::UnexpectedError)
            )
        }
    }
}