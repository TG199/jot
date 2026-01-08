use jot::configuration::get_configuration;
use jot::issue_delivery_worker::run_worker_until_stopped;
use jot::startup::Application;
use jot::telemetry::{get_subscriber, init_subscriber};
use std::fmt::{Debug, Display};
use tokio::task::JoinError;

async fn main() -> anyhow::Result<()> {
    let subscriber = get_subcriber("jot".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");
    let application = Application::build(configuration.clone()).await?;

    Ok(())
}
