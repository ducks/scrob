-- Create users table
CREATE TABLE IF NOT EXISTS users (
  id BIGSERIAL PRIMARY KEY,
  username TEXT UNIQUE NOT NULL,
  password_hash TEXT NOT NULL,
  is_admin BOOLEAN NOT NULL DEFAULT false,
  created_at BIGINT NOT NULL
);

-- Create api_tokens table
CREATE TABLE IF NOT EXISTS api_tokens (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL,
  token TEXT UNIQUE NOT NULL,
  label TEXT,
  created_at BIGINT NOT NULL,
  last_used_at BIGINT,
  revoked BOOLEAN NOT NULL DEFAULT false,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create scrobs table
CREATE TABLE IF NOT EXISTS scrobs (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL,
  artist TEXT NOT NULL,
  track TEXT NOT NULL,
  album TEXT,
  duration BIGINT,
  timestamp BIGINT NOT NULL,
  created_at BIGINT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_api_tokens_token ON api_tokens(token) WHERE revoked = false;
CREATE INDEX IF NOT EXISTS idx_api_tokens_user_id ON api_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_scrobs_user_id ON scrobs(user_id);
CREATE INDEX IF NOT EXISTS idx_scrobs_timestamp ON scrobs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_scrobs_user_timestamp ON scrobs(user_id, timestamp DESC);
