use std::{collections::HashMap, str::FromStr};

use reqwest::{Method, header::{HeaderMap, HeaderName, HeaderValue}};

use crate::{settings::email::{EmailSendEnpointJsonFieldsSettings, EmailServerSettings}, error::client::email::EmailClientConstructionError, model::email::Email};

pub struct EmailClient {
    http_client: reqwest::Client,
    url: String,
    method: Method,
    headers: HeaderMap,
    json_fields_map: EmailSendEnpointJsonFieldsSettings
}

impl EmailClient {
    /// Create a new email client instance from config settings
    pub fn new(config: EmailServerSettings) -> Result<Self, EmailClientConstructionError> {
        let http_client = reqwest::Client::new();
        let base_url = config.base_url.trim_end_matches("/");
        let endpoint_route = config.send_endpoint.route.trim_start_matches("/");
        let url = format!("{}/{}", base_url, endpoint_route);
        let method = Method::from_str(&config.send_endpoint.method)
            .map_err(|_| EmailClientConstructionError::InvalidMethod(String::from("Invalid method provided for the send endpoint of email client")))?;

        let mut headers = HeaderMap::new();
        for header in config.send_endpoint.headers {
            let header_name = HeaderName::from_str(header.name.as_str())
                .map_err(|_| EmailClientConstructionError::InvalidHeaderValue(format!("Invalid email server's send endpoint header name:\n{}: {}", header.name, header.value)))?;
            let header_value = HeaderValue::from_str(header.value.as_str())
                .map_err(|_| EmailClientConstructionError::InvalidHeaderValue(format!("Invalid email server's send endpoint header value:\n{}: {}", header.name, header.value)))?;

            headers.insert(header_name, header_value);
        }

        Ok(Self {
            http_client,
            url,
            method,
            headers,
            json_fields_map: config.send_endpoint.json_fields
        })
    }

    /// Send an email from created EmailClient instance.
    pub async fn send_email(&self, from: Email, to: Email, subject: String, text_body: String, html_body: String) -> Result<(), reqwest::Error> {
        let mut body_map = HashMap::new();
        body_map.insert(self.json_fields_map.from.clone(), from.as_ref().to_string());
        body_map.insert(self.json_fields_map.to.clone(), to.as_ref().to_string());
        body_map.insert(self.json_fields_map.subject.clone(), subject);
        body_map.insert(self.json_fields_map.text_body.clone(), text_body);
        body_map.insert(self.json_fields_map.html_body.clone(), html_body);

        self.http_client
            // Use the returned application address
            .request(self.method.clone(), &self.url)
            .headers(self.headers.clone())
            .json(&serde_json::json!(body_map))
            .send()
            .await
            .map(|_| ())
    }
}