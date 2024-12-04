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
