use std::collections::HashMap;

use authen::settings::Settings;
use wiremock::{Mock, MockServer, Request, ResponseTemplate, matchers::{method, path}};

pub async fn get_mock_email_api(config: &Settings, expected_email_count: u64) -> Mock {
    Mock::given(method(config.email.server.send_endpoint.method.clone()))
        .and(path(config.email.server.send_endpoint.route.clone()))
        .respond_with(ResponseTemplate::new(200))
        .expect(expected_email_count)
}

pub async fn get_request_from_mock_server(mock_server: &MockServer, request_index: usize) -> Request {
    mock_server.received_requests()
        .await
        .expect("Mock email server haven't got the request.")
        .get(request_index)
        .unwrap()
        .clone()
}

pub async fn get_registration_confirmation_code_from_request(request: Request, config: Settings) -> String {
    let recieved_request_body: HashMap<String, String> = request.body_json()
        .unwrap();
    let text_body: &String = recieved_request_body.get("TextBody")
        .expect("No TextBody in the request.");
    
    let email_config = config.registration_confirmation_email();
    // split the body in two parts with the code placeholder in the middle
    let parts = email_config.text_body
        .as_ref()
        .split("%code%")
        .collect::<Vec<&str>>();

    // the validation ensures the code is appearing exactly one time
    let part1 = parts.get(0).unwrap();
    let part2 = parts.get(1).unwrap();

    let confirmation_code = text_body.replace(part1, "").replace(part2, "");

    confirmation_code
}

pub async fn get_login_confirmation_code_from_request(request: Request, config: Settings) -> String {
    let recieved_request_body: HashMap<String, String> = request.body_json()
        .unwrap();
    let text_body: &String = recieved_request_body.get("TextBody")
        .expect("No TextBody in the request.");
    
    let email_config = config.login_confirmation_email();
    // split the body in two parts with the code placeholder in the middle
    let parts = email_config.text_body
        .as_ref()
        .split("%code%")
        .collect::<Vec<&str>>();

    // the validation ensures the code is appearing exactly one time
    let part1 = parts.get(0).unwrap();
    let part2 = parts.get(1).unwrap();

    let confirmation_code = text_body.replace(part1, "").replace(part2, "");

    confirmation_code
}