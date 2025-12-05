# Claude Development Notes

## Project Overview

This is a self-hosted music scrobble server with a REST API, designed to work
with the last-fm-rs client library. It provides an alternative to Last.fm's
API for tracking music listening history.

## Architecture

### Tech Stack
- **Language**: Rust (edition 2024)
- **HTTP Server**: axum 0.8
- **Database**: SQLite (via sqlx 0.7) with offline query checking
- **Auth**: Token-based (Bearer tokens)
- **Password Hashing**: bcrypt
- **CORS**: tower-http CORS layer (permissive)

### Key Design Decisions

1. **API-only server**: No server-side HTML rendering. All clients (web UI,
   TUI, music players) communicate via REST over HTTP.

2. **Token-based auth**: Single unified auth model:
   - Machine clients (music players) use static API tokens
   - Human users login via POST /login, receive token for UI storage
   - All requests use `Authorization: Bearer <token>` header
   - Token validation happens via axum extractor (`AuthUser::from_headers`)

3. **SQLite with portability**: Using SQLite for v1 simplicity, but SQL is
   written to be portable to Postgres. All queries use sqlx macros for
   compile-time checking.

4. **REST over GraphQL**: Simple REST endpoints for straightforward CRUD
   operations. No need for query complexity or introspection features.

## Database Schema

### users
- Primary auth table
- Bcrypt password hashes (cost 12)
- Admin flag for future RBAC

### api_tokens
- One-to-many with users
- `revoked` flag for soft deletion
- `last_used_at` auto-updated on each request
- `label` for user-friendly identification

### scrobs
- Core scrobble data
- Artist/track required, album/duration optional
- `timestamp` = when track was played (Unix timestamp)
- `created_at` = when scrobble was recorded (Unix timestamp)

## Code Organization

```
src/
├── main.rs           - Axum setup, routing, CORS
├── config.rs         - Environment variable parsing
├── auth.rs           - Token validation, password hashing, AuthUser extractor
├── db/
│   ├── mod.rs        - Pool creation, migration runner
│   └── models.rs     - sqlx::FromRow types
└── routes/
    ├── mod.rs        - Module exports
    ├── auth.rs       - POST /login endpoint
    ├── scrobble.rs   - POST /now, POST /scrob endpoints
    └── stats.rs      - GET /recent, GET /top/artists, GET /top/tracks
```

## SQLx Query Macros

### Important: Type Annotations

SQLite + sqlx requires explicit type annotations for NOT NULL fields that use
AUTOINCREMENT. Use the `!` suffix to mark fields as NOT NULL:

```rust
sqlx::query_as!(
  Scrob,
  r#"
  SELECT id as "id!", artist, track, album, timestamp as "timestamp!"
  FROM scrobs
  WHERE user_id = ?
  "#,
  user_id
)
```

Without `!` annotations, sqlx infers `Option<i64>` which doesn't match the
model.

### Offline Query Checking

The `.sqlx/` directory contains query metadata for offline compilation. This
was generated with:

```bash
export DATABASE_URL="sqlite:scrob.db"
cargo sqlx prepare
```

Commit `.sqlx/` to version control to allow builds without a database.

## Authentication Flow

### Initial Setup (Bootstrap)

1. Create first user directly in database using one of:
   - Python script with bcrypt
   - `./scripts/create_user.sh` helper
   - `./scripts/bootstrap.sh` (interactive, creates user + gets token)
   - Direct SQL with pre-hashed password

2. Login via POST /login to get initial token:
   ```bash
   curl -X POST http://localhost:3000/login \
     -H "Content-Type: application/json" \
     -d '{"username": "alice", "password": "pass"}'
   ```

3. Response contains token, username, and admin status.

### For Web UIs

1. User calls POST /login with username/password
2. Server validates credentials, creates new token with label "session"
3. UI stores token (localStorage, etc.)
4. UI sends token in `Authorization: Bearer <token>` on all requests
5. Server extracts token via `AuthUser::from_headers`, looks up in
   `api_tokens`, resolves user

### For Machine Clients

1. User logs into web UI
2. User creates API token (future: POST /tokens endpoint, or via bootstrap
   script)
3. Token is returned once (only time the full value is visible)
4. User copies token into music player config
5. Music player sends token on all requests

### Token Resolution (auth.rs)

Token extraction and user resolution happens in `AuthUser::from_headers`:
- Extract `Authorization` header
- Parse `Bearer <token>`
- Look up token in database (check not revoked)
- Update `last_used_at`
- Fetch associated user
- Return `AuthUser` or error

All protected endpoints use this extractor to require authentication.

## REST API Design

### Authentication

**POST /login**
- Body: `{"username": "alice", "password": "pass"}`
- Response: `{"token": "...", "username": "alice", "is_admin": false}`
- No auth required

### Scrobbling

**POST /now**
- Body: `{"artist": "...", "track": "...", "album": "..."}`
- Response: 200 OK (currently just logs, doesn't store)
- Requires auth

**POST /scrob**
- Body: Array of scrobbles with `artist`, `track`, `timestamp`, optional
  `album`, `duration`
- Response: Array of created scrobbles with ids
- Requires auth
- Accepts batch submissions (array of scrobbles)

### Statistics

**GET /recent?limit=20**
- Returns recent scrobbles for authenticated user
- Query param: `limit` (default 20, max 100)
- Response: Array of scrobbles with id, artist, track, album, timestamp
- Requires auth

**GET /top/artists?limit=10**
- Returns top artists by play count
- Query param: `limit` (default 10, max 100)
- Response: Array of `{"name": "...", "count": 123}`
- Requires auth

**GET /top/tracks?limit=10**
- Returns top tracks by play count
- Query param: `limit` (default 10, max 100)
- Response: Array of `{"artist": "...", "track": "...", "count": 123}`
- Requires auth

### Health Check

**GET /health**
- Returns 200 OK
- No auth required

## Integration with last-fm-rs

The client library (https://github.com/ducks/last-fm-rs) has token mode that
posts JSON to this server:

- `POST /now` for now-playing updates
- `POST /scrob` for scrobble submissions

The client sends `Authorization: Bearer <token>` and JSON bodies matching the
request types in `routes/scrobble.rs`.

## Testing Strategy

### Manual Testing with curl

```bash
# Login
TOKEN=$(curl -s -X POST http://localhost:3000/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"pass"}' | jq -r .token)

# Submit scrobble
curl -X POST http://localhost:3000/scrob \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '[{"artist":"Pink Floyd","track":"Time","timestamp":1701619200}]'

# Get recent scrobbles
curl http://localhost:3000/recent?limit=10 \
  -H "Authorization: Bearer $TOKEN"

# Get top artists
curl http://localhost:3000/top/artists?limit=5 \
  -H "Authorization: Bearer $TOKEN"
```

### Unit Tests

Currently no unit tests. Future additions should test:
- Password hashing/verification (auth.rs)
- Token generation (auth.rs)
- AuthUser extractor (auth.rs)

### Integration Tests

Future: Use a test database and test the full REST API flow.

## Common Development Tasks

### Adding a New Endpoint

1. Add handler function to appropriate file in `src/routes/`
2. Handler should accept `State(pool): State<SqlitePool>` for database access
3. Protected endpoints should accept `headers: axum::http::HeaderMap` and call
   `AuthUser::from_headers(&pool, &headers).await?`
4. Use axum extractors: `Json<T>` for request body, `Query<T>` for query
   params
5. Return `Result<Json<Response>, (StatusCode, Json<ErrorResponse>)>`
6. Add route to router in `src/main.rs`

### Adding a Database Table

1. Create new migration: `migrations/00X_description.sql`
2. Add model to `src/db/models.rs` with `#[derive(FromRow)]`
3. Run migration: `cargo sqlx migrate run`
4. Update `.sqlx/`: `cargo sqlx prepare`

### Schema Changes

After changing the database schema, restart the server and test endpoints.

## Deployment

### With Docker

```bash
docker-compose up -d
```

Data persists in `./data/` directory on host.

### With Nix

```bash
nix-shell
cargo build --release
./target/release/scrob
```

### Environment Variables

- `DATABASE_URL` - SQLite path (default: `sqlite:scrob.db`)
- `HOST` - Bind address (default: `127.0.0.1`, use `0.0.0.0` for Docker)
- `PORT` - Port number (default: `3000`)
- `RUST_LOG` - Logging (default: `scrob=info`)

## Known Limitations / Future Work

### Current Limitations

1. **No now_playing table**: The POST /now endpoint just logs and returns 200.
   Could add a `now_playing` table with one row per user.

2. **No pagination**: `/recent` supports limit but no cursor-based pagination.

3. **No search**: No full-text search for artists/tracks.

4. **No user management endpoints**: Must create first user via script or
   direct DB access. No POST /register or token management endpoints yet.

5. **No bulk operations**: No bulk delete, bulk update, etc.

6. **SQLite limitations**: No concurrent writes (though reads are fine). For
   high-traffic deployments, migrate to Postgres.

### Future Enhancements

1. **User registration**: Add POST /register for self-service signup.

2. **Token management**: Add POST /tokens, GET /tokens, DELETE /tokens/:id for
   API token CRUD.

3. **Scrobble editing**: Allow users to edit/delete their scrobbles via PUT
   /scrobs/:id and DELETE /scrobs/:id.

4. **Export**: Add GET /export endpoint for JSON/CSV export.

5. **Statistics**: More detailed stats (listening time, streak tracking,
   per-album stats).

6. **Artist/Album metadata**: Fetch from MusicBrainz or similar.

7. **Postgres support**: Add feature flag for Postgres vs SQLite.

8. **Rate limiting**: Prevent abuse of the API.

9. **WebSocket subscriptions**: Real-time updates for now-playing across
   devices.

10. **Admin endpoints**: User management, token revocation, etc.

## Debugging Tips

### Server won't start

- Check `DATABASE_URL` is set correctly
- Ensure migrations have run: `cargo sqlx migrate run`
- Check port 3000 isn't already in use

### SQLx compile errors

- Ensure database exists and migrations are current
- Run `cargo sqlx prepare` to update query metadata
- Check all `!` annotations on NOT NULL fields

### Authentication not working

- Check `Authorization` header format: `Bearer <token>` (not `bearer`)
- Verify token exists in database and `revoked = 0`
- Check server logs for token lookup errors

### REST errors

- Check request body matches expected JSON structure
- Verify all required fields are provided
- Use `RUST_LOG=scrob=debug` for detailed logging

## Best Practices

1. **Always use parameterized queries**: Never string interpolation for SQL.

2. **Use AuthUser extractor for auth**: Don't manually parse tokens in each
   handler.

3. **Validate input early**: Check constraints (e.g., max limit) before DB
   operations.

4. **Use transactions for multi-step operations**: Not yet implemented, but
   future operations that touch multiple tables should use transactions.

5. **Keep response types separate from DB models**: Allows independent
   evolution of API and storage.

6. **Use meaningful error messages**: Return user-friendly errors from
   handlers.

7. **Log authentication failures**: Helps detect brute-force attempts.

## Resources

- axum docs: https://docs.rs/axum/
- sqlx docs: https://docs.rs/sqlx/
- tower-http docs: https://docs.rs/tower-http/
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/

## Contact

For questions about this codebase, refer to the README or check the last-fm-rs
integration docs.
