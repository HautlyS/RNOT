#!/bin/bash
set -e

echo "=== RNOT Code Quality Verification ==="
echo

echo "1. Running cargo fmt check..."
cargo fmt --all --check
echo "✓ Code formatting is correct"
echo

echo "2. Running cargo clippy (strict mode)..."
cargo clippy --all-targets --all-features -- -D warnings
echo "✓ No clippy warnings"
echo

echo "3. Running cargo test..."
cargo test --quiet
echo "✓ All tests pass"
echo

echo "4. Building release binary..."
cargo build --release --quiet
echo "✓ Release build successful"
echo

echo "5. Checking binary size..."
BINARY_SIZE=$(du -h target/release/rnot | cut -f1)
echo "Binary size: $BINARY_SIZE"
echo

echo "6. Running integration tests..."
./test.sh > /dev/null 2>&1
echo "✓ Integration tests pass"
echo

echo "=== All quality checks passed! ==="
echo
echo "Summary:"
echo "  - Zero clippy warnings"
echo "  - Zero formatting issues"
echo "  - All tests passing"
echo "  - Release build successful"
echo "  - Integration tests passing"
