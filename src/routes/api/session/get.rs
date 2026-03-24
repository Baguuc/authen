use actix_web::{HttpRequest, HttpResponse, web::{Data, Query}};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::{auth::jwt::deserialize_claims_from_user_token, error::{api::session::SessionGetInfoError, query::user::RetrieveUserError}, model::{comma_separated_vec::CommaSeparatedVec}, query::user::{get_user_id::get_user_id_from_email, is_active::is_user_active, retrieve::retrieve_user, verify_password::verify_user_password}, settings::Settings, utils::error::log_map};

#[derive(Deserialize, Debug)]
pub struct QueryBody {
    fields: CommaSeparatedVec
}

#[derive(Serialize, Debug)]
pub struct ResponseBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password_hash: Option<String>
}

/// Session info retrieval endpoint available @ GET /api/session
#[instrument(name = "Retrieving session info", skip(db_conn, config))]
pub async fn get_session(
    Query(query): Query<QueryBody>,
    db_conn: Data<PgPool>,
    config: Data<Settings>,
    req: HttpRequest
) -> actix_web::Result<HttpResponse, SessionGetInfoError> {
    tracing::debug!("Acquiring the database connection.");
    let mut db_conn = db_conn.acquire()
        .await
        .map_err(|err| log_map(format!("Cannot acquire the connection to the database.\n{}", err), SessionGetInfoError::UnexpectedError))?;

    tracing::info!("Getting the user token.");
    let authorization_header = match req.headers().get("authorization") {
        Some(h) => h.to_str()
            .map_err(|err| log_map(err, SessionGetInfoError::UnexpectedError))?
            .to_string(),
        None => return Err(SessionGetInfoError::NoToken)
    };

    tracing::info!("Got authorization header: '{}'", authorization_header);

    if !authorization_header.starts_with("Bearer ") {
        return Err(SessionGetInfoError::InvalidToken);
    }

    let token = &authorization_header.strip_prefix("Bearer ").unwrap();

    tracing::info!("Decoding user token.");
    let claims = match deserialize_claims_from_user_token(&
        config.jwt.hashing_key,
        &config.jwt_validation(),
        token
    ) {
        Ok(claims) => claims,
        Err(_) => return Err(SessionGetInfoError::InvalidToken)
    };
    let user_id = claims.sub;

    tracing::info!("Fetching user data.");
    let user = match retrieve_user(&mut *db_conn, user_id).await {
        Ok(user) => user,
        // when the user from the token do not exist it implies that the token is invalid
        Err(RetrieveUserError::NotExists) => return Err(SessionGetInfoError::InvalidToken),
        Err(RetrieveUserError::Sqlx(err)) => return log_map(err, Err(SessionGetInfoError::UnexpectedError))
    };

    let fields = query.fields.as_ref();

    let body = ResponseBody {
        id: fields.contains(&String::from("id")).then_some(user.id),
        email: fields.contains(&String::from("email")).then_some(user.email),
        password_hash: fields.contains(&String::from("password_hash")).then_some(user.password_hash),
    };

    tracing::debug!("Responding with {:?}", body);

    Ok(HttpResponse::Ok().json(body))
}