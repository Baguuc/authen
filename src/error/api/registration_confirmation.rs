use actix_web::http::StatusCode;


/// Enum modelling errors that can occur during registration confirmation.
#[derive(Debug, thiserror::Error)]
pub enum ConfirmationError {
    /// Confirmation with provided ID do not exists.
    #[error("CONFIRMATION_NOT_EXISTS")]
    ConfirmationNotExists,
    /// Invalid code
    #[error("INVALID_CODE")]
    WrongCode,
    /// Unexpected error happened.
    #[error("UNEXPECTED_ERROR")]
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
}