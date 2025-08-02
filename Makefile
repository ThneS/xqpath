# XQPath Makefile - 测试管理和构建自动化

.PHONY: help test test-quick test-core test-config test-debug test-all clean lint fmt check

# 默认目标
help:
	@echo "XQPath 测试管理"
	@echo ""
	@echo "可用命令:"
	@echo "  test-quick    快速测试 (核心功能)"
	@echo "  test-core     核心功能测试"
	@echo "  test-config   配置管理测试"
	@echo "  test-debug    调试功能测试"
	@echo "  test-all      所有测试"
	@echo "  test-perf     性能测试"
	@echo ""
	@echo "代码质量:"
	@echo "  check         语法检查"
	@echo "  lint          代码检查"
	@echo "  fmt           代码格式化"
	@echo ""
	@echo "构建管理:"
	@echo "  build         构建项目"
	@echo "  clean         清理构建文件"
	@echo "  release       发布构建"

# 快速测试 - 日常开发使用
test-quick:
	@echo "🚀 运行快速测试..."
	@cargo test --lib --test integration --test config_debug_features \
		--features config-management,interactive-debug \
		--quiet

# 核心功能测试
test-core:
	@echo "🔧 运行核心功能测试..."
	@cargo test --lib --test integration

# 配置管理测试
test-config:
	@echo "⚙️ 运行配置管理测试..."
	@cargo test --test config_debug_features --features config-management \
		--quiet -- --test-threads=1

# 调试功能测试
test-debug:
	@echo "🐛 运行调试功能测试..."
	@cargo test --test config_debug_features --features interactive-debug \
		--quiet -- --test-threads=1

# 所有测试
test-all:
	@echo "🧪 运行所有测试..."
	@cargo test --all --features config-management,interactive-debug

# 性能测试
test-perf:
	@echo "⚡ 运行性能测试..."
	@cargo bench --features config-management,interactive-debug 2>/dev/null || \
		echo "性能测试需要 nightly Rust 或 criterion feature"

# 测试统计
test-stats:
	@echo "📊 测试统计信息:"
	@echo "文件数量: $$(find tests/ -name '*.rs' | wc -l)"
	@echo "代码行数: $$(find tests/ -name '*.rs' -exec cat {} \; | wc -l)"
	@echo "测试用例: $$(cargo test --features config-management,interactive-debug -- --list 2>/dev/null | grep -c 'test ' || echo '0')"

# 语法检查
check:
	@echo "🔍 语法检查..."
	@cargo check --all-features

# 代码检查
lint:
	@echo "📋 代码检查..."
	@cargo clippy --all-features -- -D warnings

# 代码格式化
fmt:
	@echo "🎨 代码格式化..."
	@cargo fmt --all

# 构建项目
build:
	@echo "🔨 构建项目..."
	@cargo build --all-features

# 发布构建
release:
	@echo "📦 发布构建..."
	@cargo build --release --all-features

# 清理
clean:
	@echo "🧹 清理构建文件..."
	@cargo clean
	@rm -rf target/

# 开发环境检查
dev-check: fmt lint check test-quick
	@echo "✅ 开发环境检查完成!"

# CI检查 - 用于持续集成
ci-check: check lint test-all
	@echo "✅ CI检查完成!"

# 发布前检查
pre-release: ci-check test-perf test-stats
	@echo "✅ 发布前检查完成!"

# 一键测试 - 根据参数选择测试类型
test:
	@./scripts/test-runner.sh quick --fast
