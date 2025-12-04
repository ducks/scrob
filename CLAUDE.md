# Claude Development Notes

## Project Overview

This is a self-hosted music scrobble server with a GraphQL API, designed to work
with the last-fm-rs client library. It provides an alternative to Last.fm's API
for tracking music listening history.

## Architecture

### Tech Stack
- **Language**: Rust (edition 2024)
- **HTTP Server**: axum 0.8
- **GraphQL**: async-graphql 7.0
- **Database**: SQLite (via sqlx 0.7) with offline query checking
- **Auth**: Token-based (Bearer tokens)
- **Password Hashing**: bcrypt

### Key Design Decisions

1. **API-only server**: No server-side HTML rendering. All clients (web UI, TUI,
   music players) communicate via GraphQL over HTTP.

2. **Token-based auth**: Single unified auth model:
   - Machine clients (music players) use static API tokens
   - Human users login via GraphQL mutation, receive token for UI storage
   - All requests use `Authorization: Bearer <token>` header

3. **SQLite with portability**: Using SQLite for v1 simplicity, but SQL is
   written to be portable to Postgres. All queries use sqlx macros for
   compile-time checking.

4. **GraphQL over REST**: Chosen for flexibility, introspection, and to allow
   UI developers to query exactly what they need.

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
- `timestamp` = when track was played
- `created_at` = when scrobble was recorded

## Code Organization

```
src/
├── main.rs           - Axum setup, routing, GraphQL handler
├── config.rs         - Environment variable parsing
├── auth.rs           - Token validation, password hashing
├── db/
│   ├── mod.rs        - Pool creation, migration runner
│   └── models.rs     - sqlx::FromRow types
└── graphql/
    ├── mod.rs        - Schema construction
    ├── types.rs      - GraphQL types (User, Scrob, inputs)
    ├── context.rs    - Request context with current user
    ├── query.rs      - Read operations
    └── mutation.rs   - Write operations
```

## SQLx Query Macros

### Important: Type Annotations

SQLite + sqlx requires explicit type annotations for NOT NULL fields that use
AUTOINCREMENT. Use the `!` suffix to mark fields as NOT NULL:

```rust
sqlx::query_as!(
  User,
  r#"
  SELECT id as "id!", username, password_hash,
         is_admin as "is_admin: bool", created_at as "created_at!"
  FROM users
  WHERE id = ?
  "#,
  user_id
)
```

Without `!` annotations, sqlx infers `Option<i64>` which doesn't match the model.

### Offline Query Checking

The `.sqlx/` directory contains query metadata for offline compilation. This was
generated with:

```bash
export DATABASE_URL="sqlite:scrob.db"
cargo sqlx prepare
```

**Commit `.sqlx/` to version control** to allow builds without a database.

## Authentication Flow

### Initial Setup (Bootstrap)

1. **Create first user directly in database** using one of:
   - Python script with bcrypt
   - `./scripts/create_user.sh` helper
   - Direct SQL with pre-hashed password

2. **Login via GraphQL** to get initial token:
   ```graphql
   mutation { login(username: "alice", password: "pass") { token } }
   ```

3. Now you have a token and can use the GraphQL API normally.

### For Web UIs

1. User calls `login(username, password)` mutation
2. Server validates credentials, creates new token with label "UI session"
3. UI stores token (localStorage, etc.)
4. UI sends token in `Authorization: Bearer <token>` on all requests
5. Server extracts token → looks up in `api_tokens` → resolves user → puts in
   GraphQL context

### For Machine Clients

1. User logs into web UI (or GraphQL Playground)
2. User calls `createApiToken(label: "my-music-player")` mutation (requires auth)
3. Token is returned once (only time the full value is visible)
4. User copies token into music player config
5. Music player sends token on all requests

**Important**: `createApiToken` requires authentication, so you must already
have a token (from `login`) to create additional tokens.

### Token Resolution (main.rs:74-98)

Token extraction and user resolution happens in the `graphql_handler`:
- Extract `Authorization` header
- Parse `Bearer <token>`
- Look up token in database (check not revoked)
- Update `last_used_at`
- Fetch associated user
- Put user in GraphQL context

If no token or invalid token, `current_user` is `None` in context.

## GraphQL API Design

### Query Root

- `me` - Returns current user or null (no auth required to call, returns null)
- `recentScrobs` - Requires auth, user's recent listens
- `topArtists` - Requires auth, aggregated artist stats
- `topTracks` - Requires auth, aggregated track stats

### Mutation Root

- `login` - No auth required, returns `AuthPayload` with token and user
- `scrob` - Requires auth, single scrobble
- `scrobBatch` - Requires auth, up to 50 scrobbles
- `nowPlaying` - Requires auth, stub implementation (just returns true)
- `createApiToken` - Requires auth, returns token with full value
- `revokeApiToken` - Requires auth, soft-deletes token

### Context Pattern

`GraphQLContext::require_user()` is a helper that returns the user or an error.
Use this in resolvers that need authentication:

```rust
let gql_ctx = ctx.data::<GraphQLContext>()?;
let user = gql_ctx.require_user()?;
```

## Integration with last-fm-rs

The client library (https://github.com/ducks/last-fm-rs) has been extended with
a token mode that posts JSON to this server:

- `POST {base_url}/now` for now-playing (maps to `nowPlaying` mutation)
- `POST {base_url}/scrob` for scrobbles (maps to `scrob`/`scrobBatch` mutations)

The client sends `Authorization: Bearer <token>` and JSON bodies matching the
`NowPlayingInput` and `ScrobInput` types.

**Note**: The current implementation uses GraphQL for everything. If integrating
with last-fm-rs, you may want to add REST-style POST handlers at `/now` and
`/scrob` that internally call the GraphQL mutations, or update last-fm-rs to
send GraphQL requests.

## Testing Strategy

### Manual Testing with GraphQL Playground

1. Start server: `cargo run`
2. Visit `http://localhost:3000/playground`
3. Create user via script: `./scripts/create_user.sh alice password true`
4. Login mutation to get token
5. Set HTTP header in playground: `{"Authorization": "Bearer <token>"}`
6. Test mutations and queries

### Unit Tests

Currently no unit tests. Future additions should test:
- Password hashing/verification (auth.rs)
- Token generation (auth.rs)
- GraphQL context helpers (graphql/context.rs)

### Integration Tests

Future: Use a test database and test the full GraphQL API flow.

## Common Development Tasks

### Adding a New Query

1. Add resolver method to `QueryRoot` in `src/graphql/query.rs`
2. Use `#[Object]` macro, method must be `async`
3. Extract `GraphQLContext` and `DbPool` from `ctx`
4. Optionally require auth with `require_user()`
5. Query database with sqlx
6. Return GraphQL types (not db models)

### Adding a New Mutation

Same as query, but in `src/graphql/mutation.rs`.

### Adding a Database Table

1. Create new migration: `migrations/00X_description.sql`
2. Add model to `src/db/models.rs` with `#[derive(FromRow)]`
3. Run migration: `cargo sqlx migrate run`
4. Update `.sqlx/`: `cargo sqlx prepare`

### Schema Changes

After changing GraphQL types/resolvers, restart the server and refresh the
playground to see updated schema.

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

1. **No now_playing table**: The `nowPlaying` mutation just logs and returns
   true. Could add a `now_playing` table with one row per user.

2. **No pagination**: `recentScrobs` supports `before` cursor but no `after`.
   `topArtists` and `topTracks` just limit results.

3. **No search**: No full-text search for artists/tracks.

4. **No user management UI**: Must create first user via script or direct DB
   access.

5. **No bulk operations**: No bulk delete, bulk update, etc.

6. **SQLite limitations**: No concurrent writes (though reads are fine). For
   high-traffic deployments, migrate to Postgres.

### Future Enhancements

1. **User registration**: Add `register` mutation for self-service signup.

2. **Password reset**: Email-based password reset flow.

3. **Scrobble editing**: Allow users to edit/delete their scrobbles.

4. **Export**: Export scrobbles to JSON/CSV.

5. **Statistics**: More detailed stats (listening time, streak tracking, etc.).

6. **Artist/Album metadata**: Fetch from MusicBrainz or similar.

7. **Postgres support**: Add feature flag for Postgres vs SQLite.

8. **Rate limiting**: Prevent abuse of the API.

9. **WebSocket subscriptions**: Real-time updates for now-playing across
   devices.

10. **Admin panel**: GraphQL mutations for user management, token revocation,
    etc.

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

### GraphQL errors

- Use GraphQL Playground to inspect full error messages
- Check `RUST_LOG=scrob=debug` for detailed logging
- Verify all required fields are provided in mutations

## Best Practices

1. **Always use parameterized queries**: Never string interpolation for SQL.

2. **Use GraphQL context for auth**: Don't pass user_id as GraphQL arguments.

3. **Validate input early**: Check constraints (e.g., max 50 scrobbles) before
   DB operations.

4. **Use transactions for multi-step mutations**: Not yet implemented, but
   future mutations that touch multiple tables should use transactions.

5. **Keep GraphQL types separate from DB models**: Allows independent evolution
   of API and storage.

6. **Use meaningful error messages**: Return user-friendly errors from
   mutations.

7. **Log authentication failures**: Helps detect brute-force attempts.

## Resources

- async-graphql docs: https://async-graphql.github.io/async-graphql/
- sqlx docs: https://docs.rs/sqlx/
- axum docs: https://docs.rs/axum/
- GraphQL spec: https://spec.graphql.org/

## Contact

For questions about this codebase, refer to the README or check the last-fm-rs
integration docs.
