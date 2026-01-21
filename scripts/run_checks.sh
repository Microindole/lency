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

# Default Steps
RUN_FMT=true
RUN_CLIPPY=true
RUN_TEST=true
RUN_LCY_TEST=true
RUN_FILE_SIZE=true
RUN_TODO=true
RUN_EDITOR=true

# Parse Arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --fast|--quick)
            echo -e "${YELLOW}⚡ Quick mode enabled: Skipping integration tests and file size checks${NC}"
            RUN_LCY_TEST=false
            RUN_FILE_SIZE=false
            ;;
        --fmt)
            RUN_FMT=true; RUN_CLIPPY=false; RUN_TEST=false; RUN_LCY_TEST=false; RUN_FILE_SIZE=false; RUN_TODO=false
            ;;
        --clippy)
            RUN_FMT=false; RUN_CLIPPY=true; RUN_TEST=false; RUN_LCY_TEST=false; RUN_FILE_SIZE=false; RUN_TODO=false
            ;;
        --test)
             RUN_FMT=false; RUN_CLIPPY=false; RUN_TEST=true; RUN_LCY_TEST=false; RUN_FILE_SIZE=false; RUN_TODO=false
            ;;
        --lcy)
             RUN_FMT=false; RUN_CLIPPY=false; RUN_TEST=false; RUN_LCY_TEST=true; RUN_FILE_SIZE=false; RUN_TODO=false
             ;;
        -h|--help)
            echo "Usage: ./run_checks.sh [OPTIONS]"
            echo "Options:"
            echo "  --fast, --quick   Skip integration tests and file size checks"
            echo "  --fmt             Run only cargo fmt"
            echo "  --clippy          Run only cargo clippy"
            echo "  --test            Run only unit tests"
            echo "  --lcy             Run only .lcy integration tests"
            echo "  -h, --help        Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown parameter passed: $1${NC}"
            exit 1
            ;;
    esac
    shift
done

echo -e "${BLUE}===================================${NC}"
echo -e "${BLUE}   Starting Lency Code Checks   ${NC}"
echo -e "${BLUE}===================================${NC}"

# 1. Format Check
if [ "$RUN_FMT" = true ]; then
    print_step "Checking code formatting"
    if cargo fmt --all -- --check; then
        print_success "Format check"
    else
        print_error "Format check"
        echo -e "${YELLOW}Run 'cargo fmt' to fix this.${NC}"
        exit 1
    fi
fi

# 2. Clippy (Lint)
if [ "$RUN_CLIPPY" = true ]; then
    print_step "Running Clippy lints"
    if cargo clippy $CARGO_FLAGS -- $CLIPPY_FLAGS; then
        print_success "Clippy check"
    else
        print_error "Clippy check"
        exit 1
    fi
fi

# 3. Tests
if [ "$RUN_TEST" = true ]; then
    print_step "Running Unit Tests"
    export RUST_MIN_STACK=8388608
    if cargo test; then
        print_success "Unit tests"
    else
        print_error "Unit tests"
        exit 1
    fi
fi

# 4. Lcy Integration Tests
if [ "$RUN_LCY_TEST" = true ]; then
    print_step "Running .lcy Integration Tests"
    if bash scripts/run_lcy_tests.sh; then
        print_success "Lcy tests"
    else
        print_error "Lcy tests"
        exit 1
    fi
fi

# 5. File Size Check
if [ "$RUN_FILE_SIZE" = true ]; then
    print_step "Checking File Sizes"
    if python3 scripts/check_file_size.py; then
        print_success "File size check"
    else
        print_error "File size check"
        exit 1
    fi
fi

# 6. TODO Check (Informational)
if [ "$RUN_TODO" = true ]; then
    print_step "Scanning for TODOs"
    python3 scripts/check_todos.py
    # check_todos.py always exits 0 usually, but let's just mark it done
    print_success "TODO scan completed"
fi

# 7. Code Quality (Banned Patterns)
# Helps catch unwrap, panic, and printing in library code
print_step "Checking Code Quality"
if python3 scripts/check_banned_patterns.py; then
   print_success "Quality check"
else
   print_error "Quality check"
   echo -e "${YELLOW}Please remove banned patterns (unwrap/println) or add exemptions.${NC}"
   exit 1
fi

# 8. Editor Extension Check
if [ "$RUN_EDITOR" = true ]; then
    print_step "Checking Editor Extension"
    if bash scripts/check_editor.sh; then
        print_success "Editor extension check"
    else
        print_error "Editor extension check"
        exit 1
    fi
fi

echo -e "\n${BLUE}===================================${NC}"
echo -e "${GREEN}🎉 All checks passed! Ready to commit.${NC}"
echo -e "${BLUE}===================================${NC}"
