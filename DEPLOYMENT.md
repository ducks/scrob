# Deployment Guide

## Quick Start with Docker (Recommended)

The easiest way to run scrob is with Docker Compose, which handles everything
automatically.

### Prerequisites

- Docker and Docker Compose installed
- PostgreSQL database (see below)

### Setup

1. **Set up PostgreSQL**

Use Docker Compose to run both scrob and Postgres:

```bash
cd ~/dev/scrob

# Create docker-compose.yml (see example below)
# Edit environment variables as needed
docker-compose up -d
```

Example `docker-compose.yml`:

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: scrob
      POSTGRES_USER: scrob
      POSTGRES_PASSWORD: your_secure_password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  scrob:
    build: .
    environment:
      DATABASE_URL: postgres://scrob:your_secure_password@postgres:5432/scrob
      HOST: 0.0.0.0
      PORT: 3000
      RUST_LOG: scrob=info
    ports:
      - "3000:3000"
    depends_on:
      - postgres

volumes:
  postgres_data:
```

2. **Create your user**

```bash
# Get a shell in the scrob container
docker-compose exec scrob /bin/bash

# Run the create user script
./scripts/create_user.sh your_username your_password true
```

3. **Access scrob**

Visit `http://localhost:3000` and login with your credentials.

That's it! Docker handles building, migrations, and running the service.

---

## Manual Installation (Alternative)

If you prefer to run without Docker, use the automated setup script:

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

### 1. Build the Server

```bash
cd ~/dev/scrob
nix-shell --run "cargo build --release"
```

The binary will be at `target/release/scrob`.

### 2. Choose Installation Location

```bash
# Create application directory
sudo mkdir -p /opt/scrob
sudo chown $USER:$USER /opt/scrob

# Copy binary
cp target/release/scrob /opt/scrob/

# Create data directory
mkdir -p /opt/scrob/data

# Copy migrations
cp -r migrations /opt/scrob/
```

### 3. Set Up Database

```bash
cd /opt/scrob
export DATABASE_URL="sqlite:./data/scrob.db"

# Run migrations (requires sqlx-cli)
sqlx migrate run
```

If you don't have sqlx-cli installed:

```bash
# Create empty database
touch /opt/scrob/data/scrob.db

# Run migrations manually
sqlite3 /opt/scrob/data/scrob.db < migrations/001_init.sql
```

### 4. Create Your User

```bash
cd ~/dev/scrob
nix-shell --run "DATABASE_URL=sqlite:/opt/scrob/data/scrob.db bash ./scripts/create_user.sh YOUR_USERNAME YOUR_PASSWORD true"
```

### 5. Create Systemd Service

Create `/etc/systemd/system/scrob.service`:

```ini
[Unit]
Description=Scrob Music Scrobble Server
After=network.target

[Service]
Type=simple
User=YOUR_USERNAME
WorkingDirectory=/opt/scrob
Environment="DATABASE_URL=sqlite:/opt/scrob/data/scrob.db"
Environment="HOST=127.0.0.1"
Environment="PORT=3000"
Environment="RUST_LOG=scrob=info"
ExecStart=/opt/scrob/scrob
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Replace `YOUR_USERNAME` with your actual username.

### 6. Start the Service

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

### 7. Test It Works

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

If you've built the UI:

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

```bash
# Recent logs
sudo journalctl -u scrob -n 50

# Follow logs
sudo journalctl -u scrob -f
```

### Restart Service

```bash
sudo systemctl restart scrob
```

### Backup Database

```bash
# Simple backup
cp /opt/scrob/data/scrob.db ~/scrob-backup-$(date +%Y%m%d).db

# Or with systemd timer for automatic backups
```

### Update the Server

```bash
cd ~/dev/scrob
git pull
nix-shell --run "cargo build --release"
sudo systemctl stop scrob
cp target/release/scrob /opt/scrob/
sudo systemctl start scrob
```

### Check Database Size

```bash
du -h /opt/scrob/data/scrob.db
```

SQLite is very efficient. Even with thousands of scrobbles, the database
should stay under a few MB.

## Troubleshooting

### Service Won't Start

Check logs:
```bash
sudo journalctl -u scrob -n 100 --no-pager
```

Common issues:
- Database file permissions (should be owned by your user)
- Port already in use (check with `sudo lsof -i :3000`)
- Migrations not run

### Can't Connect

- Check service is running: `sudo systemctl status scrob`
- Check port is listening: `sudo lsof -i :3000`
- Check firewall rules if accessing from network

### Database Locked Errors

SQLite doesn't handle concurrent writes well. If you see these:
- Make sure only one scrob process is running
- For multiple users, consider migrating to Postgres

## Future: VPS Deployment

When you're ready to deploy to a VPS:

1. Same systemd setup, but use a reverse proxy (Caddy/nginx)
2. Set up HTTPS with Let's Encrypt
3. Point your domain to the VPS
4. Consider Postgres for better concurrent write handling
5. Add rate limiting and security headers

For now, local deployment is simpler and works great for personal use.

## Uninstalling

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
