#!/bin/bash

# XQPath 快速验证脚本
# 这是一个便捷的入口点，调用实际的验证脚本

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🔍 运行 XQPath 调试功能验证..."
echo "项目路径: $PROJECT_ROOT"
echo

# 确保脚本具有执行权限
chmod +x "$SCRIPT_DIR/verify-debug.sh"

# 运行验证脚本
cd "$PROJECT_ROOT"
exec "$SCRIPT_DIR/verify-debug.sh"
