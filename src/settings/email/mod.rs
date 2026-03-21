use serde::{Deserialize, de::Error};
use crate::model::email::Email;

#[derive(serde::Deserialize, Clone)]
pub struct EmailSettings {
    pub server: EmailServerSettings,
    pub sender: Email,
    pub registration: Option<ConfirmationEmailSettings>,
    pub login: Option<ConfirmationEmailSettings>
}

#[derive(serde::Deserialize, Clone)]
pub struct EmailServerSettings {
    pub base_url: String,
    pub send_endpoint: EmailSendEndpointSettings
}

#[derive(serde::Deserialize, Clone)]
pub struct EmailSendEndpointSettings {
    pub method: String,
    pub route: String,
    pub headers: Vec<EmailSendEnpointHeaderSettings>,
    pub json_fields: EmailSendEnpointJsonFieldsSettings
}

#[derive(serde::Deserialize, Clone)]
pub struct EmailSendEnpointHeaderSettings {
    pub name: String,
    pub value: String
}

#[derive(serde::Deserialize, Clone)]
pub struct EmailSendEnpointJsonFieldsSettings {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub text_body: String,
    pub html_body: String
}

#[derive(serde::Deserialize, Clone)]
pub struct ConfirmationEmailSettings {
    pub subject: String,
    pub text_body: ConfirmationEmailBody,
    pub html_body: ConfirmationEmailBody
}

#[derive(Clone)]
pub struct ConfirmationEmailBody(String);

impl ConfirmationEmailBody {
    pub fn parse(s: String) -> Result<Self, String> {
        let placeholder_count = s.matches("%code%")
            .count();
        
        if placeholder_count == 0 {
            return Err(String::from("There is not"))
        }

        if placeholder_count > 1 {
            return Err(String::from("The code placeholder should appear exactly one time."))
        }

        Ok(Self(s))
    }
}

impl<'de> Deserialize<'de> for ConfirmationEmailBody {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
            let raw = String::deserialize(deserializer)?;
            Self::parse(raw).map_err(D::Error::custom)
    }
}

impl AsRef<str> for ConfirmationEmailBody {
    fn as_ref(&self) -> &str {
        &self.0
    }
}