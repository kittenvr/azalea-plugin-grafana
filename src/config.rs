use serde::Deserialize;
use std::fs;
use crate::error::{Result, PluginError};

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub prometheus_endpoint: String,
    pub loki_endpoint: String,
    pub bot_token: String,
    pub server_address: String,
    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,
}

fn default_metrics_port() -> u16 {
    3030
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_str = fs::read_to_string("config.toml")
            .map_err(|e| PluginError::Config(format!("Failed to read config file: {}", e)))?;
            
        toml::from_str(&config_str)
            .map_err(|e| PluginError::Config(format!("Failed to parse config: {}", e)))
    }

    pub fn validate(&self) -> Result<()> {
        if self.prometheus_endpoint.is_empty() {
            return Err(PluginError::Config("Prometheus endpoint cannot be empty".into()));
        }
        if self.loki_endpoint.is_empty() {
            return Err(PluginError::Config("Loki endpoint cannot be empty".into()));
        }
        if self.bot_token.is_empty() {
            return Err(PluginError::Config("Bot token cannot be empty".into()));
        }
        if self.server_address.is_empty() {
            return Err(PluginError::Config("Server address cannot be empty".into()));
        }
        Ok(())
    }
}
