#!/bin/bash

# XQPath 基准测试运行脚本
# 用于运行所有基准测试并生成报告

set -e

echo "🚀 XQPath v1.4.2 基准测试套件"
echo "================================"

# 检查必要的功能是否可用
echo "📋 检查功能可用性..."

FEATURES="profiling,benchmark"
BASIC_FEATURES=""

# 运行基础基准测试
echo ""
echo "🔥 运行基础性能基准测试..."
cargo bench --bench performance

# 运行高级基准测试
echo ""
echo "⚡ 运行高级基准测试套件..."
if cargo check --features="$FEATURES" >/dev/null 2>&1; then
    echo "✅ profiling 和 benchmark 功能可用"
    cargo bench --bench advanced_benchmarks --features="$FEATURES"
else
    echo "⚠️  某些功能不可用，运行基础测试"
    cargo bench --bench advanced_benchmarks
fi

# 运行性能演示
echo ""
echo "🎯 运行性能演示..."
if cargo check --features="$FEATURES" >/dev/null 2>&1; then
    echo "✅ 运行完整功能演示"
    cargo run --features="$FEATURES" --example performance_demo
else
    echo "⚠️  运行基础功能演示"
    cargo run --example performance_demo
fi

echo ""
echo "📊 基准测试报告位置:"
echo "  - target/criterion/ - Criterion HTML 报告"
echo "  - benchmark_report.html - 自定义基准报告"
echo "  - performance_report.html - 性能监控报告"

echo ""
echo "✨ 基准测试完成！"
