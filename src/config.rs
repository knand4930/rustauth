// config.rs

use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_maxage: i64,
    pub server_host: String,
    pub server_port: u16,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env"),
            jwt_secret: env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set in .env"),
            jwt_maxage: env::var("JWT_MAXAGE")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .expect("JWT_MAXAGE must be a valid integer"),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .expect("SERVER_PORT must be a valid u16"),
        }
    }
}
