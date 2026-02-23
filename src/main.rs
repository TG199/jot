use jot::configuration::get_configuration;
use jot::startup::Application;
use jot::telemetry::{get_log_level_for_env, get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration");

    let log_level = get_log_level_for_env(&configuration.application.environment);
    let subscriber = get_subscriber("jot".into(), log_level, std::io::stdout);
    init_subscriber(subscriber);

    tracing::info!(
        environment = %configuration.application.environment,
        "Starting Jot API on {}:{}",
        configuration.application.host,
        configuration.application.port,
    );

    let application = Application::build(configuration.clone()).await?;
    application.run_until_stopped().await?;

    Ok(())
}
