use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::auth::AuthUser;

#[derive(Debug, Deserialize)]
pub struct NowPlayingRequest {
    pub artist: String,
    pub track: String,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub duration: Option<u64>,
    pub track_number: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ScrobbleRequest {
    pub artist: String,
    pub track: String,
    pub timestamp: u64,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub duration: Option<u64>,
    pub track_number: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ScrobbleResponse {
    pub id: i64,
    pub artist: String,
    pub track: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn now_playing(
    headers: axum::http::HeaderMap,
    State(pool): State<SqlitePool>,
    Json(req): Json<NowPlayingRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let user = AuthUser::from_headers(&pool, &headers).await
        .map_err(|status| (status, Json(ErrorResponse { error: "Unauthorized".to_string() })))?;
    // For now-playing, we just log it - we don't store it
    tracing::info!(
        "Now playing for user {}: {} - {}",
        user.id,
        req.artist,
        req.track
    );

    Ok(StatusCode::OK)
}

pub async fn scrobble(
    headers: axum::http::HeaderMap,
    State(pool): State<SqlitePool>,
    Json(scrobbles): Json<Vec<ScrobbleRequest>>,
) -> Result<Json<Vec<ScrobbleResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let user = AuthUser::from_headers(&pool, &headers).await
        .map_err(|status| (status, Json(ErrorResponse { error: "Unauthorized".to_string() })))?;

    tracing::info!("Received {} scrobble(s) from user {}", scrobbles.len(), user.id);

    let mut results = Vec::new();

    for scrob in scrobbles {
        let now = chrono::Utc::now().timestamp();
        let timestamp = scrob.timestamp as i64;
        let duration = scrob.duration.map(|d| d as i64);

        let result = sqlx::query!(
            r#"
            INSERT INTO scrobs (user_id, artist, track, album, duration, timestamp, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            user.id,
            scrob.artist,
            scrob.track,
            scrob.album,
            duration,
            timestamp,
            now
        )
        .execute(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            )
        })?;

        let scrob_id = result.last_insert_rowid();

        tracing::info!(
            "Scrobbled for user {}: {} - {} (id: {})",
            user.id,
            scrob.artist,
            scrob.track,
            scrob_id
        );

        results.push(ScrobbleResponse {
            id: scrob_id,
            artist: scrob.artist,
            track: scrob.track,
            timestamp,
        });
    }

    Ok(Json(results))
}
