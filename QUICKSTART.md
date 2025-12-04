# Quick Start Guide

## Step 1: Start the Server

### With Docker
```bash
docker-compose up -d
```

### With Nix
```bash
nix-shell
cargo run
```

### Without Nix
```bash
export DATABASE_URL="sqlite:scrob.db"
cargo run
```

Server will be at: `http://localhost:3000`

## Step 2: Create Your First User

You need to create a user directly in the database since there's no public
registration endpoint yet.

### Option A: Using the Bootstrap Script (Easiest)

This script creates a user, logs in, and optionally creates an API token all in
one go:

```bash
./scripts/bootstrap.sh
```

It will prompt you for:
- Username
- Password
- Admin status (y/N)
- Whether to create an API token for a client

The script outputs both a session token (for GraphQL Playground) and optionally
an API token (for your music player).

**Skip to Step 4** if you use this option!

### Option B: Using Python (Manual)

```bash
python3 << 'EOF'
import sqlite3
import bcrypt
import time

# Configuration
username = 'alice'
password = 'mypassword'
is_admin = 1  # 1 for admin, 0 for regular user
db_path = './data/scrob.db'  # or './scrob.db' for local development

# Hash password and insert user
password_hash = bcrypt.hashpw(password.encode(), bcrypt.gensalt()).decode()
timestamp = int(time.time())

conn = sqlite3.connect(db_path)
cursor = conn.cursor()
cursor.execute(
    'INSERT INTO users (username, password_hash, is_admin, created_at) VALUES (?, ?, ?, ?)',
    (username, password_hash, is_admin, timestamp)
)
conn.commit()
conn.close()

print(f'User "{username}" created successfully')
EOF
```

### Option C: Using the Helper Script (User Only)

This only creates the user, doesn't login:

```bash
./scripts/create_user.sh alice mypassword true
```

### Option D: Using SQLite CLI

```bash
# Generate bcrypt hash first (using Python)
HASH=$(python3 -c "import bcrypt; print(bcrypt.hashpw(b'mypassword', bcrypt.gensalt()).decode())")

# Insert into database
sqlite3 ./data/scrob.db << EOF
INSERT INTO users (username, password_hash, is_admin, created_at)
VALUES ('alice', '$HASH', 1, $(date +%s));
EOF
```

## Step 3: Login and Get a Token

Open GraphQL Playground at `http://localhost:3000/playground` and run:

```graphql
mutation Login {
  login(username: "alice", password: "mypassword") {
    token
    user {
      id
      username
      isAdmin
    }
  }
}
```

Copy the token from the response. It will look something like:
```
16e2a3b4c5d6e7f8a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4
```

## Step 4: Use the Token

### In GraphQL Playground

Click "HTTP HEADERS" at the bottom of the playground and add:

```json
{
  "Authorization": "Bearer YOUR_TOKEN_HERE"
}
```

Now you can make authenticated requests:

```graphql
query Me {
  me {
    id
    username
    isAdmin
  }
}
```

### Create Additional API Tokens

For your music player or other clients:

```graphql
mutation CreateToken {
  createApiToken(label: "my-music-player") {
    id
    label
    token
  }
}
```

**Important**: The token value is only shown once! Copy it immediately.

## Step 5: Submit Your First Scrobble

```graphql
mutation FirstScrobble {
  scrob(input: {
    artist: "Pink Floyd"
    track: "Time"
    album: "The Dark Side of the Moon"
    duration: 413
    timestamp: "2025-12-03T12:00:00Z"
  }) {
    id
    artist
    track
    timestamp
  }
}
```

## Step 6: View Your Scrobbles

```graphql
query MyScrobbles {
  recentScrobs(limit: 10) {
    id
    artist
    track
    album
    timestamp
  }
}
```

## Using with last-fm-rs Client

Once you have a token, you can use it with the Rust client:

```rust
use last_fm_rs::Client;

let client = Client::with_token(
    "http://localhost:3000/graphql",
    "your-api-token-here"
)?;

// Now scrobble from your music player
client.scrobble(&scrobbles).await?;
```

## Creating Additional Users

Once logged in as an admin user, you can create tokens for yourself, then use
those tokens to create additional users programmatically, or continue using the
Python/script method from Step 2.

## Troubleshooting

### "Session key required" error
You need to include the `Authorization: Bearer <token>` header.

### "Invalid username or password"
Check the username and password are correct. Passwords are case-sensitive.

### "Authentication required"
The mutation/query requires a valid token. Make sure you've set the
Authorization header.

### Server won't start
- Check the database path exists
- Ensure migrations ran: `cargo sqlx migrate run`
- Check port 3000 is available

### Can't create user
- Ensure Python has bcrypt installed: `pip install bcrypt`
- Check database file permissions
- Verify database path is correct (./data/scrob.db for Docker, ./scrob.db for local)
