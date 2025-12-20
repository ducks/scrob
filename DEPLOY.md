# Deploying scrob to VPS

## Prerequisites

On your VPS, you need:
- Docker (20.10+)
- Docker Compose (2.0+)
- Git

## Installation

### 1. Clone the repository on your VPS

```bash
ssh your-vps
cd ~
git clone https://github.com/yourusername/scrob.git
cd scrob
```

### 2. Create environment file

```bash
cp .env.example .env
nano .env  # Edit and set a secure POSTGRES_PASSWORD
```

### 3. Build and start services

```bash
# Build the Docker image
docker compose -f docker-compose.prod.yml build

# Start services in detached mode
docker compose -f docker-compose.prod.yml up -d

# View logs
docker compose -f docker-compose.prod.yml logs -f
```

### 4. Create your first user

```bash
# Run the bootstrap script inside the container
docker compose -f docker-compose.prod.yml exec scrob-server ./scripts/bootstrap.sh
```

Follow the prompts to create a user and get your API token.

## Management Commands

```bash
# View status
docker compose -f docker-compose.prod.yml ps

# View logs
docker compose -f docker-compose.prod.yml logs -f scrob-server

# Restart services
docker compose -f docker-compose.prod.yml restart

# Stop services
docker compose -f docker-compose.prod.yml down

# Stop and remove data (WARNING: destroys database)
docker compose -f docker-compose.prod.yml down -v
```

## Testing the API

```bash
# Health check
curl http://your-vps-ip:3000/health

# Login (after creating user)
curl -X POST http://your-vps-ip:3000/login \
  -H "Content-Type: application/json" \
  -d '{"username": "youruser", "password": "yourpassword"}'
```

## Updating

```bash
cd ~/scrob
git pull
docker compose -f docker-compose.prod.yml build
docker compose -f docker-compose.prod.yml up -d
```

## Adding HTTPS with nginx

If you want to add a reverse proxy with HTTPS:

```nginx
server {
    listen 80;
    server_name scrob.yourdomain.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Then use certbot to add SSL:
```bash
sudo certbot --nginx -d scrob.yourdomain.com
```

## Firewall

If using UFW:
```bash
# Allow SSH (if not already)
sudo ufw allow 22/tcp

# Allow scrob API
sudo ufw allow 3000/tcp

# Or if using nginx reverse proxy, only allow 80/443
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
```
