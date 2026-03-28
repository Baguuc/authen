use actix_web::{HttpResponse, web::{Path, Json, Data}};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::{command::confirmation_code::delete::delete_confirmation_code, settings::Settings, error::{api::confirmation_code::ConfirmationError, query::confirmation_code::ConfirmationCodeVerificationError}, model::{confirmation_code::ConfirmationCode, confirmation_code_type::ConfirmationCodeType}, query::confirmation_code::{get_user_id::get_user_id_from_registration_code, verify::verify_confirmation_code}, utils::{error::log_map}, auth::jwt::generate_user_token};

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

/// User login confirmation endpoint available @ POST /api/confirmations/login/{}
#[instrument(name = "Confirming a user login.", skip(db_conn, config))]
pub async fn post_confirmations_login(
    path_data: Path<PathData>,
    Json(body): Json<JsonData>,
    db_conn: Data<PgPool>,
    config: Data<Settings>
) -> actix_web::Result<HttpResponse, ConfirmationError> {
    tracing::debug!("Acquiring the database connection.");
    let mut db_conn = db_conn.acquire()
        .await
        .map_err(|err| log_map(format!("Cannot acquire the connection to the database.\n{}", err), ConfirmationError::UnexpectedError))?;

    let code_id = path_data.confirmation_id;
    let code = body.code;
    let argon2_instance = config.argon2_instance();
    let jwt_header = config.jwt_header();

    tracing::info!("Verifying the registration code (code_id = {}, code = {}).", code_id, code.as_ref());
    match verify_confirmation_code(&mut *db_conn, argon2_instance, code_id, code, ConfirmationCodeType::Login).await {
        Ok(false) => return Err(ConfirmationError::WrongCode),
        Err(ConfirmationCodeVerificationError::Sqlx(err)) => return log_map(err, Err(ConfirmationError::UnexpectedError)),
        Err(ConfirmationCodeVerificationError::NotExists) => return Err(ConfirmationError::ConfirmationNotExists),
        _ => ()
    };

    tracing::info!("Retrieving the user id.");
    let user_id = get_user_id_from_registration_code(&mut *db_conn, code_id, ConfirmationCodeType::Login).await
        .map_err(|err| log_map(err, ConfirmationError::UnexpectedError))?;

    tracing::info!("Generating the token.");
    let token = generate_user_token(&config.jwt.hashing_key, &jwt_header, config.jwt_expires_in(), user_id)
        .map_err(|err| log_map(format!("Cannot generate the user token, JWT error: {}", err), ConfirmationError::UnexpectedError))?;

    tracing::info!("Deleting the confirmation code.");
    delete_confirmation_code(&mut *db_conn, code_id, ConfirmationCodeType::Login)
        .await
        .map_err(|err| log_map(err, ConfirmationError::UnexpectedError))?;

    Ok(HttpResponse::Ok().json(ResponseBody {
        token
    }))
}