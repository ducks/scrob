use async_graphql::*;

use crate::db::DbPool;
use super::types::{User, Scrob, TopArtist, TopTrack, TimeRangeInput, DateTimeScalar};
use super::context::GraphQLContext;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
  /// Get the currently authenticated user
  async fn me(&self, ctx: &Context<'_>) -> Result<Option<User>> {
    let gql_ctx = ctx.data::<GraphQLContext>()?;

    Ok(gql_ctx.current_user.clone().map(|u| u.into()))
  }

  /// Get recent scrobbles for the authenticated user
  async fn recent_scrobs(
    &self,
    ctx: &Context<'_>,
    #[graphql(default = 20)] limit: i32,
    before: Option<DateTimeScalar>,
  ) -> Result<Vec<Scrob>> {
    let gql_ctx = ctx.data::<GraphQLContext>()?;
    let user = gql_ctx.require_user()?;
    let pool = ctx.data::<DbPool>()?;

    let limit = limit.min(100) as i64;
    let before_ts = before.map(|dt| dt.timestamp()).unwrap_or(i64::MAX);

    let scrobs = sqlx::query_as!(
      crate::db::models::Scrob,
      r#"
      SELECT id as "id!", user_id as "user_id!", artist, track, album, duration, timestamp as "timestamp!", created_at as "created_at!"
      FROM scrobs
      WHERE user_id = ? AND timestamp < ?
      ORDER BY timestamp DESC
      LIMIT ?
      "#,
      user.id,
      before_ts,
      limit
    )
    .fetch_all(pool)
    .await?;

    Ok(scrobs.into_iter().map(|s| s.into()).collect())
  }

  /// Get top artists for the authenticated user
  async fn top_artists(
    &self,
    ctx: &Context<'_>,
    range: Option<TimeRangeInput>,
    #[graphql(default = 20)] limit: i32,
  ) -> Result<Vec<TopArtist>> {
    let gql_ctx = ctx.data::<GraphQLContext>()?;
    let user = gql_ctx.require_user()?;
    let pool = ctx.data::<DbPool>()?;

    let limit = limit.min(100) as i64;
    let from_ts = range.as_ref().and_then(|r| r.from.map(|dt| dt.timestamp())).unwrap_or(0);
    let to_ts = range.as_ref().and_then(|r| r.to.map(|dt| dt.timestamp())).unwrap_or(i64::MAX);

    let results = sqlx::query!(
      r#"
      SELECT artist as name, COUNT(*) as count
      FROM scrobs
      WHERE user_id = ? AND timestamp >= ? AND timestamp <= ?
      GROUP BY artist
      ORDER BY count DESC
      LIMIT ?
      "#,
      user.id,
      from_ts,
      to_ts,
      limit
    )
    .fetch_all(pool)
    .await?;

    Ok(results.into_iter().map(|r| TopArtist {
      name: r.name,
      count: r.count as i32,
    }).collect())
  }

  /// Get top tracks for the authenticated user
  async fn top_tracks(
    &self,
    ctx: &Context<'_>,
    range: Option<TimeRangeInput>,
    #[graphql(default = 20)] limit: i32,
  ) -> Result<Vec<TopTrack>> {
    let gql_ctx = ctx.data::<GraphQLContext>()?;
    let user = gql_ctx.require_user()?;
    let pool = ctx.data::<DbPool>()?;

    let limit = limit.min(100) as i64;
    let from_ts = range.as_ref().and_then(|r| r.from.map(|dt| dt.timestamp())).unwrap_or(0);
    let to_ts = range.as_ref().and_then(|r| r.to.map(|dt| dt.timestamp())).unwrap_or(i64::MAX);

    let results = sqlx::query!(
      r#"
      SELECT artist, track, COUNT(*) as count
      FROM scrobs
      WHERE user_id = ? AND timestamp >= ? AND timestamp <= ?
      GROUP BY artist, track
      ORDER BY count DESC
      LIMIT ?
      "#,
      user.id,
      from_ts,
      to_ts,
      limit
    )
    .fetch_all(pool)
    .await?;

    Ok(results.into_iter().map(|r| TopTrack {
      artist: r.artist,
      track: r.track,
      count: r.count as i32,
    }).collect())
  }
}
