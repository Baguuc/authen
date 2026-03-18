use actix_web::{HttpResponse, web::{Data, Json}};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::{Connection, PgPool, error::ErrorKind};
use tracing::instrument;
use uuid::Uuid;

use crate::{clients::email::EmailClient, command::{confirmation_code::create::create_confirmation_code, user::create::create_user}, configuration::Settings, error::{api::user::UserRegistrationError, command::user::UserCreationError}, model::{confirmation_code_type::ConfirmationCodeType, email::Email}, utils::error::log_map};

#[derive(serde::Serialize)]
pub struct ResponseBody {
    confirmation_id: Uuid
}

#[derive(Deserialize, Debug)]
pub struct BodyData {
    email: Email,
    password: Secret<String>
}

/// User registration endpoint available @ POST /api/users
#[instrument(name = "Registering a user", skip(db_conn, config, email_client))]
pub async fn post_users(
    Json(form_body): Json<BodyData>,
    config: Data<Settings>,
    db_conn: Data<PgPool>,
    email_client: Data<EmailClient>
) -> actix_web::Result<HttpResponse, UserRegistrationError> {
    tracing::debug!("Acquiring the database connection.");
    let mut db_conn = db_conn.acquire()
        .await
        .map_err(|err| log_map(format!("Cannot acquire the connection to the database.\n{}", err), UserRegistrationError::UnexpectedError))?;

    tracing::debug!("Begining the transaction.");
    let mut transaction = db_conn.begin().await
        .map_err(|err| log_map(format!("Cannot start the transaction.\n{}", err), UserRegistrationError::UnexpectedError))?;
    
    tracing::info!("Creating the user.");
    let user_id = create_user(
        &mut *transaction,
        form_body.email.as_ref(),
        form_body.password.expose_secret()
    )
        .await
        .map_err(|err| {
            match err {
                UserCreationError::Argon2(err) => log_map(err, UserRegistrationError::UnexpectedError),
                UserCreationError::Sqlx(err) => match err.as_database_error() {
                    Some(err) => match err.kind() {
                        ErrorKind::UniqueViolation => UserRegistrationError::UserExists,
                        _ => log_map(err, UserRegistrationError::UnexpectedError)
                    },
                    None => log_map(err, UserRegistrationError::UnexpectedError)
                }
            }
        })?;
    
    tracing::info!("Creating the registration code.");
    let (confirmation_id, code) = create_confirmation_code(&mut *transaction, user_id, ConfirmationCodeType::Registration)
        .await
        // unexpected because no error should happen
        .map_err(|err| log_map(err, UserRegistrationError::UnexpectedError))?;

    tracing::info!("Sending the confirmation email.");
    let sender_email = config.email.sender.clone();
    let result = email_client.send_email(
        sender_email,
        form_body.email,
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
                    UserRegistrationError::UnexpectedError
                ))?;
            Ok(HttpResponse::Ok().json(ResponseBody {
                confirmation_id 
            }))
        },
        Err(err) => {
            log_map(
                format!("Email server has wrong configuration, couldn't send a confirmation email.\n{}", err),
                Err(UserRegistrationError::UnexpectedError)
            )
        }
    }
}