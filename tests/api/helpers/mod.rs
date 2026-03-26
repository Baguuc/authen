pub mod mock;
pub mod database;
pub mod app;

use authen::{settings::Settings, telemetry::{get_tracing_subscriber, init_tracing_subscriber}};
use wiremock::{Mock, MockServer};
use std::sync::LazyLock;

use crate::helpers::{app::TestApp, mock::get_mock_email_api, app::spawn_app};

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    let subscriber = get_tracing_subscriber(subscriber_name, default_filter_level, std::io::stdout);
    init_tracing_subscriber(subscriber);
});

/// init the system before test
pub async fn init() -> (TestApp, MockServer, reqwest::Client, Settings, Mock) {
    let mock_server = MockServer::start().await;
    let app = spawn_app(Some(mock_server.uri())).await;
    let http_client = reqwest::Client::new();
    let config = Settings::parse().unwrap();
    
    let mock = get_mock_email_api(&config, 1).await;

    (app, mock_server, http_client, config, mock)
}