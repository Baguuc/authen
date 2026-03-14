use std::fmt::Display;

use actix_web::{HttpResponse, http::StatusCode, web::{Path, Json, Data}};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::{command::{activate_user, delete_registration_code}, error::{api::ConfirmationError, query::RegistrationCodeVerifyError}, model::confirmation_code::ConfirmationCode, query::{get_user_id::get_user_id_from_registration_id, verify_registration_code}, utils::error::log_map};

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

/// User registration confirmation endpoint available @ POST /api/confirmations/registration/{}
#[instrument(name = "Confirming a user registration.", skip(db_conn))]
pub async fn post_confirmations_registration(
    path_data: Path<PathData>,
    Json(body): Json<JsonData>,
    db_conn: Data<PgPool>,
) -> actix_web::Result<HttpResponse, ConfirmationError> {
    tracing::debug!("Acquiring the database connection.");
    let mut db_conn = db_conn.acquire()
        .await
        .map_err(|err| log_map(format!("Cannot acquire the connection to the database.\n{}", err), ConfirmationError::UnexpectedError))?;

    let code_id = path_data.confirmation_id;
    let code = body.code;

    tracing::info!("Verifying the registration code (code_id = {}, code = {}).", code_id, code.as_ref());
    match verify_registration_code(&mut db_conn, code_id, code).await {
        Ok(false) => return Err(ConfirmationError::InvalidCode),
        Err(RegistrationCodeVerifyError::Sqlx(err)) => return log_map(err, Err(ConfirmationError::UnexpectedError)),
        Err(RegistrationCodeVerifyError::NotExists) => return Err(ConfirmationError::ConfirmationNotExists),
        _ => ()
    };

    tracing::info!("Retrieving the user id.");
    let user_id = get_user_id_from_registration_id(&mut db_conn, code_id).await
        .map_err(|err| log_map(err, ConfirmationError::UnexpectedError))?;

    tracing::info!("Activating the user.");
    activate_user(&mut db_conn, user_id).await
        .map_err(|err| log_map(err, ConfirmationError::UnexpectedError))?;

    tracing::info!("Deleting the confirmation code.");
    delete_registration_code(&mut db_conn, code_id)
        .await
        .map_err(|err| log_map(err, ConfirmationError::UnexpectedError))?;

    Ok(HttpResponse::Ok().finish())
}