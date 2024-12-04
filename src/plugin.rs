use azalea::*;
use prometheus::IntGauge;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;
use azalea::AzaleaPlugin;
use azalea::Player;

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
        logging::send_log_to_loki(&self.config.loki_endpoint, log).await;
    }

    async fn on_player_leave(&self, player: &Player) {
        let mut metrics = self.metrics.lock().await;
        metrics.player_count.dec();
        let log = Log::new("player_leave", player.username.clone(), None);
        logging::send_log_to_loki(&self.config.loki_endpoint, log).await;
    }

    async fn on_chat_message(&self, player: &Player, message: &str) {
        let log = Log::new("chat_message", player.username.clone(), Some(message.to_string()));
        logging::send_log_to_loki(&self.config.loki_endpoint, log).await;
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
