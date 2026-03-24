use actix_web::http::StatusCode;


/// Enum modelling errors that can occur during session creation.
#[derive(Debug, thiserror::Error)]
pub enum SessionCreationError {
    /// Confirmation with provided ID do not exists.
    #[error("USER_NOT_EXISTS")]
    UserNotExists,
    /// The user is inactive.
    #[error("USER_INACTIVE")]
    UserNotActive,
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
            Self::UserNotActive => StatusCode::FORBIDDEN,
            Self::WrongPassword => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// Enum modelling errors that can occur during getting session info.
#[derive(Debug, thiserror::Error)]
pub enum SessionGetInfoError {
    /// No token
    #[error("NO_TOKEN")]
    NoToken,
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
            Self::NoToken | Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}