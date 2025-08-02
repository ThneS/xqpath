#!/bin/bash

# XQPath 测试管理脚本
# 用于优化不同feature的测试执行和管理

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 测试配置
CARGO_CMD="cargo test"
DEFAULT_FEATURES="config-management,interactive-debug"

# 显示帮助信息
show_help() {
    echo "XQPath 测试管理脚本"
    echo ""
    echo "用法: $0 [选项] [测试类型]"
    echo ""
    echo "测试类型:"
    echo "  core          核心功能测试 (基础API)"
    echo "  config        配置管理测试"
    echo "  debug         调试功能测试"
    echo "  integration   集成测试"
    echo "  performance   性能测试"
    echo "  all           所有测试"
    echo "  quick         快速测试 (跳过性能测试)"
    echo ""
    echo "选项:"
    echo "  -h, --help    显示帮助信息"
    echo "  -v, --verbose 详细输出"
    echo "  -q, --quiet   静默模式"
    echo "  -f, --fast    快速模式 (并行测试)"
    echo "  --no-features 不启用任何features"
    echo "  --features    指定features"
    echo ""
    echo "示例:"
    echo "  $0 core                    # 运行核心测试"
    echo "  $0 quick -f               # 快速并行测试"
    echo "  $0 all --features debug   # 只测试debug功能"
}

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查依赖
check_dependencies() {
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo 未安装"
        exit 1
    fi
}

# 运行特定测试组
run_test_group() {
    local test_type=$1
    local features=$2
    local extra_args=$3

    log_info "运行 $test_type 测试..."

    case $test_type in
        "core")
            # 核心功能测试 - 不需要额外features
            $CARGO_CMD --lib $extra_args
            $CARGO_CMD --test integration $extra_args
            ;;
        "config")
            # 配置管理测试
            $CARGO_CMD --test config_debug_features --features config-management $extra_args
            $CARGO_CMD --lib --features config-management $extra_args
            ;;
        "debug")
            # 调试功能测试
            $CARGO_CMD --test config_debug_features --features interactive-debug $extra_args
            $CARGO_CMD --lib --features interactive-debug $extra_args
            ;;
        "integration")
            # 集成测试
            $CARGO_CMD --tests --features "$features" $extra_args
            ;;
        "performance")
            # 性能测试
            log_info "运行性能基准测试..."
            cargo bench --features "$features" 2>/dev/null || log_warning "性能测试需要 nightly Rust"
            ;;
        "quick")
            # 快速测试 - 跳过性能测试
            $CARGO_CMD --lib --features "$features" $extra_args
            $CARGO_CMD --test integration --features "$features" $extra_args
            $CARGO_CMD --test config_debug_features --features "$features" $extra_args
            ;;
        "all")
            # 所有测试
            $CARGO_CMD --all --features "$features" $extra_args
            ;;
        *)
            log_error "未知的测试类型: $test_type"
            show_help
            exit 1
            ;;
    esac
}

# 获取测试统计信息
get_test_stats() {
    local features=$1

    log_info "收集测试统计信息..."

    echo "=== 测试文件统计 ==="
    find tests/ -name "*.rs" -exec basename {} \; | sort
    echo ""

    echo "=== 代码行数统计 ==="
    find tests/ -name "*.rs" -exec wc -l {} \; | sort -n
    echo ""

    echo "=== 测试数量统计 ==="
    if [ -n "$features" ]; then
        cargo test --features "$features" -- --list 2>/dev/null | grep -c "test " || echo "0"
    else
        cargo test -- --list 2>/dev/null | grep -c "test " || echo "0"
    fi
    echo " 个测试用例"
    echo ""
}

# 清理测试输出
cleanup_test_output() {
    # 清理编译缓存
    if [ -d "target/debug/deps" ]; then
        log_info "清理测试缓存..."
        find target/debug/deps -name "*test*" -type f -delete 2>/dev/null || true
    fi
}

# 主函数
main() {
    local test_type="quick"
    local features="$DEFAULT_FEATURES"
    local verbose=false
    local quiet=false
    local fast=false
    local extra_args=""

    # 解析命令行参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -v|--verbose)
                verbose=true
                extra_args="$extra_args --nocapture"
                shift
                ;;
            -q|--quiet)
                quiet=true
                extra_args="$extra_args -q"
                shift
                ;;
            -f|--fast)
                fast=true
                extra_args="$extra_args --test-threads=0"
                shift
                ;;
            --no-features)
                features=""
                shift
                ;;
            --features)
                features="$2"
                shift 2
                ;;
            core|config|debug|integration|performance|all|quick)
                test_type="$1"
                shift
                ;;
            *)
                log_error "未知参数: $1"
                show_help
                exit 1
                ;;
        esac
    done

    # 检查依赖
    check_dependencies

    # 显示运行信息
    if [ "$quiet" = false ]; then
        log_info "测试类型: $test_type"
        log_info "Features: ${features:-"none"}"
        if [ "$fast" = true ]; then
            log_info "并行测试模式已启用"
        fi
        echo ""
    fi

    # 记录开始时间
    start_time=$(date +%s)

    # 运行测试
    if run_test_group "$test_type" "$features" "$extra_args"; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        log_success "测试完成! 耗时: ${duration}秒"

        # 显示统计信息
        if [ "$test_type" = "all" ] && [ "$quiet" = false ]; then
            get_test_stats "$features"
        fi
    else
        log_error "测试失败!"
        exit 1
    fi

    # 清理
    if [ "$test_type" = "all" ]; then
        cleanup_test_output
    fi
}

# 运行主函数
main "$@"
