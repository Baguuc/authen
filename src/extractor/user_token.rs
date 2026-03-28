use std::future::ready;

use crate::error::extractor::user_token::UserTokenExtractionError;

/// Extracts the user JWT token from HttpRequest with the extractor pattern.
/// Example:
pub struct UserTokenExtractor(String);

impl UserTokenExtractor {
    pub fn parse(s: String) -> Result<Self, String> {
        if !s.starts_with("Bearer ") {
            return Err(String::from("Wrong token type"));
        }
        let stripped_token = s.replace("Bearer ", "").to_string();
        
        Ok(Self(stripped_token))
    }
}

impl AsRef<str> for UserTokenExtractor {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl actix_web::FromRequest for UserTokenExtractor {
    type Error = UserTokenExtractionError;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let raw_token = match req.headers().get("authorization") {
            Some(token) => token,
            None => return ready(Err(UserTokenExtractionError::NotFound)),
        };

        let token = match raw_token.to_str() {
            Ok(token) => token.to_string(),
            Err(_) => return ready(Err(UserTokenExtractionError::Invalid)),
        };

        match Self::parse(token) {
            Ok(token) => ready(Ok(token)),
            Err(_) => ready(Err(UserTokenExtractionError::WrongType))
        }
    }
}