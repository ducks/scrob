use async_graphql::*;
use chrono::Utc;

use crate::{
  auth::{generate_token, verify_password},
  db::DbPool,
};
use super::types::{AuthPayload, Scrob, ScrobInput, NowPlayingInput, ApiToken};
use super::context::GraphQLContext;

pub struct MutationRoot;

#[Object]
impl MutationRoot {
  /// Login with username and password, returns a token for the UI to use
  async fn login(
    &self,
    ctx: &Context<'_>,
    username: String,
    password: String,
  ) -> Result<AuthPayload> {
    let pool = ctx.data::<DbPool>()?;

    // Find user
    let user = sqlx::query_as!(
      crate::db::models::User,
      r#"
      SELECT id as "id!", username, password_hash, is_admin as "is_admin: bool", created_at as "created_at!"
      FROM users
      WHERE username = ?
      "#,
      username
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| Error::new("Invalid username or password"))?;

    // Verify password
    if !verify_password(&password, &user.password_hash)
      .map_err(|e| Error::new(format!("Password verification failed: {}", e)))? {
      return Err(Error::new("Invalid username or password"));
    }

    // Create new token for this login
    let token = generate_token();
    let now = Utc::now().timestamp();

    sqlx::query!(
      r#"
      INSERT INTO api_tokens (user_id, token, label, created_at, revoked)
      VALUES (?, ?, ?, ?, 0)
      "#,
      user.id,
      token,
      "UI session",
      now
    )
    .execute(pool)
    .await?;

    Ok(AuthPayload {
      token,
      user: user.into(),
    })
  }

  /// Submit a single scrobble
  async fn scrob(&self, ctx: &Context<'_>, input: ScrobInput) -> Result<Scrob> {
    let gql_ctx = ctx.data::<GraphQLContext>()?;
    let user = gql_ctx.require_user()?;
    let pool = ctx.data::<DbPool>()?;

    let now = Utc::now().timestamp();
    let timestamp = input.timestamp.timestamp();
    let duration = input.duration.map(|d| d as i64);

    let result = sqlx::query!(
      r#"
      INSERT INTO scrobs (user_id, artist, track, album, duration, timestamp, created_at)
      VALUES (?, ?, ?, ?, ?, ?, ?)
      "#,
      user.id,
      input.artist,
      input.track,
      input.album,
      duration,
      timestamp,
      now
    )
    .execute(pool)
    .await?;

    let scrob_id = result.last_insert_rowid();
    let scrob = sqlx::query_as!(
      crate::db::models::Scrob,
      r#"
      SELECT id as "id!", user_id as "user_id!", artist, track, album, duration, timestamp as "timestamp!", created_at as "created_at!"
      FROM scrobs
      WHERE id = ?
      "#,
      scrob_id
    )
    .fetch_one(pool)
    .await?;

    Ok(scrob.into())
  }

  /// Submit multiple scrobbles in a batch
  async fn scrob_batch(&self, ctx: &Context<'_>, inputs: Vec<ScrobInput>) -> Result<Vec<Scrob>> {
    let gql_ctx = ctx.data::<GraphQLContext>()?;
    let user = gql_ctx.require_user()?;
    let pool = ctx.data::<DbPool>()?;

    if inputs.len() > 50 {
      return Err(Error::new("Maximum 50 scrobbles per batch"));
    }

    let now = Utc::now().timestamp();
    let mut scrobs = Vec::new();

    for input in inputs {
      let timestamp = input.timestamp.timestamp();
      let duration = input.duration.map(|d| d as i64);

      let result = sqlx::query!(
        r#"
        INSERT INTO scrobs (user_id, artist, track, album, duration, timestamp, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        user.id,
        input.artist,
        input.track,
        input.album,
        duration,
        timestamp,
        now
      )
      .execute(pool)
      .await?;

      let scrob_id = result.last_insert_rowid();
      let scrob = sqlx::query_as!(
        crate::db::models::Scrob,
        r#"
        SELECT id as "id!", user_id as "user_id!", artist, track, album, duration, timestamp as "timestamp!", created_at as "created_at!"
        FROM scrobs
        WHERE id = ?
        "#,
        scrob_id
      )
      .fetch_one(pool)
      .await?;

      scrobs.push(scrob.into());
    }

    Ok(scrobs)
  }

  /// Update now playing status (stub implementation for v1)
  async fn now_playing(&self, ctx: &Context<'_>, input: NowPlayingInput) -> Result<bool> {
    let gql_ctx = ctx.data::<GraphQLContext>()?;
    let _user = gql_ctx.require_user()?;

    // For v1, just return success
    // Could store in a now_playing table in the future
    tracing::debug!(
      "Now playing for user: {} - {}",
      input.artist,
      input.track
    );

    Ok(true)
  }

  /// Create a new API token for the authenticated user
  async fn create_api_token(&self, ctx: &Context<'_>, label: Option<String>) -> Result<ApiToken> {
    let gql_ctx = ctx.data::<GraphQLContext>()?;
    let user = gql_ctx.require_user()?;
    let pool = ctx.data::<DbPool>()?;

    let token_value = generate_token();
    let now = Utc::now().timestamp();

    let result = sqlx::query!(
      r#"
      INSERT INTO api_tokens (user_id, token, label, created_at, revoked)
      VALUES (?, ?, ?, ?, 0)
      "#,
      user.id,
      token_value,
      label,
      now
    )
    .execute(pool)
    .await?;

    let token_id = result.last_insert_rowid();
    let mut api_token: ApiToken = sqlx::query_as!(
      crate::db::models::ApiToken,
      r#"
      SELECT id, user_id, token, label, created_at, last_used_at, revoked as "revoked: bool"
      FROM api_tokens
      WHERE id = ?
      "#,
      token_id
    )
    .fetch_one(pool)
    .await?
    .into();

    // Include the actual token value only on creation
    api_token.token = Some(token_value);

    Ok(api_token)
  }

  /// Revoke an API token
  async fn revoke_api_token(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
    let gql_ctx = ctx.data::<GraphQLContext>()?;
    let user = gql_ctx.require_user()?;
    let pool = ctx.data::<DbPool>()?;

    let token_id: i64 = id.parse()
      .map_err(|_| Error::new("Invalid token ID"))?;

    // Ensure the token belongs to the user
    let result = sqlx::query!(
      r#"
      UPDATE api_tokens
      SET revoked = 1
      WHERE id = ? AND user_id = ?
      "#,
      token_id,
      user.id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
  }
}
