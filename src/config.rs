use std::env;
use std::sync::LazyLock;

pub struct Config {
    pub public_root_url: String,
    pub host: String,
    pub port: String,
    pub secret: String,
    pub database_url: String,
}

impl Config {
    pub fn get_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

static DEVELOPMENT_ENCRYPTION_KEY: &str = "czNjcjN0LXMzY3IzdC1zM2NyM3QtczNjcjN0LXMzY3IzdA==";

pub static APP_CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let public_root_url =
        env::var("PUBLIC_ROOT_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let host = env::var("APPLICATION_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("APPLICATION_PORT").unwrap_or_else(|_| "3000".to_string());
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| DEVELOPMENT_ENCRYPTION_KEY.to_string());
    let database_url = env::var("DATABASE_URL").unwrap_or(String::from("postgres://username:password@host/database?currentSchema=my_schema"));

    Config {
        public_root_url,
        host,
        port,
        secret,
        database_url,
    }
});
