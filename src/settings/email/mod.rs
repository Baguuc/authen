use crate::model::email::Email;

#[derive(serde::Deserialize, Clone)]
pub struct EmailSettings {
    pub server: EmailServerSettings,
    pub sender: Email,
    pub registration: RegistrationEmailSettings,
    pub login: LoginEmailSettings    
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
pub struct RegistrationEmailSettings {
    pub subject: String,
    pub text_body: String,
    pub html_body: String
}

#[derive(serde::Deserialize, Clone)]
pub struct LoginEmailSettings {
    pub subject: String,
    pub text_body: String,
    pub html_body: String
}