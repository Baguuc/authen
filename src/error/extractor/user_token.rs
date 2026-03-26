/// Represents one of the errors that can happen during extraction of the JWT token from HTTP request's headers.
#[derive(thiserror::Error, Debug)]
pub enum UserTokenExtractionError {
    /// This means that the authorization header was not found in the request.
    #[error("authorization_not_found")]
    NotFound,
    /// This means that the authorization header was found, but it was malformed.
    #[error("invalid_authorization")]
    Invalid,
    /// This means that the authorization header was not of type `Bearer`.
    #[error("wrong_authorization_type")]
    WrongType,
}

impl actix_web::error::ResponseError for UserTokenExtractionError {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::BadRequest().json(serde_json::json!({ "code": self.to_string() }))
    }
}