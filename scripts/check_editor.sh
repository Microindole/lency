#!/bin/bash
# Lency Editor Extension CI Check
# 验证插件的语法、类型及构建稳定性。

set -e

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
ROOT_DIR=$(cd "$SCRIPT_DIR/.." && pwd)
EXT_DIR="$ROOT_DIR/editors/vscode"

echo "🔍 Starting Editor Extension Checks..."

if [ ! -d "$EXT_DIR" ]; then
    echo "❌ Error: Editor directory not found."
    exit 1
fi

cd "$EXT_DIR"

# 1. 检查 Node.js 环境
if ! command -v npm >/dev/null 2>&1; then
    echo "⚠️ Skip: npm not found, skipping editor build check."
    exit 0
fi

echo "📦 Installing dependencies..."
npm install --silent

echo "🏗️ Building extension..."
npm run build

echo "✅ Editor extension check passed!"
