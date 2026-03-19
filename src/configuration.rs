use argon2::{Algorithm, Argon2, Params, Version};
use chrono::Duration;
use jsonwebtoken::{Header, Validation};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use crate::model::email::Email;

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email: EmailSettings,
    pub jwt: JwtSettings,
    pub argon2: Option<ArgonSettings>
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

    /// Shorthand for calling the DatabaseSettings::connect_options from top-level of config
    pub fn connect_options(&self) -> PgConnectOptions {
        self.database.connect_options()
    }

    /// Construct a argon2 instance from settings if specified, return Argon2::default() if None.
    pub fn argon2_instance(&self) -> Argon2 {
        if let Some(settings) = self.argon2.clone() {
            let algorithm = settings.algorithm.clone().into();
            let version = settings.version.clone().into();
            let parameters = settings.parameters();

            Argon2::new(algorithm, version, parameters)
        } else {
            Argon2::default()
        }
    }

    /// Construct a JWT header from jwt settings
    pub fn jwt_header(&self) -> Header {
        Header::new(self.jwt.algorithm)
    }

    /// Construct a JWT validation from jwt settings
    pub fn jwt_validation(&self) -> Validation {
        Validation::new(self.jwt.algorithm)
    }

    /// Get a chrono::Duration from configuration's value of expires_in
    pub fn jwt_expires_in(&self) -> Duration {
        Duration::minutes(self.jwt.expires_in)
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
    pub server: EmailServerSettings,
    pub sender: Email
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
pub struct JwtSettings {
    pub algorithm: jsonwebtoken::Algorithm,
    pub hashing_key: String,
    // value provided in minutes
    pub expires_in: i64
}

#[derive(serde::Deserialize, Clone)]
pub struct ArgonSettings {
    pub algorithm: ArgonAlgorithm,
    pub version: ArgonVersion,
    pub parameters: Option<ArgonParameterSettings>
}

impl ArgonSettings {
    /// Get Argon2 params from the parameter settings, or default when not specified
    pub fn parameters(&self) -> Params {
        if let Some(settings) = self.parameters.clone() {
            settings.into()
        } else {
            Params::default()
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ArgonAlgorithm {
    Argon2d,
    Argon2i,
    Argon2id
}

impl Into<Algorithm> for ArgonAlgorithm {
    fn into(self) -> Algorithm {
        match self {
            Self::Argon2d => Algorithm::Argon2d,
            Self::Argon2i => Algorithm::Argon2i,
            Self::Argon2id => Algorithm::Argon2id
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ArgonVersion {
    V0x13,
    V0x10
}

impl Into<Version> for ArgonVersion {
    fn into(self) -> Version {
        match self {
            Self::V0x10 => Version::V0x10,
            Self::V0x13 => Version::V0x13
        }
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct ArgonParameterSettings {
    pub memory: u32,
    pub time: u32,
    pub parallel: u32,
    pub output_len: Option<usize>
}

impl Into<Params> for ArgonParameterSettings {
    fn into(self) -> Params {
        Params::new(
            self.memory,
            self.time,
            self.parallel,
            self.output_len
        )
        .expect("Couldn't construct the Argon2 parameters.")
    }
}