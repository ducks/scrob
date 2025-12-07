use axum::{extract::State, http::StatusCode, Json, extract::Path};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::AuthUser;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// User Management

#[derive(Debug, Serialize)]
pub struct UserListItem {
    pub id: i64,
    pub username: String,
    pub is_admin: bool,
    pub created_at: i64,
    pub scrobble_count: i64,
}

#[derive(Debug, Serialize)]
pub struct UserDetail {
    pub id: i64,
    pub username: String,
    pub is_admin: bool,
    pub created_at: i64,
    pub scrobble_count: i64,
    pub last_scrobble: Option<i64>,
}

pub async fn list_users(
    headers: axum::http::HeaderMap,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<UserListItem>>, (StatusCode, Json<ErrorResponse>)> {
    let auth = AuthUser::from_headers(&pool, &headers).await
        .map_err(|status| (status, Json(ErrorResponse { error: "Unauthorized".to_string() })))?;

    if !auth.is_admin {
        return Err((StatusCode::FORBIDDEN, Json(ErrorResponse { error: "Admin access required".to_string() })));
    }

    let users = sqlx::query!(
        r#"
        SELECT
            u.id as "id!",
            u.username,
            u.is_admin as "is_admin: bool",
            u.created_at as "created_at!",
            COUNT(s.id) as "scrobble_count!"
        FROM users u
        LEFT JOIN scrobs s ON u.id = s.user_id
        GROUP BY u.id, u.username, u.is_admin, u.created_at
        ORDER BY u.created_at DESC
        "#
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

    Ok(Json(users.into_iter().map(|u| UserListItem {
        id: u.id,
        username: u.username,
        is_admin: u.is_admin,
        created_at: u.created_at,
        scrobble_count: u.scrobble_count,
    }).collect()))
}

pub async fn get_user(
    headers: axum::http::HeaderMap,
    State(pool): State<PgPool>,
    Path(user_id): Path<i64>,
) -> Result<Json<UserDetail>, (StatusCode, Json<ErrorResponse>)> {
    let auth = AuthUser::from_headers(&pool, &headers).await
        .map_err(|status| (status, Json(ErrorResponse { error: "Unauthorized".to_string() })))?;

    if !auth.is_admin {
        return Err((StatusCode::FORBIDDEN, Json(ErrorResponse { error: "Admin access required".to_string() })));
    }

    let user = sqlx::query!(
        r#"
        SELECT
            id as "id!",
            username,
            is_admin as "is_admin: bool",
            created_at as "created_at!"
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        )
    })?
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(ErrorResponse { error: "User not found".to_string() })))?;

    let scrobble_count = sqlx::query!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM scrobs
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        )
    })?;

    let last_scrobble = sqlx::query!(
        r#"
        SELECT MAX(timestamp) as "last_scrobble"
        FROM scrobs
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        )
    })?;

    Ok(Json(UserDetail {
        id: user.id,
        username: user.username,
        is_admin: user.is_admin,
        created_at: user.created_at,
        scrobble_count: scrobble_count.count,
        last_scrobble: last_scrobble.last_scrobble,
    }))
}

pub async fn delete_user(
    headers: axum::http::HeaderMap,
    State(pool): State<PgPool>,
    Path(user_id): Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let auth = AuthUser::from_headers(&pool, &headers).await
        .map_err(|status| (status, Json(ErrorResponse { error: "Unauthorized".to_string() })))?;

    if !auth.is_admin {
        return Err((StatusCode::FORBIDDEN, Json(ErrorResponse { error: "Admin access required".to_string() })));
    }

    if auth.id == user_id {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Cannot delete yourself".to_string() })));
    }

    // Delete user's scrobbles first
    sqlx::query!("DELETE FROM scrobs WHERE user_id = $1", user_id)
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

    // Delete user's tokens
    sqlx::query!("DELETE FROM api_tokens WHERE user_id = $1", user_id)
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

    // Delete user
    let result = sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
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

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, Json(ErrorResponse { error: "User not found".to_string() })));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct ToggleAdminRequest {
    pub is_admin: bool,
}

pub async fn toggle_admin(
    headers: axum::http::HeaderMap,
    State(pool): State<PgPool>,
    Path(user_id): Path<i64>,
    Json(req): Json<ToggleAdminRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let auth = AuthUser::from_headers(&pool, &headers).await
        .map_err(|status| (status, Json(ErrorResponse { error: "Unauthorized".to_string() })))?;

    if !auth.is_admin {
        return Err((StatusCode::FORBIDDEN, Json(ErrorResponse { error: "Admin access required".to_string() })));
    }

    if auth.id == user_id {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Cannot change your own admin status".to_string() })));
    }

    let result = sqlx::query!(
        "UPDATE users SET is_admin = $1 WHERE id = $2",
        req.is_admin,
        user_id
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

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, Json(ErrorResponse { error: "User not found".to_string() })));
    }

    Ok(StatusCode::OK)
}

// System Stats

#[derive(Debug, Serialize)]
pub struct SystemStats {
    pub total_users: i64,
    pub total_scrobbles: i64,
    pub total_artists: i64,
    pub total_tracks: i64,
}

#[derive(Debug, Serialize)]
pub struct TopUser {
    pub username: String,
    pub scrobble_count: i64,
}

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub stats: SystemStats,
    pub top_users: Vec<TopUser>,
}

pub async fn get_stats(
    headers: axum::http::HeaderMap,
    State(pool): State<PgPool>,
) -> Result<Json<StatsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let auth = AuthUser::from_headers(&pool, &headers).await
        .map_err(|status| (status, Json(ErrorResponse { error: "Unauthorized".to_string() })))?;

    if !auth.is_admin {
        return Err((StatusCode::FORBIDDEN, Json(ErrorResponse { error: "Admin access required".to_string() })));
    }

    let total_users = sqlx::query!("SELECT COUNT(*) as \"count!\" FROM users")
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            )
        })?;

    let total_scrobbles = sqlx::query!("SELECT COUNT(*) as \"count!\" FROM scrobs")
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            )
        })?;

    let total_artists = sqlx::query!("SELECT COUNT(DISTINCT artist) as \"count!\" FROM scrobs")
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            )
        })?;

    let total_tracks = sqlx::query!("SELECT COUNT(DISTINCT artist || ' - ' || track) as \"count!\" FROM scrobs")
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            )
        })?;

    let top_users = sqlx::query!(
        r#"
        SELECT
            u.username,
            COUNT(s.id) as "scrobble_count!"
        FROM users u
        LEFT JOIN scrobs s ON u.id = s.user_id
        GROUP BY u.id, u.username
        HAVING COUNT(s.id) > 0
        ORDER BY COUNT(s.id) DESC
        LIMIT 10
        "#
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

    Ok(Json(StatsResponse {
        stats: SystemStats {
            total_users: total_users.count,
            total_scrobbles: total_scrobbles.count,
            total_artists: total_artists.count,
            total_tracks: total_tracks.count,
        },
        top_users: top_users.into_iter().map(|u| TopUser {
            username: u.username,
            scrobble_count: u.scrobble_count,
        }).collect(),
    }))
}

// Moderation

pub async fn delete_scrobble(
    headers: axum::http::HeaderMap,
    State(pool): State<PgPool>,
    Path(scrobble_id): Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let auth = AuthUser::from_headers(&pool, &headers).await
        .map_err(|status| (status, Json(ErrorResponse { error: "Unauthorized".to_string() })))?;

    if !auth.is_admin {
        return Err((StatusCode::FORBIDDEN, Json(ErrorResponse { error: "Admin access required".to_string() })));
    }

    let result = sqlx::query!("DELETE FROM scrobs WHERE id = $1", scrobble_id)
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

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, Json(ErrorResponse { error: "Scrobble not found".to_string() })));
    }

    Ok(StatusCode::NO_CONTENT)
}
