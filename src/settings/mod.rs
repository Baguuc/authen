pub mod application;
pub mod database;
pub mod email;
pub mod jwt;
pub mod argon2;

use ::argon2::Argon2;
use chrono::Duration;
use jsonwebtoken::{Header, Validation};
use sqlx::postgres::PgConnectOptions;
use crate::{consts::{DEFAULT_LOGIN_EMAIL_HTML_BODY, DEFAULT_LOGIN_EMAIL_SUBJECT, DEFAULT_LOGIN_EMAIL_TEXT_BODY, DEFAULT_REGISTRATION_EMAIL_HTML_BODY, DEFAULT_REGISTRATION_EMAIL_SUBJECT, DEFAULT_REGISTRATION_EMAIL_TEXT_BODY}, settings::{application::ApplicationSettings, argon2::ArgonSettings, database::DatabaseSettings, email::{ConfirmationEmailBody, ConfirmationEmailSettings, EmailSettings}, jwt::JwtSettings}};

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

    /// Get the configuration of registration confirmation email or its defaults if not configured.
    pub fn registration_confirmation_email(&self) -> ConfirmationEmailSettings {
        if let Some(email) = self.email.registration.clone() {
            email
        } else {
            ConfirmationEmailSettings {
                subject: String::from(DEFAULT_REGISTRATION_EMAIL_SUBJECT),
                text_body: ConfirmationEmailBody::parse(String::from(DEFAULT_REGISTRATION_EMAIL_TEXT_BODY)).unwrap(),
                html_body: ConfirmationEmailBody::parse(String::from(DEFAULT_REGISTRATION_EMAIL_HTML_BODY)).unwrap(),
            }
        }
    }

    /// Get the configuration of login confirmation email or its defaults if not configured.
    pub fn login_confirmation_email(&self) -> ConfirmationEmailSettings {
        if let Some(email) = self.email.login.clone() {
            email
        } else {
            ConfirmationEmailSettings {
                subject: String::from(DEFAULT_LOGIN_EMAIL_SUBJECT),
                text_body: ConfirmationEmailBody::parse(String::from(DEFAULT_LOGIN_EMAIL_TEXT_BODY)).unwrap(),
                html_body: ConfirmationEmailBody::parse(String::from(DEFAULT_LOGIN_EMAIL_HTML_BODY)).unwrap(),
            }
        }
    }
}