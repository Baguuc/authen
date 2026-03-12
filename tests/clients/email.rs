use std::str::FromStr;
use authen::{clients::email::EmailClient, configuration::Settings, model::email::Email};
use fake::{Fake, faker::{internet::en::SafeEmail, lorem::en::Sentence}};
use wiremock::{Mock, MockServer, ResponseTemplate, http::Method, matchers::{header, method, path}};

#[tokio::test]
pub async fn email_client_sends_requests() {
     // Arrange
    let mock_server = MockServer::start().await;
    let mut configuration = Settings::parse().unwrap();
    configuration.email.server.base_url = mock_server.uri();
    let email_server_configuration = configuration.email.server;
    let email_send_endpoint_configuration = email_server_configuration.clone().send_endpoint;
    let email_client = EmailClient::new(email_server_configuration).unwrap();

    let req_route = email_send_endpoint_configuration.route;
    let req_method = Method::from_str(email_send_endpoint_configuration.method.as_str()).unwrap();

    let mut builder = Mock::given(path(req_route))
        .and(method(req_method));

    for req_header in email_send_endpoint_configuration.headers {
        builder = builder.and(header(req_header.name, req_header.value));
    }
    
    builder.respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Act
    let result = email_client.send_email(
        // if unwrap panics, we know that either validation or generation logic is wrong.
        Email::parse(SafeEmail().fake()).unwrap(),
        Email::parse(SafeEmail().fake()).unwrap(),
        Sentence(1..10).fake(),
        Sentence(1..10).fake(),
        Sentence(1..10).fake()
    ).await;

    // Assert
    assert!(result.is_ok());
    // Mock verifies asserts in destructor
}