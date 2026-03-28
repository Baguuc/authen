use actix_web::{HttpResponse, body::BoxBody, http::StatusCode};
use serde::Serialize;

/// Enum modelling errors that can occur during user registration.
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum UserRegistrationError {
    /// User with this email already exists in the database.
    #[error("user_exists")]
    UserExists,
    /// Unexpected error happened.
    #[error("unexpected_error")]
    UnexpectedError,
}

impl actix_web::ResponseError for UserRegistrationError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::UserExists => StatusCode::CONFLICT,
            Self::UnexpectedError => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::new(self.status_code())
            // won't fail
            .set_body(BoxBody::new(serde_json::to_string(self).unwrap()))
    }
}