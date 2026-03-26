use actix_web::{ResponseError, http::StatusCode};

/// Represents one of the errors that can happen during extraction of the JWT token from HTTP request's headers.
#[derive(thiserror::Error, Debug)]
pub enum UserTokenExtractionError {
    /// This means that the authorization header was not found in the request.
    #[error("AUTHORIZATION_NOT_FOUND")]
    NotFound,
    /// This means that the authorization header was found, but it was malformed.
    #[error("INVALID_AUTHORIZATION_HEADER_VALUE")]
    Invalid,
    /// This means that the authorization header was not of type `Bearer`.
    #[error("WRONG_AUTHORIZATION_TYPE")]
    WrongType,
}

impl ResponseError for UserTokenExtractionError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        StatusCode::UNAUTHORIZED
    }
}