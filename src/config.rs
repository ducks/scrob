use std::env;

#[derive(Debug, Clone)]
pub struct Config {
  pub database_url: String,
  pub port: u16,
  pub host: String,
}

impl Config {
  pub fn from_env() -> Result<Self, String> {
    let database_url = env::var("DATABASE_URL")
      .unwrap_or_else(|_| "postgres://localhost/scrob".to_string());

    let port = env::var("PORT")
      .unwrap_or_else(|_| "3000".to_string())
      .parse()
      .map_err(|e| format!("Invalid PORT: {}", e))?;

    let host = env::var("HOST")
      .unwrap_or_else(|_| "127.0.0.1".to_string());

    Ok(Self {
      database_url,
      port,
      host,
    })
  }

  pub fn bind_address(&self) -> String {
    format!("{}:{}", self.host, self.port)
  }
}
