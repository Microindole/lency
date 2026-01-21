#!/bin/bash

# Lency IDE 开发模式启动脚本 (V6 Professional 版)
# 集成了自动构建流程。

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
EXT_PATH="$ROOT_DIR/editors/vscode"

# 1. 尝试自动编译 TypeScript (如果环境支持)
if command -v npm >/dev/null 2>&1; then
    echo "📦 正在编译扩展源码..."
    cd "$EXT_PATH" && npm install --silent && npm run build --silent
    cd "$ROOT_DIR"
fi

# 2. 检查编译产物
if [ ! -f "$EXT_PATH/dist/extension.js" ]; then
    echo "⚠️ 警告: 未找到编译产物 ($EXT_PATH/dist/extension.js)。"
    echo "如果是通过 Antigravity 运行，请确保您在主窗口运行了 'npm run build'。"
fi

# 3. 确定编辑器命令
if command -v antigravity >/dev/null 2>&1; then
    IDE_CMD="antigravity"
elif command -v cursor >/dev/null 2>&1; then
    IDE_CMD="cursor"
elif command -v code >/dev/null 2>&1; then
    IDE_CMD="code"
else
    echo "❌ 错误: 未找到 IDE 命令。"
    exit 1
fi

echo "🚀 正在以 Professional 模式启动 $IDE_CMD..."
$IDE_CMD --extensionDevelopmentPath "$EXT_PATH" "$ROOT_DIR"
