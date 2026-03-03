#!/bin/bash
set -e

# Configuration
CARGO_FLAGS="--all-targets --all-features"
CLIPPY_FLAGS="-D warnings"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper Functions
print_step() {
    echo -e "\n${BLUE}🚀 $1...${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1 passed${NC}"
}

print_error() {
    echo -e "${RED}❌ $1 failed${NC}"
}

RUST_SCOPE="rust"
if [[ "$#" -ne 0 ]]; then
    echo -e "${RED}run_checks.sh 不接受参数。该脚本固定为 Rust 专用检查。${NC}"
    exit 1
fi

echo -e "${BLUE}===================================${NC}"
echo -e "${BLUE}   Starting Rust-side Checks   ${NC}"
echo -e "${BLUE}===================================${NC}"

# 1. Format Check
print_step "Checking code formatting"
if cargo fmt --all -- --check; then
    print_success "Format check"
else
    print_error "Format check"
    echo -e "${YELLOW}Run 'cargo fmt' to fix this.${NC}"
    exit 1
fi

# 2. Clippy (Lint)
print_step "Running Clippy lints"
if cargo clippy $CARGO_FLAGS -- $CLIPPY_FLAGS; then
    print_success "Clippy check"
else
    print_error "Clippy check"
    exit 1
fi

# 3. Tests
print_step "Running Unit Tests"
export RUST_MIN_STACK=8388608
if cargo test; then
    print_success "Unit tests"
else
    print_error "Unit tests"
    exit 1
fi

# 4. File Size Check (Rust scope)
print_step "Checking File Sizes"
if python3 scripts/check_file_size.py --scope "$RUST_SCOPE"; then
    print_success "File size check"
else
    print_error "File size check"
    exit 1
fi

# 5. TODO Check (Rust scope)
print_step "Scanning for TODOs"
python3 scripts/check_todos.py --scope "$RUST_SCOPE"
print_success "TODO scan completed"

# 6. Code Quality (Rust scope)
print_step "Checking Code Quality"
if python3 scripts/check_banned_patterns.py --scope "$RUST_SCOPE"; then
   print_success "Quality check"
else
   print_error "Quality check"
   echo -e "${YELLOW}Please remove banned patterns (unwrap/println) or add exemptions.${NC}"
   exit 1
fi

echo -e "\n${BLUE}===================================${NC}"
echo -e "${GREEN}🎉 All checks passed! Ready to commit.${NC}"
echo -e "${BLUE}===================================${NC}"
