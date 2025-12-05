# PostgreSQL Setup Guide

Scrob uses PostgreSQL for production deployments. This guide covers setting up
Postgres for local development and production use.

## Why PostgreSQL?

- Handles concurrent writes better than SQLite
- Required for multi-user deployments
- Better performance at scale
- Industry standard for web applications

## Local Development Setup

### Installing PostgreSQL

**On NixOS/with Nix:**
```bash
nix-shell -p postgresql
```

**On Ubuntu/Debian:**
```bash
sudo apt install postgresql postgresql-contrib
```

**On macOS:**
```bash
brew install postgresql
brew services start postgresql
```

**On Arch:**
```bash
sudo pacman -S postgresql
sudo systemctl start postgresql
```

### Creating the Database

```bash
# Create database user (if needed)
sudo -u postgres createuser -P scrob
# Enter password when prompted

# Create database
sudo -u postgres createdb -O scrob scrob

# Or using psql
sudo -u postgres psql
postgres=# CREATE USER scrob WITH PASSWORD 'your_password';
postgres=# CREATE DATABASE scrob OWNER scrob;
postgres=# \q
```

### Setting Up Connection String

Create a `.env` file or set environment variable:

```bash
# For local development
export DATABASE_URL="postgres://scrob:your_password@localhost/scrob"

# Or for peer authentication (no password)
export DATABASE_URL="postgres://localhost/scrob"
```

### Running Migrations

```bash
cd ~/dev/scrob
export DATABASE_URL="postgres://scrob:your_password@localhost/scrob"

# Run migrations
nix-shell --run "cargo sqlx migrate run"
```

### Regenerating Query Cache

After migrations, regenerate the sqlx query cache:

```bash
nix-shell --run "cargo sqlx prepare"
```

This creates/updates the `.sqlx/` directory which is committed to git for
offline builds.

## Production Setup

### Connection String Format

```
postgres://username:password@host:port/database?sslmode=require
```

For managed Postgres services (AWS RDS, DigitalOcean, etc.), always use
`sslmode=require`.

### Managed PostgreSQL Services

**DigitalOcean Managed Database:**
- They provide the full connection string
- Includes SSL certificate
- Example: `postgres://user:pass@db-host-do.db.ondigitalocean.com:25060/scrob?sslmode=require`

**AWS RDS:**
- Create Postgres instance in RDS console
- Use security groups to allow access
- Connection: `postgres://user:pass@instance.region.rds.amazonaws.com:5432/scrob?sslmode=require`

**Render:**
- Provides free Postgres instances
- Connection string in dashboard
- Auto-expires after 90 days on free tier

### Environment Variables

For production, set:

```bash
DATABASE_URL=postgres://user:pass@host/db?sslmode=require
HOST=0.0.0.0
PORT=3000
RUST_LOG=scrob=info
```

### Database Backups

**Using pg_dump:**
```bash
pg_dump -h host -U scrob scrob > backup.sql
```

**Automated backups:**
```bash
# Add to crontab
0 2 * * * pg_dump -h localhost -U scrob scrob | gzip > /backups/scrob-$(date +\%Y\%m\%d).sql.gz
```

**Restore:**
```bash
psql -h host -U scrob scrob < backup.sql
```

## Connecting to Postgres

### Using psql

```bash
# Connect to local database
psql -U scrob scrob

# Connect to remote database
psql postgres://user:pass@host/scrob?sslmode=require
```

### Common psql Commands

```sql
-- List tables
\dt

-- Describe table
\d users

-- Show table sizes
SELECT
  schemaname,
  tablename,
  pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- Count scrobbles
SELECT COUNT(*) FROM scrobs;

-- Recent scrobbles
SELECT artist, track, to_timestamp(timestamp) FROM scrobs ORDER BY timestamp DESC LIMIT 10;
```

## Migrating from SQLite

If you're migrating from SQLite to Postgres:

### 1. Export SQLite Data

```bash
sqlite3 scrob.db .dump > sqlite_dump.sql
```

### 2. Convert SQLite SQL to Postgres

The dump needs some modifications:
- Remove SQLite-specific pragmas
- Change `INTEGER PRIMARY KEY AUTOINCREMENT` to `BIGSERIAL PRIMARY KEY`
- Change `INTEGER` to `BIGINT` for timestamps
- Change `BOOLEAN` values from `0/1` to `false/true`

Or use a migration tool like `pgloader`:

```bash
pgloader sqlite://scrob.db postgres://scrob:pass@localhost/scrob
```

### 3. Verify Data

```sql
SELECT COUNT(*) FROM users;
SELECT COUNT(*) FROM scrobs;
SELECT COUNT(*) FROM api_tokens;
```

## Troubleshooting

### Connection Refused

- Check Postgres is running: `sudo systemctl status postgresql`
- Check port is open: `sudo lsof -i :5432`
- Check `pg_hba.conf` for authentication settings

### Authentication Failed

- Verify username/password in connection string
- Check `pg_hba.conf` for peer vs md5 authentication
- Try: `sudo -u postgres psql` to get in as superuser

### Permission Denied

```sql
-- Grant permissions to user
GRANT ALL PRIVILEGES ON DATABASE scrob TO scrob;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO scrob;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO scrob;
```

### Migrations Fail

- Check current migration version: `SELECT * FROM _sqlx_migrations;`
- To reset: `DROP TABLE IF EXISTS _sqlx_migrations; DROP TABLE users; DROP TABLE api_tokens; DROP TABLE scrobs;`
- Then re-run: `cargo sqlx migrate run`

## Performance Tuning

For production Postgres:

```sql
-- Enable query logging (temporarily)
ALTER DATABASE scrob SET log_statement = 'all';

-- Check slow queries
SELECT query, calls, total_time, mean_time
FROM pg_stat_statements
ORDER BY mean_time DESC
LIMIT 10;

-- Analyze tables
ANALYZE scrobs;
ANALYZE users;
```

## Connection Pooling

Scrob uses sqlx's built-in connection pooling. Default settings are good for
most use cases. For high traffic, tune with environment variables:

```bash
# Maximum connections (default: 10)
SQLX_MAX_CONNECTIONS=20

# Minimum idle connections (default: 0)
SQLX_MIN_CONNECTIONS=5
```
