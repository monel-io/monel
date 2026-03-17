use std::time::Duration;

/// Server configuration.
pub struct Config {
    pub port: u16,
    pub host: String,
    pub tls: Option<TlsConfig>,
    pub max_connections: usize,
    pub request_timeout: Duration,
}

pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "0.0.0.0".into(),
            tls: None,
            max_connections: 1024,
            request_timeout: Duration::from_secs(30),
        }
    }
}
