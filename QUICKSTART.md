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

The script outputs both a session token and optionally an API token (for your
music player).

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

```bash
curl -X POST http://localhost:3000/login \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "password": "mypassword"}'
```

Copy the token from the response. It will look something like:
```
16e2a3b4c5d6e7f8a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4
```

## Step 4: Use the Token

### Test Authentication

```bash
# Set your token
export TOKEN="your-token-here"

# Get recent scrobbles (empty if you haven't scrobbled yet)
curl http://localhost:3000/recent?limit=10 \
  -H "Authorization: Bearer $TOKEN"
```

## Step 5: Submit Your First Scrobble

```bash
curl -X POST http://localhost:3000/scrob \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '[{
    "artist": "Pink Floyd",
    "track": "Time",
    "album": "The Dark Side of the Moon",
    "duration": 413,
    "timestamp": '$(date +%s)'
  }]'
```

## Step 6: View Your Scrobbles

```bash
# Recent scrobbles
curl http://localhost:3000/recent?limit=10 \
  -H "Authorization: Bearer $TOKEN"

# Top artists
curl http://localhost:3000/top/artists?limit=10 \
  -H "Authorization: Bearer $TOKEN"

# Top tracks
curl http://localhost:3000/top/tracks?limit=10 \
  -H "Authorization: Bearer $TOKEN"
```

## Using with last-fm-rs Client

Once you have a token, you can use it with the Rust client:

```rust
use last_fm_rs::Client;

let client = Client::with_token(
    "http://localhost:3000",
    "your-api-token-here"
)?;

// Now scrobble from your music player
client.scrobble(&scrobbles).await?;
```

## Creating Additional Users

Once logged in, you can create additional users using the same methods from
Step 2, or continue using the Python/script method.

## Troubleshooting

### "Unauthorized" error
You need to include the `Authorization: Bearer <token>` header.

### "Invalid username or password"
Check the username and password are correct. Passwords are case-sensitive.

### Server won't start
- Check the database path exists
- Ensure migrations ran: `cargo sqlx migrate run`
- Check port 3000 is available

### Can't create user
- Ensure Python has bcrypt installed: `pip install bcrypt`
- Check database file permissions
- Verify database path is correct (`./data/scrob.db` for Docker, `./scrob.db`
  for local)
