use actix_web::{HttpResponse, web::{Json, Data}};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;
use secrecy::SecretString;
use crate::{clients::email::EmailClient, command::confirmation_code::create::create_confirmation_code, error::{api::session::SessionCreationError, query::user::{GetUserIdError, UserCheckIsActiveError, UserPasswordVerificationError}}, model::{confirmation_code_type::ConfirmationCodeType, email::Email}, query::user::{get_user_id::get_user_id_from_email, is_active::is_user_active, verify_password::verify_user_password}, settings::Settings, utils::error::log_map};

/// Helper struct to deserialize data from request's json body.
#[derive(Deserialize, Debug)]
pub struct JsonData {
    email: Email,
    password: SecretString
}

#[derive(Serialize, Debug)]
pub struct ResponseBody {
    confirmation_id: Uuid
}

/// User session creating endpoint
#[instrument(name = "Creating new user session.", skip(db_conn, email_client, config))]
pub async fn post_session(
    Json(body): Json<JsonData>,
    db_conn: Data<PgPool>,
    email_client: Data<EmailClient>,
    config: Data<Settings>
) -> actix_web::Result<HttpResponse, SessionCreationError> {
    let argon2_instance = config.argon2_instance();
    let email_config = config.login_confirmation_email();


    // 1. Begin a transaction
    tracing::debug!("Begining a transaction");
    let mut transaction = db_conn.begin()
        .await
        .map_err(|err| log_map(format!("Cannot begin the transaction.\n{}", err), SessionCreationError::UnexpectedError))?;

    
    // 2. Retrieve the user's id
    let user_id = match get_user_id_from_email(&mut *transaction, &body.email).await {
        Err(GetUserIdError::NotExists) => return Err(SessionCreationError::UserNotExists),
        Err(GetUserIdError::Sqlx(err)) => return log_map(err, Err(SessionCreationError::UnexpectedError)),
        Ok(user_id) => user_id,
    };

    // 3. Check if user is active
    match is_user_active(&mut *transaction, &user_id).await {
        Err(UserCheckIsActiveError::NotExists) => return Err(SessionCreationError::UserNotExists),
        Err(UserCheckIsActiveError::Sqlx(err)) => return log_map(err, Err(SessionCreationError::UnexpectedError)),
        Ok(false) => return Err(SessionCreationError::UserNotActive),
        _ => ()
    }


    // 4. Verify user's password
    match verify_user_password(&mut *transaction, &argon2_instance, &user_id, &body.password).await {
        Ok(false) => return Err(SessionCreationError::WrongPassword),
        Err(UserPasswordVerificationError::NotExists) => return Err(SessionCreationError::UserNotExists),
        Err(UserPasswordVerificationError::Sqlx(err)) => return log_map(err, Err(SessionCreationError::UnexpectedError)),
        _ => ()
    };

    
    // 5. Create a confirmation code
    let (confirmation_id, code) = create_confirmation_code(&mut *transaction, &argon2_instance, user_id, ConfirmationCodeType::Login)
        .await
        // unexpected because no error should happen
        .map_err(|err| log_map(err, SessionCreationError::UnexpectedError))?;


    // 6. Send the confirmation code to user's email
    tracing::info!("Sending the confirmation code.");
    
    let subject = String::from(email_config.subject.clone());
    let text_body = String::from(email_config.text_body.as_ref().replace("%code%", &code));
    let html_body = String::from(email_config.html_body.as_ref().replace("%code%", &code));

    let sender_email = config.email.sender.clone();
    let result = email_client.send_email(
        sender_email,
        body.email,
        // content customization and link will be implemented in the near future
        subject,
        text_body,
        html_body
    )
    .await;

    // 7. Match the result
    match result {
        Ok(_) => {
            // only commit when email was successful.
            // if not commited sqlx will rollback automatically on drop.
            transaction.commit()
                .await
                .map_err(|err| log_map(
                    format!("Unexpected error occured while commiting changes to the database.\n{}", err),
                    SessionCreationError::UnexpectedError
                ))?;
            Ok(HttpResponse::Ok().json(ResponseBody {
                confirmation_id 
            }))
        },
        Err(err) => {
            log_map(
                format!("Email server has wrong configuration, couldn't send a confirmation email.\n{}", err),
                Err(SessionCreationError::UnexpectedError)
            )
        }
    }
}