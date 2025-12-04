-- Create users table
CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  username TEXT UNIQUE NOT NULL,
  password_hash TEXT NOT NULL,
  is_admin BOOLEAN NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL
);

-- Create api_tokens table
CREATE TABLE IF NOT EXISTS api_tokens (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL,
  token TEXT UNIQUE NOT NULL,
  label TEXT,
  created_at INTEGER NOT NULL,
  last_used_at INTEGER,
  revoked BOOLEAN NOT NULL DEFAULT 0,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create scrobs table
CREATE TABLE IF NOT EXISTS scrobs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL,
  artist TEXT NOT NULL,
  track TEXT NOT NULL,
  album TEXT,
  duration INTEGER,
  timestamp INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_api_tokens_token ON api_tokens(token) WHERE revoked = 0;
CREATE INDEX IF NOT EXISTS idx_api_tokens_user_id ON api_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_scrobs_user_id ON scrobs(user_id);
CREATE INDEX IF NOT EXISTS idx_scrobs_timestamp ON scrobs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_scrobs_user_timestamp ON scrobs(user_id, timestamp DESC);
