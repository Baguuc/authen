use actix_web::{HttpResponse, web::{Json, Data}};
use serde::{Deserialize, Serialize};
use sqlx::{Connection, PgPool};
use tracing::instrument;
use uuid::Uuid;

use crate::{clients::email::EmailClient, command::confirmation_code::create::create_confirmation_code, settings::Settings, error::{api::session::SessionCreationError, query::user::{GetUserIdError,UserPasswordVerificationError}}, model::{confirmation_code_type::ConfirmationCodeType, email::Email}, query::user::{get_user_id::get_user_id_from_email, verify_password::verify_user_password}, utils::error::log_map};

/// Helper struct to deserialize data from request's json body.
#[derive(Deserialize, Debug)]
pub struct JsonData {
    email: Email,
    password: String
}

#[derive(Serialize, Debug)]
pub struct ResponseBody {
    confirmation_id: Uuid
}

/// User registration confirmation endpoint available @ POST /api/confirmations/registration/{}
#[instrument(name = "Authenticating a user", skip(db_conn, email_client, config))]
pub async fn post_session(
    Json(body): Json<JsonData>,
    db_conn: Data<PgPool>,
    email_client: Data<EmailClient>,
    config: Data<Settings>
) -> actix_web::Result<HttpResponse, SessionCreationError> {
    tracing::debug!("Acquiring the database connection.");
    let mut db_conn = db_conn.acquire()
        .await
        .map_err(|err| log_map(format!("Cannot acquire the connection to the database.\n{}", err), SessionCreationError::UnexpectedError))?;

    tracing::debug!("Begining a transaction");
    let mut transaction = db_conn.begin()
        .await
        .map_err(|err| log_map(format!("Cannot begin the transaction.\n{}", err), SessionCreationError::UnexpectedError))?;

    tracing::info!("Getting user id.");
    let user_id = match get_user_id_from_email(&mut *transaction, &body.email).await {
        Err(GetUserIdError::NotExists) => return Err(SessionCreationError::UserNotExists),
        Err(GetUserIdError::Sqlx(err)) => return log_map(err, Err(SessionCreationError::UnexpectedError)),
        Ok(user_id) => user_id,
    };

    let argon2_instance = config.argon2_instance();

    tracing::info!("Verifying the users password.");
    match verify_user_password(&mut *transaction, &argon2_instance, &user_id, &body.password).await {
        Ok(false) => return Err(SessionCreationError::WrongPassword),
        Err(UserPasswordVerificationError::NotExists) => return Err(SessionCreationError::UserNotExists),
        Err(UserPasswordVerificationError::Sqlx(err)) => return log_map(err, Err(SessionCreationError::UnexpectedError)),
        _ => ()
    };

    tracing::info!("Creating the registration code.");
    let (confirmation_id, code) = create_confirmation_code(&mut *transaction, &argon2_instance, user_id, ConfirmationCodeType::Login)
        .await
        // unexpected because no error should happen
        .map_err(|err| log_map(err, SessionCreationError::UnexpectedError))?;

    tracing::info!("Sending the confirmation code.");
    let sender_email = config.email.sender.clone();
    let result = email_client.send_email(
        sender_email,
        body.email,
        // content customization and link will be implemented in the near future
        String::from("Confirm your account"),
        format!("Confirm your account using the code {}", code),
        format!("<b>Confirm your account using the code {}<b>", code)
    )
    .await;

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