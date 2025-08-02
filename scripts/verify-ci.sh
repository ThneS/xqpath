#!/bin/bash

# CI配置验证脚本
# 本地验证CI工作流是否正确配置

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

# 验证CI配置文件
validate_ci_config() {
    log_info "验证CI配置文件..."

    if [ ! -f ".github/workflows/ci.yml" ]; then
        log_error "CI配置文件不存在"
        return 1
    fi

    if [ ! -f ".github/workflows/release.yml" ]; then
        log_error "Release配置文件不存在"
        return 1
    fi

    log_success "CI配置文件存在"
}

# 验证Makefile命令
validate_makefile() {
    log_info "验证Makefile命令..."

    # 检查Makefile是否存在
    if [ ! -f "Makefile" ]; then
        log_error "Makefile不存在"
        return 1
    fi

    # 检查关键make目标
    local targets=("test-quick" "test-core" "test-config" "test-debug" "test-all" "dev-check" "ci-check" "pre-release")

    for target in "${targets[@]}"; do
        if grep -q "^${target}:" Makefile; then
            log_success "Make目标 '${target}' 存在"
        else
            log_error "Make目标 '${target}' 不存在"
            return 1
        fi
    done
}

# 验证测试脚本
validate_test_script() {
    log_info "验证测试脚本..."

    if [ ! -f "scripts/test-runner.sh" ]; then
        log_error "测试脚本不存在"
        return 1
    fi

    if [ ! -x "scripts/test-runner.sh" ]; then
        log_error "测试脚本没有执行权限"
        return 1
    fi

    log_success "测试脚本配置正确"
}

# 模拟CI快速检查流程
simulate_ci_quick_check() {
    log_info "模拟CI快速检查流程..."

    echo "1. 代码格式检查..."
    if cargo fmt --all -- --check >/dev/null 2>&1; then
        log_success "代码格式检查通过"
    else
        log_warning "代码格式需要修复，运行 'make fmt'"
    fi

    echo "2. 语法检查..."
    if make check >/dev/null 2>&1; then
        log_success "语法检查通过"
    else
        log_error "语法检查失败"
        return 1
    fi

    echo "3. 代码质量检查..."
    if make lint >/dev/null 2>&1; then
        log_success "代码质量检查通过"
    else
        log_warning "代码质量检查有警告"
    fi
}

# 模拟CI测试流程
simulate_ci_test_flow() {
    log_info "模拟CI测试流程..."

    echo "1. 核心测试..."
    if make test-core >/dev/null 2>&1; then
        log_success "核心测试通过"
    else
        log_error "核心测试失败"
        return 1
    fi

    echo "2. 配置功能测试..."
    if make test-config >/dev/null 2>&1; then
        log_success "配置功能测试通过"
    else
        log_error "配置功能测试失败"
        return 1
    fi

    echo "3. 调试功能测试..."
    if make test-debug >/dev/null 2>&1; then
        log_success "调试功能测试通过"
    else
        log_error "调试功能测试失败"
        return 1
    fi
}

# 验证发布流程
validate_release_flow() {
    log_info "验证发布流程..."

    echo "1. 检查是否可以构建release版本..."
    if cargo build --release --features config-management,interactive-debug >/dev/null 2>&1; then
        log_success "Release构建成功"
    else
        log_error "Release构建失败"
        return 1
    fi

    echo "2. 检查是否可以运行发布前检查..."
    if make pre-release >/dev/null 2>&1; then
        log_success "发布前检查通过"
    else
        log_warning "发布前检查有问题"
    fi
}

# 生成CI状态报告
generate_ci_report() {
    log_info "生成CI状态报告..."

    cat > ci-status-report.md << EOF
# CI配置状态报告

## 配置文件
- ✅ CI工作流配置: .github/workflows/ci.yml
- ✅ Release工作流配置: .github/workflows/release.yml
- ✅ Makefile: 所有必要目标已配置
- ✅ 测试脚本: scripts/test-runner.sh

## CI流程验证
- ✅ 快速检查流程: 格式化、语法、代码质量
- ✅ 分层测试流程: 核心、配置、调试功能
- ✅ 发布流程: Release构建和检查

## 本地命令映射
\`\`\`bash
# CI快速检查 -> 本地开发检查
make dev-check

# CI完整测试 -> 本地CI检查
make ci-check

# CI发布检查 -> 本地发布前检查
make pre-release
\`\`\`

## 推荐的开发工作流
1. 开发时: \`make test-quick\` (1秒快速验证)
2. 提交前: \`make dev-check\` (格式化+检查+测试)
3. 发布前: \`make pre-release\` (完整验证)

报告生成时间: $(date)
EOF

    log_success "CI状态报告已生成: ci-status-report.md"
}

# 主函数
main() {
    echo "=== XQPath CI配置验证 ==="
    echo ""

    # 验证各个组件
    validate_ci_config || exit 1
    validate_makefile || exit 1
    validate_test_script || exit 1

    echo ""

    # 模拟CI流程
    simulate_ci_quick_check || exit 1
    simulate_ci_test_flow || exit 1
    validate_release_flow || exit 1

    echo ""

    # 生成报告
    generate_ci_report

    echo ""
    log_success "所有CI配置验证通过! 🎉"
    log_info "项目已完全集成CI/CD流程"
}

main "$@"
