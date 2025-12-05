use axum::{extract::{Query, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::AuthUser;

#[derive(Debug, Deserialize)]
pub struct RecentScrobsQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct TopQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct Scrob {
    pub id: i64,
    pub artist: String,
    pub track: String,
    pub album: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Serialize)]
pub struct TopArtist {
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct TopTrack {
    pub artist: String,
    pub track: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn recent_scrobbles(
    headers: axum::http::HeaderMap,
    State(pool): State<PgPool>,
    Query(query): Query<RecentScrobsQuery>,
) -> Result<Json<Vec<Scrob>>, (StatusCode, Json<ErrorResponse>)> {
    let user = AuthUser::from_headers(&pool, &headers).await
        .map_err(|status| (status, Json(ErrorResponse { error: "Unauthorized".to_string() })))?;
    let limit = query.limit.unwrap_or(20).min(100);

    let scrobs = sqlx::query_as!(
        Scrob,
        r#"
        SELECT id as "id!", artist, track, album, timestamp as "timestamp!"
        FROM scrobs
        WHERE user_id = $1
        ORDER BY timestamp DESC
        LIMIT $2
        "#,
        user.id,
        limit
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        )
    })?;

    Ok(Json(scrobs))
}

pub async fn top_artists(
    headers: axum::http::HeaderMap,
    State(pool): State<PgPool>,
    Query(query): Query<TopQuery>,
) -> Result<Json<Vec<TopArtist>>, (StatusCode, Json<ErrorResponse>)> {
    let user = AuthUser::from_headers(&pool, &headers).await
        .map_err(|status| (status, Json(ErrorResponse { error: "Unauthorized".to_string() })))?;
    let limit = query.limit.unwrap_or(10).min(100);

    let artists = sqlx::query_as!(
        TopArtist,
        r#"
        SELECT artist as name, COUNT(*) as "count!: i64"
        FROM scrobs
        WHERE user_id = $1
        GROUP BY artist
        ORDER BY COUNT(*) DESC
        LIMIT $2
        "#,
        user.id,
        limit
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        )
    })?;

    Ok(Json(artists))
}

pub async fn top_tracks(
    headers: axum::http::HeaderMap,
    State(pool): State<PgPool>,
    Query(query): Query<TopQuery>,
) -> Result<Json<Vec<TopTrack>>, (StatusCode, Json<ErrorResponse>)> {
    let user = AuthUser::from_headers(&pool, &headers).await
        .map_err(|status| (status, Json(ErrorResponse { error: "Unauthorized".to_string() })))?;
    let limit = query.limit.unwrap_or(10).min(100);

    let tracks = sqlx::query_as!(
        TopTrack,
        r#"
        SELECT artist as "artist!", track as "track!", COUNT(*) as "count!: i64"
        FROM scrobs
        WHERE user_id = $1
        GROUP BY artist, track
        ORDER BY COUNT(*) DESC
        LIMIT $2
        "#,
        user.id,
        limit
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        )
    })?;

    Ok(Json(tracks))
}
