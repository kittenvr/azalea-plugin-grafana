use reqwest::Client;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::Result;

#[derive(Debug, Serialize)]
pub struct Log {
    pub event: String,
    pub username: String,
    pub message: Option<String>,
    pub timestamp: u64,
}

impl Log {
    pub fn new(event: impl Into<String>, username: impl Into<String>, message: Option<String>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Log {
            event: event.into(),
            username: username.into(),
            message,
            timestamp,
        }
    }
}

pub struct Logger {
    client: Client,
    endpoint: String,
}

impl Logger {
    pub fn new(endpoint: String) -> Self {
        Logger {
            client: Client::new(),
            endpoint,
        }
    }

    pub async fn send_log(&self, log: Log) -> Result<()> {
        self.client
            .post(&self.endpoint)
            .json(&log)
            .send()
            .await?;
        Ok(())
    }
}
