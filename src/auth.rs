use crate::db::{models::User, DbPool};

/// Extract token from Authorization: Bearer <token> header
pub fn extract_token_from_header(auth_header: &str) -> Option<String> {
  auth_header
    .strip_prefix("Bearer ")
    .map(|t| t.trim().to_string())
}

/// Look up user by token
pub async fn get_user_by_token(pool: &DbPool, token: &str) -> Result<Option<User>, sqlx::Error> {
  let now = chrono::Utc::now().timestamp();

  // Find token and verify it's not revoked
  let token_row = sqlx::query!(
    r#"
    SELECT user_id as "user_id!"
    FROM api_tokens
    WHERE token = ? AND revoked = 0
    "#,
    token
  )
  .fetch_optional(pool)
  .await?;

  let user_id = match token_row {
    Some(row) => row.user_id,
    None => return Ok(None),
  };

  // Update last_used_at
  sqlx::query!(
    r#"
    UPDATE api_tokens
    SET last_used_at = ?
    WHERE token = ?
    "#,
    now,
    token
  )
  .execute(pool)
  .await?;

  // Fetch user
  let user = sqlx::query_as!(
    User,
    r#"
    SELECT id as "id!", username, password_hash, is_admin as "is_admin: bool", created_at as "created_at!"
    FROM users
    WHERE id = ?
    "#,
    user_id
  )
  .fetch_optional(pool)
  .await?;

  Ok(user)
}

/// Generate a random API token
pub fn generate_token() -> String {
  use std::time::{SystemTime, UNIX_EPOCH};

  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();

  let random_bytes: Vec<u8> = (0..32)
    .map(|_| rand::random::<u8>())
    .collect();

  format!("{:x}{}", timestamp, hex::encode(&random_bytes))
}

/// Hash a password using bcrypt
pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
  bcrypt::hash(password, bcrypt::DEFAULT_COST)
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
  bcrypt::verify(password, hash)
}
