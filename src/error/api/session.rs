use actix_web::http::StatusCode;


/// Enum modelling errors that can occur during registration confirmation.
#[derive(Debug, thiserror::Error)]
pub enum SessionCreationError {
    /// Confirmation with provided ID do not exists.
    #[error("USER_NOT_EXISTS")]
    UserNotExists,
    /// The password provided by the user is wrong.
    #[error("WRONG_PASSWORD")]
    WrongPassword,
    /// Unexpected error happened.
    #[error("UNEXPECTED_ERROR")]
    UnexpectedError,
}

impl actix_web::ResponseError for SessionCreationError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::UserNotExists => StatusCode::UNAUTHORIZED,
            Self::WrongPassword => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}