use actix_web::{HttpResponse, web::{Path, Json, Data}};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::{command::{confirmation_code::delete::delete_confirmation_code, user::activate::activate_user}, settings::Settings, error::{api::confirmation_code::ConfirmationError, query::confirmation_code::ConfirmationCodeVerificationError}, model::{confirmation_code::ConfirmationCode, confirmation_code_type::ConfirmationCodeType}, query::confirmation_code::{get_user_id::get_user_id_from_registration_code, verify::verify_confirmation_code}, utils::error::log_map};

/// Helper struct to deserialize data from request's path.
#[derive(Deserialize, Debug)]
pub struct PathData {
    confirmation_id: Uuid
}

/// Helper struct to deserialize data from request's json body.
#[derive(Deserialize, Debug)]
pub struct JsonData {
    code: ConfirmationCode
}

/// User registration confirmation endpoint available
#[instrument(name = "Confirming a user registration.", skip(db_conn, config))]
pub async fn post_confirmations_registration(
    path_data: Path<PathData>,
    Json(body): Json<JsonData>,
    db_conn: Data<PgPool>,
    config: Data<Settings>
) -> actix_web::Result<HttpResponse, ConfirmationError> {
    let code_id = path_data.confirmation_id;
    let code = body.code;
    let argon2_instance = config.argon2_instance();
    
    
    // 1. Begin a transaction
    tracing::info!("Begining a transaction");
    let mut transaction = db_conn.begin()
        .await
        .map_err(|err| log_map(format!("Cannot begin the transaction.\n{}", err), ConfirmationError::UnexpectedError))?;


    // 2. Verify the confirmation code with the one in the database.
    match verify_confirmation_code(&mut *transaction, argon2_instance, code_id, code, ConfirmationCodeType::Registration).await {
        Ok(false) => return Err(ConfirmationError::WrongCode),
        Err(ConfirmationCodeVerificationError::Sqlx(err)) => return log_map(err, Err(ConfirmationError::UnexpectedError)),
        Err(ConfirmationCodeVerificationError::NotExists) => return Err(ConfirmationError::ConfirmationNotExists),
        _ => ()
    };


    // 3. Retrieve user's id from confirmation code
    let user_id = get_user_id_from_registration_code(&mut *transaction, code_id, ConfirmationCodeType::Registration).await
        .map_err(|err| log_map(err, ConfirmationError::UnexpectedError))?;

    
    // 4. Active the user
    activate_user(&mut *transaction, user_id).await
        .map_err(|err| log_map(err, ConfirmationError::UnexpectedError))?;


    // 5. Delete the current (already used) confirmation code
    delete_confirmation_code(&mut *transaction, code_id, ConfirmationCodeType::Registration)
        .await
        .map_err(|err| log_map(err, ConfirmationError::UnexpectedError))?;


    // 6. Commit the transaction
    tracing::info!("Commiting the transaction");
    transaction.commit()
        .await
        .map_err(|err| log_map(format!("Cannot commit the transaction: {}", err), ConfirmationError::UnexpectedError))?;

    
    Ok(HttpResponse::Ok().finish())
}