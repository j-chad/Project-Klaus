use tracing_subscriber::prelude::*;

use crate::config;
use anyhow::Context;
use tracing_subscriber::{registry, EnvFilter};

pub fn init_tracing(config: &config::LoggingSettings) -> anyhow::Result<()> {
    let filter = EnvFilter::builder()
        .parse(&config.level)
        .context("Failed to parse log level")?;

    let subscriber = registry().with(filter);

    match config.format {
        config::LogFormat::Json => {
            let json_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_writer(std::io::stdout);
            subscriber.with(json_layer).init();
        }
        config::LogFormat::Pretty => {
            let pretty_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);
            subscriber.with(pretty_layer).init();
        }
    }

    Ok(())
}