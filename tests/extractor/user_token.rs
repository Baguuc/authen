use authen::extractor::user_token::UserTokenExtractor;
use fake::{Fake, faker::lorem::en::Word};

#[tokio::test]
async fn user_token_extractor_passes_bearer_type() {
    let auth_type = String::from("Bearer");
    let token: String = Word().fake();

    let auth_header_value = format!("{} {}", auth_type, token);
    let result = UserTokenExtractor::parse(auth_header_value);

    assert!(result.is_ok())
}

#[tokio::test]
async fn user_token_extractor_rejects_type_outside_bearer() {
    let mut auth_type: String = Word().fake();
    let token: String = Word().fake();

    // reject the valid authorization type
    if auth_type.to_lowercase() == "bearer" {
        auth_type = Word().fake();
    }

    let auth_header_value = format!("{} {}", auth_type, token);
    let result = UserTokenExtractor::parse(auth_header_value);

    assert!(result.is_err())
}