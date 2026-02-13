#!/bin/bash
set -e

# RNOT Release Script
# Usage: ./scripts/release.sh <version>

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: ./scripts/release.sh <version>"
    echo "Example: ./scripts/release.sh 0.2.0"
    exit 1
fi

# Validate version format
if ! [[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Version must be in format X.Y.Z"
    exit 1
fi

echo "=========================================="
echo "  RNOT Release v$VERSION"
echo "=========================================="
echo

# Check if working directory is clean
if [ -n "$(git status --porcelain)" ]; then
    echo "Error: Working directory is not clean"
    git status --short
    exit 1
fi

# Run checks
echo "[1/6] Running cargo fmt..."
cargo fmt --all -- --check

echo "[2/6] Running cargo clippy..."
cargo clippy --all-features -- -D warnings

echo "[3/6] Running tests..."
cargo test --all-features

echo "[4/6] Building release..."
cargo build --release

echo "[5/6] Updating version in Cargo.toml..."
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

echo "[6/6] Creating git tag..."
git add Cargo.toml Cargo.lock
git commit -m "Release v$VERSION"
git tag -a "v$VERSION" -m "Release v$VERSION"

echo
echo "=========================================="
echo "  Release v$VERSION ready!"
echo "=========================================="
echo
echo "Next steps:"
echo "  1. Review the changes: git show"
echo "  2. Push to GitHub: git push origin main --tags"
echo "  3. GitHub Actions will build and publish the release"
echo
