#!/bin/bash
set -e

if [ -z "$1" ] || [ -z "$2" ]; then
  echo "Usage: $0 <username> <password> [is_admin]"
  echo "Example: $0 alice mypassword true"
  exit 1
fi

USERNAME="$1"
PASSWORD="$2"
IS_ADMIN="${3:-false}"

# Hash the password using bcrypt
HASH=$(python3 -c "import bcrypt; print(bcrypt.hashpw(b'$PASSWORD', bcrypt.gensalt()).decode('utf-8'))")

TIMESTAMP=$(date +%s)

# Insert into database
DATABASE_URL="${DATABASE_URL:-postgres://scrob:scrob_password_change_me@localhost:5432/scrob}"

psql "$DATABASE_URL" <<EOF
INSERT INTO users (username, password_hash, is_admin, created_at)
VALUES ('$USERNAME', '$HASH', $IS_ADMIN, $TIMESTAMP);
EOF

echo "User '$USERNAME' created successfully (admin: $IS_ADMIN)"
