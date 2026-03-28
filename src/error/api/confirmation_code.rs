use actix_web::{HttpResponse, body::BoxBody, http::StatusCode};
use serde::Serialize;


/// Enum modelling errors that can occur during registration confirmation.
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum ConfirmationError {
    /// Confirmation with provided ID do not exists.
    #[error("confirmation_not_exists")]
    ConfirmationNotExists,
    /// Invalid code
    #[error("wrong_code")]
    WrongCode,
    /// Unexpected error happened.
    #[error("unexpected_error")]
    UnexpectedError,
}

impl actix_web::ResponseError for ConfirmationError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::ConfirmationNotExists => StatusCode::NOT_FOUND,
            Self::WrongCode => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::new(self.status_code())
            // won't fail
            .set_body(BoxBody::new(serde_json::to_string(self).unwrap()))
    }
}