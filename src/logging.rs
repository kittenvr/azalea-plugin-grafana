use reqwest::Client;
use serde::Serialize;

#[derive(Serialize)]
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

pub async fn send_log_to_loki(client: &Client, endpoint: &str, log: Log) {
    let _ = client.post(endpoint).json(&log).send().await;
}
