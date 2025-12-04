use async_graphql::*;
use chrono::{DateTime, Utc};

/// GraphQL DateTime scalar
pub type DateTimeScalar = DateTime<Utc>;

/// User type
#[derive(Debug, Clone, SimpleObject)]
pub struct User {
  pub id: ID,
  pub username: String,
  pub is_admin: bool,
  pub created_at: DateTimeScalar,
}

impl From<crate::db::models::User> for User {
  fn from(u: crate::db::models::User) -> Self {
    Self {
      id: ID(u.id.to_string()),
      username: u.username,
      is_admin: u.is_admin,
      created_at: DateTime::from_timestamp(u.created_at, 0).unwrap(),
    }
  }
}

/// API Token type
#[derive(Debug, Clone, SimpleObject)]
pub struct ApiToken {
  pub id: ID,
  pub label: Option<String>,
  pub created_at: DateTimeScalar,
  pub last_used_at: Option<DateTimeScalar>,
  pub revoked: bool,
  /// The actual token value (only returned on creation)
  #[graphql(skip)]
  pub token: Option<String>,
}

impl From<crate::db::models::ApiToken> for ApiToken {
  fn from(t: crate::db::models::ApiToken) -> Self {
    Self {
      id: ID(t.id.to_string()),
      label: t.label,
      created_at: DateTime::from_timestamp(t.created_at, 0).unwrap(),
      last_used_at: t.last_used_at.and_then(|ts| DateTime::from_timestamp(ts, 0)),
      revoked: t.revoked,
      token: None,
    }
  }
}

/// Scrobble type
#[derive(Debug, Clone, SimpleObject)]
pub struct Scrob {
  pub id: ID,
  pub artist: String,
  pub track: String,
  pub album: Option<String>,
  pub duration: Option<i32>,
  pub timestamp: DateTimeScalar,
  pub created_at: DateTimeScalar,
}

impl From<crate::db::models::Scrob> for Scrob {
  fn from(s: crate::db::models::Scrob) -> Self {
    Self {
      id: ID(s.id.to_string()),
      artist: s.artist,
      track: s.track,
      album: s.album,
      duration: s.duration.map(|d| d as i32),
      timestamp: DateTime::from_timestamp(s.timestamp, 0).unwrap(),
      created_at: DateTime::from_timestamp(s.created_at, 0).unwrap(),
    }
  }
}

/// Top artist aggregation
#[derive(Debug, Clone, SimpleObject)]
pub struct TopArtist {
  pub name: String,
  pub count: i32,
}

/// Top track aggregation
#[derive(Debug, Clone, SimpleObject)]
pub struct TopTrack {
  pub artist: String,
  pub track: String,
  pub count: i32,
}

/// Scrobble input for mutations
#[derive(Debug, Clone, InputObject)]
pub struct ScrobInput {
  pub artist: String,
  pub track: String,
  pub album: Option<String>,
  pub duration: Option<i32>,
  pub timestamp: DateTimeScalar,
}

/// Now playing input for mutations
#[derive(Debug, Clone, InputObject)]
pub struct NowPlayingInput {
  pub artist: String,
  pub track: String,
  pub album: Option<String>,
  pub duration: Option<i32>,
}

/// Time range filter for statistics
#[derive(Debug, Clone, InputObject)]
pub struct TimeRangeInput {
  pub from: Option<DateTimeScalar>,
  pub to: Option<DateTimeScalar>,
}

/// Auth payload returned from login
#[derive(Debug, Clone, SimpleObject)]
pub struct AuthPayload {
  pub token: String,
  pub user: User,
}
