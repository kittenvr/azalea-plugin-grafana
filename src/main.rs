use prometheus::{Encoder, TextEncoder, register_int_gauge};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

mod config;
mod metrics;
mod plugin;
mod logging;

#[tokio::main]
async fn main() {
    // Load configuration
    let config = config::Config::load_config().expect("Failed to load configuration");

    // Initialize Prometheus metrics
    let player_count = register_int_gauge!("minecraft_player_count", "Number of players online").unwrap();
    let tps = register_int_gauge!("minecraft_tps", "Server TPS").unwrap();
    let latency = register_int_gauge!("minecraft_latency", "Bot latency").unwrap();

    let metrics = Arc::new(Mutex::new(metrics::Metrics {
        player_count,
        tps,
        latency,
    }));

    // Initialize Azalea bot
    let bot = azalea::AzaleaBot::new(config.bot_token.clone(), config.server_address.clone())
        .await
        .expect("Failed to initialize Azalea bot");

    // Load plugin
    let plugin = plugin::Plugin::new(metrics.clone(), config.clone());
    bot.load_plugin(plugin).await.expect("Failed to load plugin");

    // Set up Prometheus metrics endpoint
    let metrics_clone = metrics.clone();
    let metrics_route = warp::path("metrics")
        .map(move || {
            let encoder = TextEncoder::new();
            let metric_families = prometheus::gather();
            let mut buffer = vec![];
            encoder.encode(&metric_families, &mut buffer).unwrap();
            String::from_utf8(buffer).unwrap()
        });

    // Set up Loki logging endpoint
    let client = Client::new();
    let loki_route = warp::path("loki")
        .and(warp::body::json())
        .map(move |log: logging::Log| {
            let client = client.clone();
            let loki_endpoint = config.loki_endpoint.clone();
            tokio::spawn(async move {
                logging::send_log_to_loki(&client, &loki_endpoint, log).await;
            });
            warp::reply::json(&"Log received")
        });

    // Start the server
    let routes = metrics_route.or(loki_route);
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
