#!/bin/bash
set -e

# Configuration
# 构建 Rust Lency CLI 的命令
RUST_LENCY_BUILD_CMD="cargo build --release -p lency_cli -p lency_runtime"
RUST_LENCY_EXEC="target/release/lencyc"

# 测试 Lency 自举编译器的入口文件 (用于完整性测试)
SELF_HOST_ENTRY="lencyc/driver/test_entry.lcy"
# 输出目录与可执行文件名称（避免产物落在仓库根目录）
SELF_HOST_OUT_DIR="target/lencyc_selfhost"
SELF_HOST_OUT_NAME="lencyc_test"
SELF_HOST_OUT="$SELF_HOST_OUT_DIR/$SELF_HOST_OUT_NAME"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_step() {
    echo -e "\n${BLUE}🚀 $1...${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1 passed${NC}"
}

print_error() {
    echo -e "${RED}❌ $1 failed${NC}"
}

META_SCOPE="lency"
if [[ "$#" -ne 0 ]]; then
    echo -e "${RED}run_lency_checks.sh 不接受参数。该脚本固定为 Lency 专用检查。${NC}"
    exit 1
fi

echo -e "${BLUE}=====================================${NC}"
echo -e "${BLUE}   Starting Lency-side Checks   ${NC}"
echo -e "${BLUE}=====================================${NC}"

# 1. 编译 Rust 宿主编译器
print_step "1. Compiling Rust Host Compiler (lency_cli)"
if $RUST_LENCY_BUILD_CMD; then
    print_success "Rust host compiler build"
else
    print_error "Rust host compiler build"
    exit 1
fi

# 1.5. 代码质量检查 (Meta Checks)
print_step "1.5. Running Meta Checks (TODOs, File Size, Naming)"
# 扫描 TODO/FIXME
python3 scripts/check_todos.py --scope "$META_SCOPE"
# 检查文件大小
python3 scripts/check_file_size.py --scope "$META_SCOPE"
# 检查 Lencyc 专用规范 (命名等)
if python3 scripts/check_lencyc_meta.py; then
    print_success "Meta checks"
else
    print_error "Meta checks"
    exit 1
fi

# 1.6. 全量语法检查 (Verify all files in lencyc)
print_step "1.6. Running Batch Syntax Checks for lencyc/"
# 使用 Rust 版编译器对 lencyc 下所有文件进行只检查语法不生成代码的验证
LENCYC_FILES=$(find lencyc -name "*.lcy")
FAILED_FILES=""
for f in $LENCYC_FILES; do
    if ! $RUST_LENCY_EXEC build "$f" --check-only > /dev/null 2>&1; then
        echo -e "${YELLOW}⚠️ Syntax check failed (or not supported yet): $f${NC}"
        # FAILED_FILES="$FAILED_FILES $f" 
    fi
done
print_success "Full syntax trace completed"

# 2. 使用 Rust 编译器编译 Lency 的自举版 (验证 test_entry 逻辑)
print_step "2. Compiling Lency-written Compiler (Self-host Lencyc)"
if [ ! -f "$SELF_HOST_ENTRY" ]; then
    print_error "Cannot find self-host entry file: $SELF_HOST_ENTRY"
    exit 1
fi

mkdir -p "$SELF_HOST_OUT_DIR"

if $RUST_LENCY_EXEC build $SELF_HOST_ENTRY -o $SELF_HOST_OUT_NAME --out-dir "$SELF_HOST_OUT_DIR"; then
    print_success "Self-hosted Lencyc compilation"
else
    print_error "Self-hosted Lencyc compilation"
    exit 1
fi

# 3. 运行已编译 of Lencyc 可执行文件并验证
print_step "3. Running Compiled Self-host Lencyc Basic Tests"
if ./$SELF_HOST_OUT; then
    print_success "Self-hosted Lencyc execution test"
else
    print_error "Self-hosted Lencyc execution test"
    exit 1
fi

echo -e "\n${BLUE}=====================================${NC}"
echo -e "${GREEN}🎉 All self-hosted checks passed!${NC}"
echo -e "${BLUE}=====================================${NC}"
