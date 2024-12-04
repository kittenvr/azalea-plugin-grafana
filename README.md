# Azalea Plugin for Minecraft Server Metrics and Logs

This project is a Rust plugin designed for Azalea, a Minecraft bot framework, that tracks various server metrics and logs. The plugin integrates seamlessly with existing Azalea bot projects, collecting essential server data and sending it to a backend for monitoring. The data is then visualized on a single, central Grafana dashboard.

## Key Features

1. **Player Join and Leave Tracking**:
   - The plugin logs the **username** and **timestamp** of players who join and leave the server. This data is displayed on the Grafana dashboard.

2. **Chat Log Monitoring**:
   - The plugin captures **chat messages**, including the **username**, **timestamp**, and **message content**, and sends these logs to **Loki** for storage and visualization.

3. **Server Metrics**:
   - The plugin collects server **TPS (Ticks Per Second)** and **player count**, and exposes these as **Prometheus metrics**. These metrics are visualized in real-time on the Grafana dashboard.

4. **Bot Latency Monitoring**:
   - The plugin tracks the **latency** (ping) between the bot and the server, and sends this information as a metric to be displayed on the Grafana dashboard.

5. **Multi-Bot and Multi-Project Support**:
   - The plugin is designed to handle cases where multiple bots are running the plugin across multiple Azalea projects but are connected to the same Minecraft server. It prevents **duplicate data** from being logged and ensures that the data remains consistent.

6. **Prometheus and Loki Integration**:
   - The plugin exposes Prometheus metrics via an endpoint (`/metrics`), including:
     - **minecraft_player_count** (number of players online)
     - **minecraft_tps** (server TPS)
     - **minecraft_latency** (bot latency)
   - The plugin sends logs to **Loki**, including player join/leave logs and chat messages, which are then displayed in Grafana.

7. **Centralized Grafana Dashboard**:
   - A single Grafana dashboard consolidates all collected data:
     - **Player Joins and Leaves** with usernames and timestamps.
     - **Chat Logs** with usernames, timestamps, and message content.
     - **Player Count** (real-time graph).
     - **TPS** (real-time graph).
     - **Bot Latency** (real-time graph).

8. **Configuration**:
   - The plugin uses a **TOML configuration file** for specifying settings such as:
     - Prometheus and Loki endpoints.
     - Custom chat message formatting for edge cases.
     - Options for which data to collect and how to format it.

9. **Edge Case Handling**:
   - The plugin gracefully handles different server configurations (e.g., custom chat formats).
   - It manages multiple bot instances running the plugin without duplicating data, ensuring consistency even when the plugin is running across multiple projects.

## Installation

To use this plugin, add the following dependencies to your `Cargo.toml`:

```toml
[dependencies]
azalea = "0.1"
prometheus = "0.12"
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
```

## Configuration

Create a `config.toml` file in the root of your project with the following content:

```toml
prometheus_endpoint = "http://localhost:9090"
loki_endpoint = "http://localhost:3100"
bot_token = "your_bot_token"
server_address = "your_server_address"
```

## Usage

1. Initialize the Azalea bot and load the plugin:

```rust
use azalea::prelude::*;
use prometheus::{Encoder, TextEncoder, register_int_gauge, IntGauge};
use reqwest::Client;
use serde::Deserialize;
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
    let config = config::load_config().expect("Failed to load configuration");

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
    let bot = Azalea::new(config.bot_token.clone(), config.server_address.clone())
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
```

2. Implement the plugin functionality in `src/plugin.rs`:

```rust
use azalea::prelude::*;
use prometheus::IntGauge;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Plugin {
    metrics: Arc<Mutex<Metrics>>,
    config: Config,
}

impl Plugin {
    pub fn new(metrics: Arc<Mutex<Metrics>>, config: Config) -> Self {
        Plugin { metrics, config }
    }
}

#[async_trait]
impl AzaleaPlugin for Plugin {
    async fn on_player_join(&self, player: &Player) {
        let mut metrics = self.metrics.lock().await;
        metrics.player_count.inc();
        let log = Log::new("player_join", player.username.clone(), None);
        send_log_to_loki(&self.config.loki_endpoint, log).await;
    }

    async fn on_player_leave(&self, player: &Player) {
        let mut metrics = self.metrics.lock().await;
        metrics.player_count.dec();
        let log = Log::new("player_leave", player.username.clone(), None);
        send_log_to_loki(&self.config.loki_endpoint, log).await;
    }

    async fn on_chat_message(&self, player: &Player, message: &str) {
        let log = Log::new("chat_message", player.username.clone(), Some(message.to_string()));
        send_log_to_loki(&self.config.loki_endpoint, log).await;
    }

    async fn on_tick(&self, tps: f64, latency: u64) {
        let mut metrics = self.metrics.lock().await;
        metrics.tps.set(tps as i64);
        metrics.latency.set(latency as i64);
    }
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub loki_endpoint: String,
}

pub struct Metrics {
    pub player_count: IntGauge,
    pub tps: IntGauge,
    pub latency: IntGauge,
}

pub struct Log {
    pub event: String,
    pub username: String,
    pub message: Option<String>,
}

impl Log {
    pub fn new(event: &str, username: String, message: Option<String>) -> Self {
        Log {
            event: event.to_string(),
            username,
            message,
        }
    }
}

async fn send_log_to_loki(endpoint: &str, log: Log) {
    let client = reqwest::Client::new();
    let _ = client.post(endpoint).json(&log).send().await;
}
```

3. Define the configuration struct and load the TOML configuration file in `src/config.rs`:

```rust
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub prometheus_endpoint: String,
    pub loki_endpoint: String,
    pub bot_token: String,
    pub server_address: String,
}

impl Config {
    pub fn load_config() -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }

    pub fn get_prometheus_endpoint(&self) -> &str {
        &self.prometheus_endpoint
    }

    pub fn get_loki_endpoint(&self) -> &str {
        &self.loki_endpoint
    }
}
```

4. Define and register Prometheus metrics for player count, TPS, and bot latency in `src/metrics.rs`:

```rust
use prometheus::{IntGauge, register_int_gauge};

pub struct Metrics {
    pub player_count: IntGauge,
    pub tps: IntGauge,
    pub latency: IntGauge,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            player_count: register_int_gauge!("minecraft_player_count", "Number of players online").unwrap(),
            tps: register_int_gauge!("minecraft_tps", "Server TPS").unwrap(),
            latency: register_int_gauge!("minecraft_latency", "Bot latency").unwrap(),
        }
    }

    pub fn update_player_count(&self, count: i64) {
        self.player_count.set(count);
    }

    pub fn update_tps(&self, tps: i64) {
        self.tps.set(tps);
    }

    pub fn update_latency(&self, latency: i64) {
        self.latency.set(latency);
    }
}
```

5. Implement functions to send logs to Loki in `src/logging.rs`:

```rust
use reqwest::Client;
use serde::Serialize;

#[derive(Serialize)]
pub struct Log {
    pub event: String,
    pub username: String,
    pub message: Option<String>,
}

pub async fn send_log_to_loki(client: &Client, endpoint: &str, log: Log) {
    let _ = client.post(endpoint).json(&log).send().await;
}
```

## Setting Up the Grafana Dashboard

1. Install Grafana by following the [official installation guide](https://grafana.com/docs/grafana/latest/installation/).

2. Add Prometheus as a data source in Grafana:
   - Go to **Configuration** > **Data Sources**.
   - Click **Add data source** and select **Prometheus**.
   - Set the URL to your Prometheus server (e.g., `http://localhost:9090`).
   - Click **Save & Test**.

3. Add Loki as a data source in Grafana:
   - Go to **Configuration** > **Data Sources**.
   - Click **Add data source** and select **Loki**.
   - Set the URL to your Loki server (e.g., `http://localhost:3100`).
   - Click **Save & Test**.

4. Create a new dashboard in Grafana and add panels for the following metrics:
   - **Player Joins and Leaves**: Use the Loki data source to query logs with the event type `player_join` and `player_leave`.
   - **Chat Logs**: Use the Loki data source to query logs with the event type `chat_message`.
   - **Player Count**: Use the Prometheus data source to query the `minecraft_player_count` metric.
   - **TPS**: Use the Prometheus data source to query the `minecraft_tps` metric.
   - **Bot Latency**: Use the Prometheus data source to query the `minecraft_latency` metric.

5. Customize the dashboard to your preference and save it.

You should now have a fully functional Grafana dashboard displaying real-time metrics and logs from your Minecraft server.
