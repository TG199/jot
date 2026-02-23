use tokio::task::JoinHandle;
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> Box<dyn Subscriber + Sync + Send>
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    let is_json = std::env::var("APP_ENVIRONMENT")
        .map(|v| v == "production")
        .unwrap_or(false);

    if is_json {
        let formatting_layer = BunyanFormattingLayer::new(name, sink);
        Box::new(
            Registry::default()
                .with(env_filter)
                .with(JsonStorageLayer)
                .with(formatting_layer),
        )
    } else {
        let formatting_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_level(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .pretty();

        Box::new(Registry::default().with(env_filter).with(formatting_layer))
    }
}

pub fn init_subscriber(subscriber: impl Subscriber + Sync + Send) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber")
}

pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}

pub fn get_log_level_for_env(environment: &str) -> String {
    match environment {
        "production" => "info".to_string(),
        "local" => "debug,sqlx=debug".to_string(),
        _ => "info".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_log_level_is_info() {
        let level = get_log_level_for_env("production");
        assert_eq!(level, "info");
    }

    #[test]
    fn local_log_level_includes_debug() {
        let level = get_log_level_for_env("local");
        assert!(level.contains("debug"));
    }

    #[test]
    fn local_log_level_includes_sqlx() {
        let level = get_log_level_for_env("local");
        assert!(level.contains("sqlx"));
    }

    #[test]
    fn default_log_level_is_info() {
        let level = get_log_level_for_env("unknown");
        assert_eq!(level, "info");
    }
}
