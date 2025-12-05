#!/usr/bin/env bash
set -e

# Seed test data: Creates a test user and adds sample scrobbles

DATABASE_URL="${DATABASE_URL:-sqlite:./data/scrob.db}"
API_URL="${API_URL:-http://localhost:3000}"

echo "=== Scrob Test Data Seeder ==="
echo ""

# Check if database exists
DB_PATH="${DATABASE_URL#sqlite:}"
if [ ! -f "$DB_PATH" ]; then
  echo "Error: Database not found at $DB_PATH"
  echo "Run 'cargo sqlx migrate run' first"
  exit 1
fi

# Create test user
USERNAME="test"
PASSWORD="test123"
IS_ADMIN=1

echo "Creating test user '$USERNAME'..."

# Check if user already exists
USER_EXISTS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM users WHERE username = '$USERNAME';" 2>/dev/null || echo "0")

if [ "$USER_EXISTS" -gt 0 ]; then
  echo "User '$USERNAME' already exists, skipping creation"
else
  HASH=$(python3 -c "import bcrypt; print(bcrypt.hashpw(b'$PASSWORD', bcrypt.gensalt()).decode('utf-8'))")
  TIMESTAMP=$(date +%s)

  sqlite3 "$DB_PATH" <<EOF
INSERT INTO users (username, password_hash, is_admin, created_at)
VALUES ('$USERNAME', '$HASH', $IS_ADMIN, $TIMESTAMP);
EOF

  echo "✓ User '$USERNAME' created (password: $PASSWORD)"
fi

echo ""
echo "Logging in to get token..."

# Login to get token
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\"}")

TOKEN=$(echo "$LOGIN_RESPONSE" | python3 -c "import sys, json; data = json.load(sys.stdin); print(data.get('token', ''))" 2>/dev/null)

if [ -z "$TOKEN" ]; then
  echo "Error: Failed to login"
  echo "Response: $LOGIN_RESPONSE"
  exit 1
fi

echo "✓ Logged in successfully"
echo ""
echo "Adding sample scrobbles..."

# Sample scrobbles data
NOW=$(date +%s)
ONE_HOUR=3600
ONE_DAY=86400

# Build a batch of scrobbles
SCROBBLES='[
  {
    "artist": "Kendrick Lamar",
    "track": "Wesley'\''s Theory",
    "album": "To Pimp a Butterfly",
    "duration": 287,
    "timestamp": '$((NOW - 1 * ONE_HOUR))'
  },
  {
    "artist": "Thundercat",
    "track": "Them Changes",
    "album": "Drunk",
    "duration": 194,
    "timestamp": '$((NOW - 2 * ONE_HOUR))'
  },
  {
    "artist": "Todd Snider",
    "track": "Beer Run",
    "album": "Near Truths and Hotel Rooms",
    "duration": 212,
    "timestamp": '$((NOW - 3 * ONE_HOUR))'
  },
  {
    "artist": "Sierra Ferrell",
    "track": "In Dreams",
    "album": "Long Time Coming",
    "duration": 201,
    "timestamp": '$((NOW - 5 * ONE_HOUR))'
  },
  {
    "artist": "Kendrick Lamar",
    "track": "King Kunta",
    "album": "To Pimp a Butterfly",
    "duration": 234,
    "timestamp": '$((NOW - 1 * ONE_DAY))'
  },
  {
    "artist": "Thundercat",
    "track": "Dragonball Durag",
    "album": "It Is What It Is",
    "duration": 177,
    "timestamp": '$((NOW - 1 * ONE_DAY - ONE_HOUR))'
  },
  {
    "artist": "Todd Snider",
    "track": "Conservative Christian, Right-Wing, Republican, Straight, White, American Males",
    "album": "Step Right Up",
    "duration": 259,
    "timestamp": '$((NOW - 2 * ONE_DAY))'
  },
  {
    "artist": "Sierra Ferrell",
    "track": "Jeremiah",
    "album": "Long Time Coming",
    "duration": 199,
    "timestamp": '$((NOW - 2 * ONE_DAY - ONE_HOUR))'
  },
  {
    "artist": "Kendrick Lamar",
    "track": "Alright",
    "album": "To Pimp a Butterfly",
    "duration": 219,
    "timestamp": '$((NOW - 3 * ONE_DAY))'
  },
  {
    "artist": "Thundercat",
    "track": "Show You the Way",
    "album": "Drunk",
    "duration": 267,
    "timestamp": '$((NOW - 3 * ONE_DAY - ONE_HOUR))'
  },
  {
    "artist": "Todd Snider",
    "track": "Tillamook County Jail",
    "album": "East Nashville Skyline",
    "duration": 248,
    "timestamp": '$((NOW - 4 * ONE_DAY))'
  },
  {
    "artist": "Sierra Ferrell",
    "track": "West Virginia Waltz",
    "album": "Long Time Coming",
    "duration": 186,
    "timestamp": '$((NOW - 5 * ONE_DAY))'
  },
  {
    "artist": "Kendrick Lamar",
    "track": "HUMBLE.",
    "album": "DAMN.",
    "duration": 177,
    "timestamp": '$((NOW - 6 * ONE_DAY))'
  },
  {
    "artist": "Thundercat",
    "track": "Funny Thing",
    "album": "Drunk",
    "duration": 151,
    "timestamp": '$((NOW - 7 * ONE_DAY))'
  },
  {
    "artist": "Todd Snider",
    "track": "Play a Train Song",
    "album": "The Devil You Know",
    "duration": 234,
    "timestamp": '$((NOW - 8 * ONE_DAY))'
  },
  {
    "artist": "Sierra Ferrell",
    "track": "Made Like That",
    "album": "Long Time Coming",
    "duration": 192,
    "timestamp": '$((NOW - 9 * ONE_DAY))'
  }
]'

RESPONSE=$(curl -s -X POST "$API_URL/scrob" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "$SCROBBLES")

SCROB_COUNT=$(echo "$RESPONSE" | python3 -c "import sys, json; data = json.load(sys.stdin); print(len(data))" 2>/dev/null || echo "0")

if [ "$SCROB_COUNT" -gt 0 ]; then
  echo "✓ Added $SCROB_COUNT scrobbles"
else
  echo "Error: Failed to add scrobbles"
  echo "Response: $RESPONSE"
  exit 1
fi

echo ""
echo "=== Test Data Seeded Successfully ==="
echo ""
echo "You can now login with:"
echo "  Username: $USERNAME"
echo "  Password: $PASSWORD"
echo ""
echo "Or use this token for API requests:"
echo "  $TOKEN"
echo ""
