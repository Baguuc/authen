use actix_web::http::StatusCode;

/// Enum modelling errors that can occur during user registration.
#[derive(Debug, thiserror::Error)]
pub enum UserRegistrationError {
    /// User with this email already exists in the database.
    #[error("USER_EXISTS")]
    UserExists,
    /// Unexpected error happened.
    #[error("UNEXPECTED_ERROR")]
    UnexpectedError,
}

impl actix_web::ResponseError for UserRegistrationError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::UserExists => StatusCode::CONFLICT,
            Self::UnexpectedError => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
