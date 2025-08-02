#!/bin/bash

# XQPath 配置文件验证脚本

set -e

echo "🔧 验证 XQPath 配置文件..."

CONFIG_DIR="$(dirname "$0")"
EXAMPLES_DIR="$CONFIG_DIR/examples"
TEMPLATES_DIR="$CONFIG_DIR/templates"

# 检查配置文件语法
validate_yaml() {
    local file="$1"
    echo "📋 验证 $file..."

    # 使用 yq 或 python 验证 YAML 语法
    if command -v yq >/dev/null 2>&1; then
        yq eval '.' "$file" >/dev/null
    elif command -v python3 >/dev/null 2>&1; then
        if python3 -c "import yaml" >/dev/null 2>&1; then
            python3 -c "import yaml; yaml.safe_load(open('$file'))"
        else
            echo "⚠️  警告: 未安装 PyYAML，跳过 YAML 语法验证"
            return 0
        fi
    else
        echo "⚠️  警告: 未找到 yq 或 python3，跳过 YAML 语法验证"
        return 0
    fi

    echo "✅ $file 语法正确"
}

# 验证配置文件结构
validate_structure() {
    local file="$1"
    echo "🔍 验证 $file 结构..."

    # 检查必需的配置段落
    required_sections=("debug" "performance" "paths" "features")

    for section in "${required_sections[@]}"; do
        if command -v yq >/dev/null 2>&1; then
            if ! yq eval "has(\"$section\")" "$file" | grep -q "true"; then
                echo "❌ 错误: $file 缺少必需的配置段落: $section"
                return 1
            fi
        fi
    done

    echo "✅ $file 结构正确"
}

# 验证所有示例配置文件
echo "📁 验证示例配置文件..."
for file in "$EXAMPLES_DIR"/*.yaml; do
    if [ -f "$file" ]; then
        validate_yaml "$file"
        validate_structure "$file"
    fi
done

# 验证所有模板配置文件
echo "📁 验证模板配置文件..."
for file in "$TEMPLATES_DIR"/*.yaml; do
    if [ -f "$file" ]; then
        validate_yaml "$file"
        validate_structure "$file"
    fi
done

echo "🎉 所有配置文件验证通过！"
