use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email: EmailSettings
}

impl Settings {
    /// Parse configuration from files in configuration/ directory and environment variables.
    pub fn parse() -> Result<Settings, config::ConfigError> {
        let base_path = std::env::current_dir().expect("Failed to determine the current directory");
        let configuration_directory = base_path.join("configuration");

        let settings = config::Config::builder()
            .add_source(config::File::from(
                configuration_directory.join("config.yaml"),
            ))
            // Add in settings from environment variables (with a prefix of APP and '__' as separator)
            // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port`
            .add_source(
                config::Environment::with_prefix("APP")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;

        settings.try_deserialize::<Settings>()
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub base_url: String
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    /// Get the sqlx's PgConnectOptions for database.
    pub fn connect_options(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
            .database(&self.database_name)
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct EmailSettings {
    pub server: EmailServerSettings
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