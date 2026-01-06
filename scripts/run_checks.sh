#!/bin/bash
set -e

echo "🚀 Starting Beryl Code Checks..."
echo "==================================="

# 1. Format Check
echo "📦 Running cargo fmt..."
cargo fmt --all -- --check
echo "✅ Format check passed"
echo ""

# 2. Clippy (Lint)
echo "🦀 Running cargo clippy..."
cargo clippy --all-targets --all-features -- -D warnings
echo "✅ Clippy check passed"
echo ""

# 3. Tests
echo "🧪 Running tests..."
export RUST_MIN_STACK=8388608
cargo test
echo "✅ Tests passed"
echo ""

# 4. File Size Check
echo "📏 Checking file sizes..."
python3 scripts/check_file_size.py
# check_file_size.py exists 1 on error, so script will stop if it fails
echo "✅ File size check passed"
echo ""

# 5. TODO Check (Informational)
echo "📝 Checking TODOs..."
python3 scripts/check_todos.py
# check_todos.py always exits 0
echo ""

echo "==================================="
echo "🎉 All checks passed! Ready to commit."
