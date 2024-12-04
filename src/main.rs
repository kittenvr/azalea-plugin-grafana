mod config;
mod error;
mod logging;
mod metrics;
mod plugin;

use error::Result;
use prometheus::{Encoder, TextEncoder};
use std::net::SocketAddr;
use warp::Filter;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter("info")
            .init()
    ).expect("Failed to set tracing subscriber");

    // Load and validate configuration
    let config = config::Config::load()?;
    config.validate()?;

    // Initialize metrics
    let metrics = metrics::Metrics::new()?;

    // Initialize Azalea bot
    let bot = azalea::AzaleaBot::new(
        config.bot_token.clone(),
        config.server_address.clone(),
    ).await.map_err(|e| error::PluginError::Bot(e.to_string()))?;

    // Initialize and load plugin
    let plugin = plugin::Plugin::new(metrics.clone(), &config);
    bot.load_plugin(plugin)
        .await
        .map_err(|e| error::PluginError::Bot(e.to_string()))?;

    // Set up metrics endpoint
    let metrics_route = warp::path("metrics").map(|| {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer)
            .expect("Failed to encode metrics");
        String::from_utf8(buffer)
            .expect("Failed to convert metrics to string")
    });

    // Start the server
    let addr: SocketAddr = ([0, 0, 0, 0], config.metrics_port).into();
    tracing::info!("Starting metrics server on {}", addr);
    warp::serve(metrics_route).run(addr).await;

    Ok(())
}
