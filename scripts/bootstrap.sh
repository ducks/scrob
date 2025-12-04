#!/bin/bash
set -e

# Bootstrap script: Creates first user, logs in, and optionally creates API token

GRAPHQL_URL="${GRAPHQL_URL:-http://localhost:3000/graphql}"
DATABASE_URL="${DATABASE_URL:-sqlite:./data/scrob.db}"

echo "=== Scrob Bootstrap Script ==="
echo ""

# Get username
read -p "Enter username: " USERNAME
if [ -z "$USERNAME" ]; then
  echo "Error: Username cannot be empty"
  exit 1
fi

# Get password
read -s -p "Enter password: " PASSWORD
echo ""
if [ -z "$PASSWORD" ]; then
  echo "Error: Password cannot be empty"
  exit 1
fi

# Ask if admin
read -p "Make this user an admin? (y/N): " IS_ADMIN_INPUT
IS_ADMIN=0
if [[ "$IS_ADMIN_INPUT" =~ ^[Yy]$ ]]; then
  IS_ADMIN=1
fi

echo ""
echo "Creating user '$USERNAME'..."

# Hash the password using Python
if ! command -v python3 &> /dev/null; then
  echo "Error: python3 is required but not installed"
  exit 1
fi

HASH=$(python3 -c "import bcrypt; print(bcrypt.hashpw(b'$PASSWORD', bcrypt.gensalt()).decode('utf-8'))" 2>/dev/null)
if [ $? -ne 0 ]; then
  echo "Error: Failed to hash password. Is bcrypt installed? (pip install bcrypt)"
  exit 1
fi

TIMESTAMP=$(date +%s)

# Insert into database
DB_PATH="${DATABASE_URL#sqlite:}"
if [ ! -f "$DB_PATH" ]; then
  echo "Error: Database not found at $DB_PATH"
  echo "Make sure the server has been started at least once to create the database"
  exit 1
fi

sqlite3 "$DB_PATH" <<EOF
INSERT INTO users (username, password_hash, is_admin, created_at)
VALUES ('$USERNAME', '$HASH', $IS_ADMIN, $TIMESTAMP);
EOF

if [ $? -ne 0 ]; then
  echo "Error: Failed to create user (username might already exist)"
  exit 1
fi

echo "✓ User created successfully"
echo ""

# Login to get token
echo "Logging in to get API token..."

if ! command -v curl &> /dev/null; then
  echo "Error: curl is required but not installed"
  exit 1
fi

# GraphQL login mutation
LOGIN_QUERY=$(cat <<EOF
{
  "query": "mutation Login(\$username: String!, \$password: String!) { login(username: \$username, password: \$password) { token user { username isAdmin } } }",
  "variables": {
    "username": "$USERNAME",
    "password": "$PASSWORD"
  }
}
EOF
)

RESPONSE=$(curl -s -X POST "$GRAPHQL_URL" \
  -H "Content-Type: application/json" \
  -d "$LOGIN_QUERY")

# Extract token using Python
TOKEN=$(echo "$RESPONSE" | python3 -c "import sys, json; data = json.load(sys.stdin); print(data.get('data', {}).get('login', {}).get('token', ''))" 2>/dev/null)

if [ -z "$TOKEN" ]; then
  echo "Error: Login failed"
  echo "Response: $RESPONSE"
  exit 1
fi

echo "✓ Login successful"
echo ""
echo "Your session token:"
echo "$TOKEN"
echo ""

# Ask if they want to create an API token for a client
read -p "Create an API token for a client? (y/N): " CREATE_CLIENT_TOKEN

if [[ "$CREATE_CLIENT_TOKEN" =~ ^[Yy]$ ]]; then
  read -p "Enter a label for this token (e.g., 'music-player'): " TOKEN_LABEL

  if [ -z "$TOKEN_LABEL" ]; then
    TOKEN_LABEL="api-token"
  fi

  echo "Creating API token..."

  CREATE_TOKEN_QUERY=$(cat <<EOF
{
  "query": "mutation CreateToken(\$label: String) { createApiToken(label: \$label) { id label token } }",
  "variables": {
    "label": "$TOKEN_LABEL"
  }
}
EOF
)

  API_RESPONSE=$(curl -s -X POST "$GRAPHQL_URL" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $TOKEN" \
    -d "$CREATE_TOKEN_QUERY")

  API_TOKEN=$(echo "$API_RESPONSE" | python3 -c "import sys, json; data = json.load(sys.stdin); print(data.get('data', {}).get('createApiToken', {}).get('token', ''))" 2>/dev/null)

  if [ -z "$API_TOKEN" ]; then
    echo "Error: Failed to create API token"
    echo "Response: $API_RESPONSE"
  else
    echo "✓ API token created successfully"
    echo ""
    echo "API Token for '$TOKEN_LABEL':"
    echo "$API_TOKEN"
    echo ""
    echo "IMPORTANT: Save this token securely. It won't be shown again."
  fi
fi

echo ""
echo "=== Bootstrap Complete ==="
echo ""
echo "Next steps:"
echo "  - Use the session token to access GraphQL Playground"
echo "  - Visit: ${GRAPHQL_URL%/graphql}/playground"
echo "  - Add header: {\"Authorization\": \"Bearer $TOKEN\"}"
echo ""
if [ -n "$API_TOKEN" ]; then
  echo "  - Configure your music player with the API token"
  echo ""
fi
