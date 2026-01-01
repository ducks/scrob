use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::AuthUser;

#[derive(Debug, Deserialize)]
pub struct PrivacyUpdate {
    pub is_private: bool,
}

#[derive(Debug, Serialize)]
pub struct PrivacyResponse {
    pub is_private: bool,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn update_privacy(
    headers: axum::http::HeaderMap,
    State(pool): State<PgPool>,
    Json(payload): Json<PrivacyUpdate>,
) -> Result<Json<PrivacyResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user = AuthUser::from_headers(&pool, &headers).await
        .map_err(|status| (status, Json(ErrorResponse { error: "Unauthorized".to_string() })))?;

    sqlx::query!(
        "UPDATE users SET is_private = $1 WHERE id = $2",
        payload.is_private,
        user.id
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

    Ok(Json(PrivacyResponse {
        is_private: payload.is_private,
    }))
}

pub async fn get_privacy(
    headers: axum::http::HeaderMap,
    State(pool): State<PgPool>,
) -> Result<Json<PrivacyResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user = AuthUser::from_headers(&pool, &headers).await
        .map_err(|status| (status, Json(ErrorResponse { error: "Unauthorized".to_string() })))?;

    Ok(Json(PrivacyResponse {
        is_private: user.user.is_private,
    }))
}
