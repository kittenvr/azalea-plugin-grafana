use std::fmt;

#[derive(Debug)]
pub enum PluginError {
    Config(String),
    Bot(String),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::Config(msg) => write!(f, "Configuration Error: {}", msg),
            PluginError::Bot(msg) => write!(f, "Bot Error: {}", msg),
        }
    }
}

impl std::error::Error for PluginError {}
