use std::env;
use std::sync::LazyLock;

pub struct Config {
    pub host: String,
    pub port: String,
    pub secret: String,
}

impl Config {
    pub fn get_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn get_root_url(&self) -> String {
        format!("http://{}", self.get_address())
    }
}

static DEVELOPMENT_ENCRYPTION_KEY: &str = "czNjcjN0LXMzY3IzdC1zM2NyM3QtczNjcjN0LXMzY3IzdA==";

pub static APP_CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let host = std::env::var("APPLICATION_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = std::env::var("APPLICATION_PORT").unwrap_or_else(|_| "3000".to_string());
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| DEVELOPMENT_ENCRYPTION_KEY.to_string());

    Config {
        host,
        port,
        secret,
    }
});