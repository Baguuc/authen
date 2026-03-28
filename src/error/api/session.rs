use actix_web::{HttpResponse, body::BoxBody, http::StatusCode};
use serde::Serialize;


/// Enum modelling errors that can occur during session creation.
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum SessionCreationError {
    /// Confirmation with provided ID do not exists.
    #[error("user_not_exists")]
    UserNotExists,
    /// The user is inactive.
    #[error("user_inactive")]
    UserNotActive,
    /// The password provided by the user is wrong.
    #[error("wrong_password")]
    WrongPassword,
    /// Unexpected error happened.
    #[error("unexpected_error")]
    UnexpectedError,
}

impl actix_web::ResponseError for SessionCreationError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::UserNotExists => StatusCode::UNAUTHORIZED,
            Self::UserNotActive => StatusCode::FORBIDDEN,
            Self::WrongPassword => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::new(self.status_code())
            // won't fail
            .set_body(BoxBody::new(serde_json::to_string(self).unwrap()))
    }
}

/// Enum modelling errors that can occur during getting session info.
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum SessionGetInfoError {
    /// No token
    #[error("missing_token")]
    MissingToken,
    /// The user is inactive.
    #[error("INVALID_TOKEN")]
    InvalidToken,
    /// Unexpected error happened.
    #[error("UNEXPECTED_ERROR")]
    UnexpectedError
}

impl actix_web::ResponseError for SessionGetInfoError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::MissingToken | Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::new(self.status_code())
            // won't fail
            .set_body(BoxBody::new(serde_json::to_string(self).unwrap()))
    }
}

/// Enum modelling errors that can occur during updating session user's password.
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum SessionUserUpdatePasswordError {
    #[error("invalid_token")]
    InvalidToken,
    #[error("invalid_password")]
    InvalidPassword,
    #[error("unexpected_error")]
    UnexpectedError
}

impl actix_web::ResponseError for SessionUserUpdatePasswordError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::InvalidPassword => StatusCode::FORBIDDEN,
            Self::UnexpectedError => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::new(self.status_code())
            // won't fail
            .set_body(BoxBody::new(serde_json::to_string(self).unwrap()))
    }
}