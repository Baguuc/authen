use std::{collections::HashMap, str::FromStr};
use authen::{clients::email::EmailClient, settings::Settings, model::email::Email};
use fake::{Fake, faker::{internet::en::SafeEmail, lorem::en::Sentence}};
use wiremock::{Mock, MockServer, ResponseTemplate, http::Method, matchers::{body_json, header, method, path}};

#[tokio::test]
pub async fn email_client_sends_requests() {
     // Arrange
    let mock_server = MockServer::start().await;
    
    let mut configuration = Settings::parse().unwrap();
    configuration.email.server.base_url = mock_server.uri();

    let email_server_configuration = configuration.clone().email.server;
    let email_client = EmailClient::new(email_server_configuration.clone()).unwrap();

    let from = Email::parse(SafeEmail().fake()).unwrap();
    let to = Email::parse(SafeEmail().fake()).unwrap();
    let subject: String = Sentence(1..10).fake();
    let text_body: String = Sentence(1..10).fake();
    let html_body: String = Sentence(1..10).fake();

    let email_send_endpoint_configuration = email_server_configuration.clone().send_endpoint;
    let req_route = email_send_endpoint_configuration.clone().route;
    let req_method = Method::from_str(email_send_endpoint_configuration.method.as_str()).unwrap();

    let mut builder = Mock::given(path(req_route))
        .and(method(req_method));

    for req_header in email_send_endpoint_configuration.headers {
        builder = builder.and(header(req_header.name, req_header.value));
    }

    let mut body_map = HashMap::new();
    body_map.insert(configuration.clone().email.server.send_endpoint.json_fields.from, from.as_ref().to_string());
    body_map.insert(configuration.clone().email.server.send_endpoint.json_fields.to, to.as_ref().to_string());
    body_map.insert(configuration.clone().email.server.send_endpoint.json_fields.subject, subject.clone());
    body_map.insert(configuration.clone().email.server.send_endpoint.json_fields.text_body, text_body.clone());
    body_map.insert(configuration.clone().email.server.send_endpoint.json_fields.html_body, html_body.clone());
    
    builder.and(body_json(body_map))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Act
    let result = email_client.send_email(
        // if unwrap panics, we know that either validation or generation logic is wrong.
        from,
        to,
        subject,
        text_body,
        html_body
    ).await;

    // Assert
    assert!(result.is_ok());
    // Mock verifies asserts in destructor
}