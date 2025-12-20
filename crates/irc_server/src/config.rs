use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub network: NetworkConfig,
    pub limits: LimitsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub version: String,
    pub motd: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NetworkConfig {
    pub bind_address: String,
    pub port: u16,
    pub max_connections: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LimitsConfig {
    pub max_channels_per_user: usize,
    pub max_message_length: usize,

    // Security & Anti-Flood
    pub max_connections_per_ip: usize,
    pub unregistered_timeout: u64,

    // These are commented out in your TOML.
    // We use Option so the parser doesn't fail if they are missing.
    pub max_channel_name_length: Option<usize>,
    pub max_topic_length: Option<usize>,
}

impl Config {
    /// Loads and parses the TOML configuration file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Helper to get channel name length with a hard fallback to RFC 2812 standard (200)
    pub fn get_max_channel_name_length(&self) -> usize {
        self.limits.max_channel_name_length.unwrap_or(200)
    }
}
