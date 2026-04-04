use std::fmt::{Debug, Display};

use authen::{settings::Settings, startup::Application, telemetry::{get_tracing_subscriber, init_tracing_subscriber}};
use tokio::task::JoinError;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // init telemetry
    let subscriber = get_tracing_subscriber("authen".into(), "info".into(), std::io::stdout);
    init_tracing_subscriber(subscriber);

    // parse config
    let configuration = Settings::parse().expect("Failed to read the configuration.");

    // configure the application
    let application = Application::configure(configuration.clone()).await?;
    // start the application
    let application_task = tokio::spawn(application.run());

    // this is in place in case that in future we might need to add a dedicated worker thread.
    tokio::select! {
        o = application_task => report_exit("API", o),
    };

    Ok(())
}

/// Report a unexpected task exit
fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{}' task failed to complete",
                task_name
            )
        }
    }
}
