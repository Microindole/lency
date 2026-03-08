#!/bin/bash
# 运行所有 .lcy 集成测试
# 此脚本用于验证语言特性没有在修复 bug 时被破坏

set -e

echo "🧪 Running .lcy integration tests..."
echo "====================================="

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

PASS=0
FAIL=0
EXPECTED_FAIL=0
EXPECTED_TODO=0
EXPECTED_FIXME=0
FAILED_FILES=()

# 查找所有 .lcy 文件
LCY_FILES=$(find "$PROJECT_ROOT/tests/integration" -name "*.lcy" | sort)

if [ -z "$LCY_FILES" ]; then
    echo "⚠️  No .lcy files found in tests/integration"
    exit 0
fi

echo ""

for file in $LCY_FILES; do
    rel_path="${file#$PROJECT_ROOT/}"
    
    # 检查文件是否包含 @expect-error 注释
    first_lines=$(head -5 "$file")
    is_expected_fail=0
    is_todo=0
    is_fixme=0
    
    if echo "$first_lines" | grep -q "@expect-error"; then
        is_expected_fail=1
        if echo "$first_lines" | grep -q "@expect-error:.*TODO"; then
            is_todo=1
        elif echo "$first_lines" | grep -q "@expect-error:.*FIXME"; then
            is_fixme=1
        fi
    fi
    
    # 使用 lencyc check 进行语义检查
    if cargo run --bin lencyc --quiet -- check "$file" > /dev/null 2>&1; then
        if [ $is_expected_fail -eq 1 ]; then
            # 预期失败但实际通过了 - 这可能意味着测试需要更新
            echo "⚠️  $rel_path (expected to fail but passed)"
            ((PASS++)) || true
        else
            echo "✅ $rel_path"
            ((PASS++)) || true
        fi
    else
        if [ $is_expected_fail -eq 1 ]; then
            if [ $is_todo -eq 1 ]; then
                echo "📋 $rel_path (TODO: 功能未实现)"
                ((EXPECTED_TODO++)) || true
            elif [ $is_fixme -eq 1 ]; then
                echo "🐛 $rel_path (FIXME: 需要修复)"
                ((EXPECTED_FIXME++)) || true
            else
                echo "🔶 $rel_path (expected failure)"
                ((EXPECTED_FAIL++)) || true
            fi
        else
            echo "❌ $rel_path"
            FAILED_FILES+=("$rel_path")
            ((FAIL++)) || true
        fi
    fi
done

echo ""
echo "====================================="

# 计算总预期失败数
TOTAL_EXPECTED=$((EXPECTED_FAIL + EXPECTED_TODO + EXPECTED_FIXME))

echo "📊 Results:"
echo "   ✅ Passed: $PASS"
echo "   🔶 Expected errors: $EXPECTED_FAIL"
echo "   📋 TODO (功能未实现): $EXPECTED_TODO"
echo "   🐛 FIXME (需要修复): $EXPECTED_FIXME"
echo "   ❌ Unexpected failures: $FAIL"

# 在 GitHub Actions 中输出警告（如果有 TODO/FIXME）
if [ -n "$GITHUB_ACTIONS" ]; then
    if [ $EXPECTED_TODO -gt 0 ]; then
        echo "::warning::有 $EXPECTED_TODO 个测试因功能未实现而跳过"
    fi
    if [ $EXPECTED_FIXME -gt 0 ]; then
        echo "::warning::有 $EXPECTED_FIXME 个测试因编译器 bug 而跳过，需要修复"
    fi
fi

if [ $FAIL -gt 0 ]; then
    echo ""
    echo "❌ Unexpected failures:"
    for f in "${FAILED_FILES[@]}"; do
        echo "   - $f"
    done
    exit 1
fi

echo ""
echo "✅ All tests passed!"
