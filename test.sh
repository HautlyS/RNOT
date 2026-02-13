#!/bin/bash
set -e

echo "=== RNOT Test Suite ==="
echo

# Test 1: Version
echo "Test 1: Version check"
./target/release/rnot --version
echo "✓ Version check passed"
echo

# Test 2: Status (no config)
echo "Test 2: Status check"
./target/release/rnot status
echo "✓ Status check passed"
echo

# Test 3: Add site
echo "Test 3: Add site"
./target/release/rnot add https://news.ycombinator.com --name "Hacker News" --selector ".storylink"
echo "✓ Add site passed"
echo

# Test 4: List sites
echo "Test 4: List sites"
./target/release/rnot list
echo "✓ List sites passed"
echo

# Test 5: Set token
echo "Test 5: Set encrypted token"
./target/release/rnot set-token "test_bot_token_123456"
echo "✓ Set token passed"
echo

# Test 6: Verify encryption
echo "Test 6: Verify token is encrypted"
TOKEN_FILE="$HOME/.config/rnot/.token"
if [ -f "$TOKEN_FILE" ]; then
    TOKEN_CONTENT=$(cat "$TOKEN_FILE")
    if [[ "$TOKEN_CONTENT" != *"test_bot_token"* ]]; then
        echo "✓ Token is encrypted (not plaintext)"
    else
        echo "✗ Token is NOT encrypted!"
        exit 1
    fi
else
    echo "✗ Token file not found!"
    exit 1
fi
echo

# Test 7: Status with token
echo "Test 7: Status with token set"
./target/release/rnot status
echo "✓ Status with token passed"
echo

# Test 8: Remove site
echo "Test 8: Remove site"
SITE_ID=$(./target/release/rnot list | grep -oP '\[\K[a-f0-9]+(?=\])' | head -1)
./target/release/rnot remove "$SITE_ID"
echo "✓ Remove site passed"
echo

# Test 9: Clear token
echo "Test 9: Clear token"
./target/release/rnot clear-token
echo "✓ Clear token passed"
echo

echo "=== All tests passed! ==="
