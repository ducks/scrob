use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct User {
  pub id: i64,
  pub username: String,
  pub password_hash: String,
  pub is_admin: bool,
  pub created_at: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct ApiToken {
  pub id: i64,
  pub user_id: i64,
  pub token: String,
  pub label: Option<String>,
  pub created_at: i64,
  pub last_used_at: Option<i64>,
  pub revoked: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct Scrob {
  pub id: i64,
  pub user_id: i64,
  pub artist: String,
  pub track: String,
  pub album: Option<String>,
  pub duration: Option<i64>,
  pub timestamp: i64,
  pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct TopArtist {
  pub name: String,
  pub count: i64,
}

#[derive(Debug, Clone)]
pub struct TopTrack {
  pub artist: String,
  pub track: String,
  pub count: i64,
}
