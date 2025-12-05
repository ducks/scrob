# Deployment Guide

## Quick Start with Docker Compose (Recommended)

The easiest way to run scrob is with Docker Compose, which handles everything
automatically including PostgreSQL, the server, and the UI.

### Prerequisites

- Docker and Docker Compose installed

### Setup

1. **Start the full stack**

```bash
cd ~/dev/scrob
docker-compose up -d
```

This will start:
- PostgreSQL database on port 5432
- Scrob server on port 3000
- Scrob UI on port 8080

2. **Create your first user**

```bash
# Get a shell in the scrob container
docker-compose exec scrob /bin/bash

# Run the create user script
./scripts/create_user.sh your_username your_password true
```

3. **Access scrob**

Visit `http://localhost:8080` and login with your credentials.

That's it! Docker handles building, migrations, and running all services.

### Configuration

The default `docker-compose.yml` uses the password `scrob_password_change_me`.
To change it:

1. Edit `docker-compose.yml`
2. Update `POSTGRES_PASSWORD` in the postgres service
3. Update the password in `DATABASE_URL` in the scrob service
4. Run `docker-compose down && docker-compose up -d`

---

## Manual Installation with systemd (Alternative)

If you prefer to run without Docker, use the automated setup script.

### Prerequisites

- Linux system with systemd
- PostgreSQL installed and running
- Nix installed (or Rust 1.82+)

### Quick Setup Script

```bash
cd ~/dev/scrob
./scripts/install.sh
```

This script will:
1. Build the release binary
2. Install to `/opt/scrob`
3. Run database migrations
4. Create systemd service
5. Prompt you to create a user

See [scripts/install.sh](scripts/install.sh) for details.

### Manual Setup Steps

If you prefer to do it manually:

#### 1. Build the Server

```bash
cd ~/dev/scrob
nix-shell --run "cargo build --release"
```

The binary will be at `target/release/scrob`.

#### 2. Choose Installation Location

```bash
# Create application directory
sudo mkdir -p /opt/scrob
sudo chown $USER:$USER /opt/scrob

# Copy binary
cp target/release/scrob /opt/scrob/

# Copy migrations
cp -r migrations /opt/scrob/
cp -r scripts /opt/scrob/
```

#### 3. Set Up PostgreSQL

```bash
# Create database and user
sudo -u postgres psql <<EOF
CREATE DATABASE scrob;
CREATE USER scrob WITH PASSWORD 'your_secure_password';
GRANT ALL PRIVILEGES ON DATABASE scrob TO scrob;
EOF

# Run migrations
cd /opt/scrob
export DATABASE_URL="postgres://scrob:your_secure_password@localhost:5432/scrob"
sqlx migrate run
```

If you don't have sqlx-cli installed:

```bash
psql "$DATABASE_URL" < migrations/001_init.sql
```

#### 4. Create Your User

```bash
cd /opt/scrob
export DATABASE_URL="postgres://scrob:your_secure_password@localhost:5432/scrob"
bash ./scripts/create_user.sh YOUR_USERNAME YOUR_PASSWORD true
```

#### 5. Create Systemd Service

Create `/etc/systemd/system/scrob.service`:

```ini
[Unit]
Description=Scrob Music Scrobble Server
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=YOUR_USERNAME
WorkingDirectory=/opt/scrob
Environment="DATABASE_URL=postgres://scrob:your_secure_password@localhost:5432/scrob"
Environment="HOST=127.0.0.1"
Environment="PORT=3000"
Environment="RUST_LOG=scrob=info"
ExecStart=/opt/scrob/scrob
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Replace `YOUR_USERNAME` with your actual username and update the database password.

#### 6. Start the Service

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable auto-start on boot
sudo systemctl enable scrob

# Start the service
sudo systemctl start scrob

# Check status
sudo systemctl status scrob
```

#### 7. Test It Works

```bash
# Check health endpoint
curl http://localhost:3000/health

# Test login
curl -X POST http://localhost:3000/login \
  -H "Content-Type: application/json" \
  -d '{"username":"YOUR_USERNAME","password":"YOUR_PASSWORD"}'
```

---

## Configuration

### Change Port

Edit `/etc/systemd/system/scrob.service` and change the `PORT` environment
variable, then:

```bash
sudo systemctl daemon-reload
sudo systemctl restart scrob
```

### Allow Network Access

If you want to access from other devices on your local network, change
`HOST=127.0.0.1` to `HOST=0.0.0.0` in the service file.

Warning: Only do this on a trusted network. Consider adding firewall rules.

### Increase Logging

Change `RUST_LOG=scrob=info` to `RUST_LOG=scrob=debug` for more detailed logs.

## Using the Server

### Configure Music Players

Most music players that support Last.fm can be configured to use scrob
instead. You'll need:

- API URL: `http://localhost:3000`
- Your token (get it by logging in via the UI or API)

### Web UI

If using Docker Compose, the UI is automatically available at
`http://localhost:8080`.

For manual deployment:

```bash
cd ~/dev/scrob-ui
nix-shell --run "npm run build"

# Serve the dist/ folder with any static server
# For example, with Python:
cd dist
python3 -m http.server 8080
```

Visit `http://localhost:8080` to access the UI.

## Maintenance

### View Logs

Docker:
```bash
docker-compose logs -f scrob
```

Systemd:
```bash
# Recent logs
sudo journalctl -u scrob -n 50

# Follow logs
sudo journalctl -u scrob -f
```

### Restart Service

Docker:
```bash
docker-compose restart scrob
```

Systemd:
```bash
sudo systemctl restart scrob
```

### Backup Database

Docker:
```bash
# Backup PostgreSQL data
docker-compose exec postgres pg_dump -U scrob scrob > scrob-backup-$(date +%Y%m%d).sql
```

Systemd:
```bash
pg_dump -U scrob scrob > ~/scrob-backup-$(date +%Y%m%d).sql
```

### Update the Server

Docker:
```bash
cd ~/dev/scrob
git pull
docker-compose down
docker-compose build --no-cache
docker-compose up -d
```

Systemd:
```bash
cd ~/dev/scrob
git pull
nix-shell --run "cargo build --release"
sudo systemctl stop scrob
cp target/release/scrob /opt/scrob/
sudo systemctl start scrob
```

### Check Database Size

Docker:
```bash
docker-compose exec postgres psql -U scrob -c "SELECT pg_size_pretty(pg_database_size('scrob'));"
```

Systemd:
```bash
psql -U scrob -c "SELECT pg_size_pretty(pg_database_size('scrob'));"
```

## Troubleshooting

### Service Won't Start

Docker:
```bash
docker-compose logs scrob
```

Systemd:
```bash
sudo journalctl -u scrob -n 100 --no-pager
```

Common issues:
- Database connection failed (check `DATABASE_URL`)
- Port already in use (check with `sudo lsof -i :3000`)
- Migrations not run

### Can't Connect

- Check service is running: `docker-compose ps` or `sudo systemctl status scrob`
- Check port is listening: `sudo lsof -i :3000`
- Check firewall rules if accessing from network

### Database Connection Errors

- Verify PostgreSQL is running
- Check credentials in `DATABASE_URL`
- Ensure database exists: `psql -U scrob -l`
- Check PostgreSQL logs: `docker-compose logs postgres` or `sudo journalctl -u postgresql`

## VPS Deployment

When you're ready to deploy to a VPS:

1. Same Docker Compose setup works great
2. Set up a reverse proxy (Caddy/nginx) for HTTPS
3. Use Let's Encrypt for SSL certificates
4. Point your domain to the VPS
5. Update `VITE_API_URL` in scrob-ui to use your domain
6. Add rate limiting and security headers
7. Use strong PostgreSQL password
8. Consider restricting PostgreSQL to localhost

## Uninstalling

Docker:
```bash
# Stop and remove containers
docker-compose down

# Remove volumes (WARNING: deletes all data!)
docker-compose down -v

# Remove images
docker rmi scrob_scrob scrob_scrob-ui
```

Systemd:
```bash
# Stop and disable service
sudo systemctl stop scrob
sudo systemctl disable scrob

# Remove service file
sudo rm /etc/systemd/system/scrob.service
sudo systemctl daemon-reload

# Remove application
sudo rm -rf /opt/scrob

# Backup database first if you want to keep your data!
```
