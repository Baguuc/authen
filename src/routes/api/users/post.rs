use actix_web::{HttpResponse, web::{Data, Json}};
use secrecy::SecretString;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::{clients::email::EmailClient, command::{confirmation_code::create::create_confirmation_code, user::create::create_user}, settings::Settings, error::{api::user::UserRegistrationError, command::user::UserCreationError}, model::{confirmation_code_type::ConfirmationCodeType, email::Email}, utils::error::log_map};

#[derive(serde::Serialize)]
pub struct ResponseBody {
    confirmation_id: Uuid
}

#[derive(Deserialize, Debug)]
pub struct BodyData {
    email: Email,
    password: SecretString
}

/// User registration endpoint
#[instrument(name = "Creating a user.", skip(db_conn, config, email_client))]
pub async fn post_users(
    Json(body): Json<BodyData>,
    config: Data<Settings>,
    db_conn: Data<PgPool>,
    email_client: Data<EmailClient>
) -> actix_web::Result<HttpResponse, UserRegistrationError> {
    let argon2_instance = config.argon2_instance();


    // 1. Begin a transaction
    tracing::debug!("Begining the transaction.");
    let mut transaction = db_conn.begin().await
        .map_err(|err| log_map(format!("Cannot start the transaction.\n{}", err), UserRegistrationError::UnexpectedError))?;

    
    // 2. Insert a user into the database.
    let user_id = create_user(
        &mut *transaction,
        &argon2_instance,
        body.email.as_ref(),
        &body.password
    )
        .await
        .map_err(|err| {
            match err {
                UserCreationError::UserExists => return UserRegistrationError::UserExists,
                UserCreationError::Argon2(err) => log_map(err, UserRegistrationError::UnexpectedError),
                UserCreationError::Sqlx(err) => log_map(err, UserRegistrationError::UnexpectedError)
            }
        })?;
    
    
    // 3. Generate a confirmation code.
    let (confirmation_id, code) = create_confirmation_code(&mut *transaction, &argon2_instance, user_id, ConfirmationCodeType::Registration)
        .await
        // unexpected because no error should happen
        .map_err(|err| log_map(err, UserRegistrationError::UnexpectedError))?;

    
    // 4. Send the confirmation code email
    tracing::info!("Sending the confirmation code.");
    let email_config = config.registration_confirmation_email();
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

    
    // 5. Match the result
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