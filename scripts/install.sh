#!/usr/bin/env bash
set -e

# Scrob systemd installation script
# This script installs scrob as a systemd service

echo "=== Scrob Installation Script ==="
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then
  echo "Error: Do not run this script as root"
  echo "It will prompt for sudo when needed"
  exit 1
fi

# Check for PostgreSQL
if ! command -v psql &> /dev/null; then
  echo "Error: PostgreSQL not found"
  echo "Please install PostgreSQL first. See POSTGRES_SETUP.md"
  exit 1
fi

# Get database connection info
echo "PostgreSQL Database Setup"
echo "-------------------------"
read -p "Database name [scrob]: " DB_NAME
DB_NAME=${DB_NAME:-scrob}

read -p "Database user [scrob]: " DB_USER
DB_USER=${DB_USER:-scrob}

read -p "Database host [localhost]: " DB_HOST
DB_HOST=${DB_HOST:-localhost}

read -s -p "Database password (empty for peer auth): " DB_PASSWORD
echo ""

if [ -z "$DB_PASSWORD" ]; then
  DATABASE_URL="postgres://$DB_HOST/$DB_NAME"
else
  DATABASE_URL="postgres://$DB_USER:$DB_PASSWORD@$DB_HOST/$DB_NAME"
fi

echo ""
echo "Using DATABASE_URL: ${DATABASE_URL//:*@/:***@}"
echo ""

# Test database connection
echo "Testing database connection..."
if [ -z "$DB_PASSWORD" ]; then
  psql -h "$DB_HOST" -d "$DB_NAME" -c '\q' 2>/dev/null
else
  PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -c '\q' 2>/dev/null
fi

if [ $? -ne 0 ]; then
  echo "Error: Could not connect to database"
  echo "Please check your connection details and ensure the database exists"
  exit 1
fi

echo "✓ Database connection successful"
echo ""

# Build release binary
echo "Building release binary..."
if command -v nix-shell &> /dev/null; then
  nix-shell --run "cargo build --release"
else
  cargo build --release
fi

if [ ! -f "target/release/scrob" ]; then
  echo "Error: Build failed"
  exit 1
fi

echo "✓ Build successful"
echo ""

# Create installation directory
echo "Creating installation directory..."
sudo mkdir -p /opt/scrob
sudo chown $USER:$USER /opt/scrob

# Copy files
echo "Installing files..."
cp target/release/scrob /opt/scrob/
cp -r migrations /opt/scrob/
mkdir -p /opt/scrob/data

echo "✓ Files installed to /opt/scrob"
echo ""

# Run migrations
echo "Running database migrations..."
cd /opt/scrob
export DATABASE_URL="$DATABASE_URL"
if command -v nix-shell &> /dev/null; then
  nix-shell --run "sqlx migrate run" -p sqlx-cli
else
  sqlx migrate run
fi

echo "✓ Migrations complete"
echo ""

# Create systemd service
echo "Creating systemd service..."

SERVICE_FILE="/etc/systemd/system/scrob.service"

sudo tee "$SERVICE_FILE" > /dev/null <<EOF
[Unit]
Description=Scrob Music Scrobble Server
After=network.target postgresql.service

[Service]
Type=simple
User=$USER
WorkingDirectory=/opt/scrob
Environment="DATABASE_URL=$DATABASE_URL"
Environment="HOST=127.0.0.1"
Environment="PORT=3000"
Environment="RUST_LOG=scrob=info"
ExecStart=/opt/scrob/scrob
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

echo "✓ Service file created"
echo ""

# Reload systemd and enable service
sudo systemctl daemon-reload
sudo systemctl enable scrob

echo "✓ Service enabled"
echo ""

# Start service
read -p "Start scrob service now? (Y/n): " START_NOW
START_NOW=${START_NOW:-Y}

if [[ "$START_NOW" =~ ^[Yy]$ ]]; then
  sudo systemctl start scrob
  sleep 2

  if sudo systemctl is-active --quiet scrob; then
    echo "✓ Service started successfully"
    echo ""
    echo "Testing server..."
    if curl -s http://localhost:3000/health > /dev/null; then
      echo "✓ Server is responding"
    else
      echo "⚠ Server not responding yet (may still be starting up)"
    fi
  else
    echo "⚠ Service failed to start"
    echo "Check logs with: sudo journalctl -u scrob -n 50"
    exit 1
  fi
fi

echo ""
echo "=== Installation Complete ==="
echo ""
echo "Next steps:"
echo "  1. Create your user:"
echo "     cd ~/dev/scrob"
echo "     DATABASE_URL='$DATABASE_URL' ./scripts/create_user.sh username password true"
echo ""
echo "  2. Access the server:"
echo "     http://localhost:3000"
echo ""
echo "Useful commands:"
echo "  sudo systemctl status scrob    - Check service status"
echo "  sudo systemctl restart scrob   - Restart service"
echo "  sudo journalctl -u scrob -f    - Follow logs"
echo ""
