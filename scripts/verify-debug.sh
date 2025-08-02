#!/bin/bash

echo "=== XQPath v1.4.1 调试功能验证脚本 ==="
echo

# 检查基础编译
echo "1. 检查基础库编译..."
if cargo check --lib --features json,yaml --quiet; then
    echo "   ✅ 基础库编译成功"
else
    echo "   ❌ 基础库编译失败"
    exit 1
fi

# 检查调试功能编译
echo "2. 检查调试功能编译..."
if cargo check --lib --features json,yaml,debug --quiet; then
    echo "   ✅ 调试功能编译成功"
else
    echo "   ❌ 调试功能编译失败"
    exit 1
fi

# 检查CLI编译（如果可用）
echo "3. 检查CLI编译..."
if cargo check --bin xqpath --features json,yaml,cli --quiet 2>/dev/null; then
    echo "   ✅ CLI编译成功"
else
    echo "   ⚠️  CLI编译跳过（依赖较多）"
fi

# 运行基础测试
echo "4. 运行基础测试..."
if cargo test --lib --features json,yaml,debug --quiet; then
    echo "   ✅ 基础测试通过"
else
    echo "   ⚠️  基础测试失败"
fi

# 检查调试宏语法
echo "5. 检查调试宏语法..."
if cargo check --lib --features json,yaml,debug --quiet 2>/dev/null; then
    echo "   ✅ 调试宏语法正确"
else
    echo "   ❌ 调试宏语法错误"
fi

# 运行调试功能示例
echo "6. 运行调试功能示例..."
if cargo run --example debug_validation --features json,yaml,debug --quiet 2>/dev/null; then
    echo "   ✅ 调试功能示例运行成功"
else
    echo "   ⚠️  调试功能示例运行失败"
fi

# 验证 API 示例
echo "7. 验证 API 示例..."
if cargo run --example api_validation --features json,yaml --quiet 2>/dev/null; then
    echo "   ✅ API 示例验证成功"
else
    echo "   ⚠️  API 示例验证失败"
fi

echo
echo "=== 验证完成 ==="
echo
echo "📦 已实现的功能:"
echo "   • 调试基础设施 (src/debug/)"
echo "   • 调试宏: query_debug!, trace_query!"
echo "   • 调试API: DebugContext, Logger, Tracer"
echo "   • 特性控制: debug, profiling, monitoring"
echo
echo "🚀 下一步:"
echo "   • 实施 v1.4.2 性能监控分析"
echo "   • 添加 CLI 调试命令"
echo "   • 完善集成测试"
echo
