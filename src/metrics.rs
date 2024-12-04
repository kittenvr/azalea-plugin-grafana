use async_trait::async_trait;
use azalea::AzaleaPlugin;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::{config::Config, metrics::Metrics, logging::{Logger, Log}};

pub struct Plugin {
    metrics: Arc<Mutex<Metrics>>,
    logger: Arc<Logger>,
}

impl Plugin {
    pub fn new(metrics: Metrics, config: &Config) -> Self {
        Plugin {
            metrics: Arc::new(Mutex::new(metrics)),
            logger: Arc::new(Logger::new(config.loki_endpoint.clone())),
        }
    }
}

#[async_trait]
impl AzaleaPlugin for Plugin {
    async fn on_player_join(&self, player: &azalea::Player) {
        let mut metrics = self.metrics.lock().await;
        metrics.player_count.inc();
        
        let log = Log::new("player_join", &player.username, None);
        if let Err(e) = self.logger.send_log(log).await {
            tracing::error!("Failed to send join log: {}", e);
        }
    }

    async fn on_player_leave(&self, player: &azalea::Player) {
        let mut metrics = self.metrics.lock().await;
        metrics.player_count.dec();
        
        let log = Log::new("player_leave", &player.username, None);
        if let Err(e) = self.logger.send_log(log).await {
            tracing::error!("Failed to send leave log: {}", e);
        }
    }

    async fn on_chat_message(&self, player: &azalea::Player, message: &str) {
        let log = Log::new("chat_message", &player.username, Some(message.to_string()));
        if let Err(e) = self.logger.send_log(log).await {
            tracing::error!("Failed to send chat log: {}", e);
        }
    }

    async fn on_tick(&self, tps: f64, latency: u64) {
        let mut metrics = self.metrics.lock().await;
        metrics.update(
            metrics.player_count.get(),
            tps as i64,
            latency as i64
        );
    }
}
