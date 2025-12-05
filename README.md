# scrob

Self-hosted music scrobble server with REST API.

**New to scrob?** Check out [QUICKSTART.md](QUICKSTART.md) for a step-by-step guide.

**Developing?** See [CLAUDE.md](CLAUDE.md) for architecture notes and best practices.

## Features

- REST API for scrobble submission and statistics
- Token-based authentication
- SQLite database (portable to Postgres)
- Docker support
- Compatible with last-fm-rs client library

## Quick Start with Docker

### Using Docker Compose

```bash
# Build and start
docker-compose up -d

# View logs
docker-compose logs -f

# Stop
docker-compose down
```

The server will be available at `http://localhost:3000`.

### Manual Docker Build

```bash
# Build image
docker build -t scrob .

# Run container
docker run -d \
  -p 3000:3000 \
  -v $(pwd)/data:/app/data \
  --name scrob \
  scrob
```

## Development

### Prerequisites

- Rust 1.82+
- SQLite3
- sqlx-cli: `cargo install sqlx-cli --no-default-features --features sqlite`

### Setup

```bash
# Install dependencies
cargo build

# Run migrations
export DATABASE_URL="sqlite:scrob.db"
cargo sqlx migrate run

# Start server
cargo run
```

### Environment Variables

- `DATABASE_URL` - Database connection string (default: `sqlite:scrob.db`)
- `HOST` - Bind address (default: `127.0.0.1`)
- `PORT` - Port number (default: `3000`)
- `RUST_LOG` - Logging level (default: `scrob=info`)

## Creating Users

### Quick Bootstrap (Recommended)

Use the interactive bootstrap script to create a user, login, and get tokens:

```bash
./scripts/bootstrap.sh
```

This handles everything for you and outputs the tokens you need.

### Using Python (requires bcrypt)

```bash
python3 -c "
import sqlite3
import bcrypt
import time

username = 'alice'
password = 'mypassword'
is_admin = 1  # 1 for admin, 0 for regular user

hash = bcrypt.hashpw(password.encode(), bcrypt.gensalt()).decode()
timestamp = int(time.time())

conn = sqlite3.connect('./data/scrob.db')
conn.execute(
    'INSERT INTO users (username, password_hash, is_admin, created_at) VALUES (?, ?, ?, ?)',
    (username, hash, is_admin, timestamp)
)
conn.commit()
print(f'User {username} created')
"
```

### Using the helper script

```bash
# Requires Python 3 with bcrypt installed
./scripts/create_user.sh alice mypassword true
```

## REST API

### Authentication

```bash
# Login and get token
curl -X POST http://localhost:3000/login \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "password": "mypassword"}'

# Response: {"token": "...", "username": "alice", "is_admin": false}
```

Use the returned token in the `Authorization` header for all protected endpoints:
```
Authorization: Bearer <token>
```

### Submit Scrobbles

```bash
curl -X POST http://localhost:3000/scrob \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '[{
    "artist": "Kendrick Lamar",
    "track": "Wesley'\''s Theory",
    "album": "To Pimp a Butterfly",
    "duration": 287,
    "timestamp": 1701619200
  }]'
```

### Get Recent Scrobbles

```bash
curl http://localhost:3000/recent?limit=20 \
  -H "Authorization: Bearer <token>"
```

### Get Top Artists

```bash
curl http://localhost:3000/top/artists?limit=10 \
  -H "Authorization: Bearer <token>"
```

### Get Top Tracks

```bash
curl http://localhost:3000/top/tracks?limit=10 \
  -H "Authorization: Bearer <token>"
```

### Now Playing

```bash
curl -X POST http://localhost:3000/now \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "artist": "Pink Floyd",
    "track": "Time",
    "album": "The Dark Side of the Moon"
  }'
```

## Integration with last-fm-rs

This server is designed to work with the [last-fm-rs](https://github.com/ducks/last-fm-rs) client library in token mode:

```rust
use last_fm_rs::Client;

let client = Client::with_token(
  "http://localhost:3000",
  "your-api-token"
)?;

// Use as normal
client.update_now_playing(&now_playing).await?;
client.scrobble(&scrobbles).await?;
```

## Database Schema

### users
- `id` - Primary key
- `username` - Unique username
- `password_hash` - Bcrypt password hash
- `is_admin` - Admin flag
- `created_at` - Unix timestamp

### api_tokens
- `id` - Primary key
- `user_id` - Foreign key to users
- `token` - Unique token string
- `label` - Optional label (e.g., "desktop", "phone")
- `created_at` - Unix timestamp
- `last_used_at` - Unix timestamp (updated on use)
- `revoked` - Revocation flag

### scrobs
- `id` - Primary key
- `user_id` - Foreign key to users
- `artist` - Artist name
- `track` - Track name
- `album` - Album name (optional)
- `duration` - Duration in seconds (optional)
- `timestamp` - When the track was played (Unix timestamp)
- `created_at` - When the scrobble was recorded (Unix timestamp)

## License

MIT OR Apache-2.0
