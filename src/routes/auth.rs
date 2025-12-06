use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::{generate_token, hash_password, verify_password};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
    pub is_admin: bool,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user = sqlx::query!(
        r#"
        SELECT id as "id!", username, password_hash, is_admin as "is_admin: bool"
        FROM users
        WHERE username = $1
        "#,
        req.username
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
    })?;

    let user = user.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid username or password".to_string(),
            }),
        )
    })?;

    if !verify_password(&req.password, &user.password_hash).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Password verification error: {}", e),
            }),
        )
    })? {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid username or password".to_string(),
            }),
        ));
    }

    let token = generate_token();
    let now = chrono::Utc::now().timestamp();

    sqlx::query!(
        r#"
        INSERT INTO api_tokens (user_id, token, label, created_at, revoked)
        VALUES ($1, $2, 'session', $3, false)
        "#,
        user.id,
        token,
        now
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create session: {}", e),
            }),
        )
    })?;

    Ok(Json(LoginResponse {
        token,
        username: user.username,
        is_admin: user.is_admin,
    }))
}

pub async fn signup(
    State(pool): State<PgPool>,
    Json(req): Json<SignupRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate username (alphanumeric and underscores only, 3-20 chars)
    if req.username.len() < 3 || req.username.len() > 20 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Username must be between 3 and 20 characters".to_string(),
            }),
        ));
    }

    if !req.username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Username can only contain letters, numbers, and underscores".to_string(),
            }),
        ));
    }

    // Validate password length
    if req.password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Password must be at least 8 characters".to_string(),
            }),
        ));
    }

    if req.password.len() > 72 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Password must be at most 72 characters".to_string(),
            }),
        ));
    }

    // Validate password complexity
    let has_lowercase = req.password.chars().any(|c| c.is_lowercase());
    let has_uppercase = req.password.chars().any(|c| c.is_uppercase());
    let has_digit = req.password.chars().any(|c| c.is_numeric());

    if !has_lowercase || !has_uppercase || !has_digit {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Password must contain at least one lowercase letter, one uppercase letter, and one number".to_string(),
            }),
        ));
    }

    // Check if username already exists
    let existing = sqlx::query!(
        r#"
        SELECT id
        FROM users
        WHERE username = $1
        "#,
        req.username
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
    })?;

    if existing.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "Username already exists".to_string(),
            }),
        ));
    }

    // Hash password
    let password_hash = hash_password(&req.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Password hashing error: {}", e),
            }),
        )
    })?;

    let now = chrono::Utc::now().timestamp();

    // Create user (first user is admin)
    let user = sqlx::query!(
        r#"
        INSERT INTO users (username, password_hash, is_admin, created_at)
        VALUES ($1, $2, NOT EXISTS(SELECT 1 FROM users), $3)
        RETURNING id as "id!", username, is_admin as "is_admin: bool"
        "#,
        req.username,
        password_hash,
        now
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create user: {}", e),
            }),
        )
    })?;

    // Generate token
    let token = generate_token();

    sqlx::query!(
        r#"
        INSERT INTO api_tokens (user_id, token, label, created_at, revoked)
        VALUES ($1, $2, 'session', $3, false)
        "#,
        user.id,
        token,
        now
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create session: {}", e),
            }),
        )
    })?;

    Ok(Json(LoginResponse {
        token,
        username: user.username,
        is_admin: user.is_admin,
    }))
}
