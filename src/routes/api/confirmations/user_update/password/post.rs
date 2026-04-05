use actix_web::{HttpResponse, web::{Path, Json, Data}};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::{command::{confirmation_code::delete::delete_confirmation_code, update_data::delete::delete_update_data, user::update_password_hash::update_password_hash}, error::{api::confirmation_code::ConfirmationError, query::confirmation_code::ConfirmationCodeVerificationError}, model::{confirmation_code::ConfirmationCode, confirmation_code_type::ConfirmationCodeType, hashed_string::HashedString}, query::{confirmation_code::{get_user_id::get_user_id_from_registration_code, verify::verify_confirmation_code}, update_data::get_update_data::get_update_data}, settings::Settings, utils::error::log_map};

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

/// Helper struct to serialize the response body
#[derive(Serialize)]
pub struct ResponseBody {
    token: String
}

/// User password update confirmation endpoint
#[instrument(name = "Confirming a user password update.", skip(db_conn, config))]
pub async fn post_confirmations_user_update_password(
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


    // 2. Verify the confirmation code
    match verify_confirmation_code(&mut *transaction, argon2_instance, code_id, code, ConfirmationCodeType::UpdateUserPassword).await {
        Ok(false) => return Err(ConfirmationError::WrongCode),
        Err(ConfirmationCodeVerificationError::Sqlx(err)) => return log_map(err, Err(ConfirmationError::UnexpectedError)),
        Err(ConfirmationCodeVerificationError::NotExists) => return Err(ConfirmationError::ConfirmationNotExists),
        _ => ()
    };


    // 3. Retrieve the user id from confirmation code.
    let user_id = get_user_id_from_registration_code(&mut *transaction, code_id, ConfirmationCodeType::UpdateUserPassword).await
        .map_err(|err| log_map(err, ConfirmationError::UnexpectedError))?;


    // 4. Retrieve the data to update from confirmation code.
    let data = get_update_data(&mut *transaction, &code_id).await
        .map_err(|err| log_map(err, ConfirmationError::UnexpectedError))?;

    let new_password_hash = data.get("password_hash")
        .ok_or(log_map(String::from("Update data lacks password_hash."), ConfirmationError::UnexpectedError))?
        .as_str()
        .ok_or(log_map(String::from("Update data's password_hash field is not a string."), ConfirmationError::UnexpectedError))?
        .to_string();


    // 5. Update the user's password hash in the database.
    update_password_hash(&mut *transaction, &user_id, &HashedString(new_password_hash)).await
        .map_err(|err| log_map(err, ConfirmationError::UnexpectedError))?;


    // 6. Delete the update data from the database
    delete_update_data(&mut *transaction, &code_id)
        .await
        .map_err(|err| log_map(err, ConfirmationError::UnexpectedError))?;

    
    // 7. Delete the confirmation code from the database.
    delete_confirmation_code(&mut *transaction, code_id, ConfirmationCodeType::UpdateUserPassword)
        .await
        .map_err(|err| log_map(err, ConfirmationError::UnexpectedError))?;

    // 8. Commit the transaction.
    tracing::info!("Commiting the transaction");
    transaction.commit()
        .await
        .map_err(|err| log_map(format!("Cannot commit the transaction: {}", err), ConfirmationError::UnexpectedError))?;

    
    Ok(HttpResponse::Ok().finish())
}