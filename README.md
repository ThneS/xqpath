# XQPath

> 高性能的 jq 风格结构化数据路径提取库 | 提供 Rust API 和命令行工具

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![Crates.io](https://img.shields.io/crates/v/xqpath.svg)](https://crates.io/crates/xqpath)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![CI](https://github.com/ThneS/xqpath/workflows/CI/badge.svg)](https://github.com/ThneS/xqpath/actions)
[![Release](https://github.com/ThneS/xqpath/workflows/Release/badge.svg)](https://github.com/ThneS/xqpath/actions)
[![Test Coverage](https://img.shields.io/badge/test_coverage-77_tests-green.svg)](#🧪-开发和测试)

## ✨ 特性

- **🚀 高性能**：快速路径提取和数据查询
- **📖 jq 兼容语法**：支持 `.field`、`[index]`、`*`、`**` 等常用路径
- **🔧 多格式支持**：JSON、YAML 格式无缝处理
- **⚡ 现代化 CLI**：10+ 专用命令，彩色输出
- **🛡️ 类型安全**：完整的 Rust 类型系统支持
- **🔍 调试工具**：完整的调试和性能分析功能
- **⚙️ 配置管理**：统一的配置文件管理和模板系统
- **🎯 交互式调试**：强大的交互式调试器

## 📦 安装

```bash
# 安装库
cargo add xqpath

# 安装命令行工具
cargo install xqpath
```

## 🚀 快速开始

### Rust 库用法

```rust
use xqpath::{query, query_one, exists};
use serde_json::json;

let data = r#"{
  "users": [
    {"name": "Alice", "age": 30},
    {"name": "Bob", "age": 25}
  ]
}"#;

// 查询多个值
let names = query!(data, "users[*].name").unwrap();
// [String("Alice"), String("Bob")]

// 查询单个值
let first_name = query_one!(data, "users[0].name").unwrap();
// Some(String("Alice"))

// 检查路径是否存在
let has_users = exists!(data, "users").unwrap();
// true
```

### 命令行工具

```bash
# 基本查询
echo '{"users": [{"name": "Alice"}]}' | xqpath get 'users[*].name'
xqpath get '.config.version' -f config.yaml

# 路径检查和类型
xqpath exists '.user.email' -f data.json
xqpath type '.users' -f data.json

# 计数和键名
xqpath count '.users[*]' -f data.json
xqpath keys '.config' -f settings.json

# 格式转换
xqpath convert yaml -f config.json
xqpath validate -f data.json

# 配置管理
xqpath config show                    # 显示当前配置
xqpath config set debug.enabled true # 设置配置项
xqpath config profile use production  # 切换配置模板
xqpath config profile list            # 列出可用模板

# 交互式调试器
xqpath debug                          # 启动交互式调试器

# 调试和性能分析
xqpath debug '.complex.path' -f data.json
xqpath trace '.users[*]' -f data.json --detailed
xqpath profile '.query' -f data.json --memory
xqpath benchmark '.path' -f data.json --iterations 1000
```

## 📖 路径语法

```bash
# 基础访问
.field              # 字段访问
.nested.field       # 嵌套字段
[0]                 # 数组索引
[*]                 # 数组通配符
.users[*].name      # 组合使用

# 高级操作
.config.**          # 递归搜索
.users | length     # 管道操作
.users | keys       # 内置函数
```

## 🔧 高级功能

### 配置管理

```bash
# 显示当前配置
xqpath config show

# 设置配置项
xqpath config set output.format yaml
xqpath config set debug.enabled true

# 配置模板管理
xqpath config profile list              # 列出可用模板
xqpath config profile use development   # 切换到开发模板
xqpath config profile use production    # 切换到生产模板
xqpath config profile create my-config  # 创建自定义模板
```

### 交互式调试器

```bash
# 启动交互式调试器
xqpath debug

# 调试器中的命令
> extract .user.name                # 提取数据
> set file data.json               # 设置数据文件
> eval ".items[*].price"           # 评估表达式
> show config                      # 显示配置
> stats                           # 性能统计
> help                            # 显示帮助
> quit                            # 退出调试器
```

### 调试和性能分析

```bash
# 调试模式 - 显示详细执行信息
xqpath debug '.complex.query' -f data.json
# 输出: 解析时间、执行时间、查询路径、错误分析

# 执行跟踪 - 追踪查询执行过程
xqpath trace '.users[*].name' -f data.json --detailed
# 输出: 执行时间统计、结果类型分析

# 性能分析 - 内存和性能指标
xqpath profile '.query' -f data.json --memory --hints

# 基准测试 - 性能基准对比
xqpath benchmark '.path' -f data.json --iterations 1000

# 实时监控 - 长时间性能观察
xqpath monitor '.path' -f data.json --duration 30 --interval 1
```

### 错误处理

```rust
// 优雅的错误处理
match query!(data, ".some.path") {
    Ok(result) => println!("Found: {:?}", result),
    Err(e) => eprintln!("Error: {}", e),
}

// 可选字段查询
let optional = query_one!(data, ".user.email")?; // 返回 Option<Value>
```

## 🧪 开发和测试

### 快速开始

```bash
# 克隆项目
git clone https://github.com/ThneS/xqpath.git
cd xqpath

# 快速测试
make test-quick

# 或使用测试脚本
./scripts/test-runner.sh quick
```

### 测试命令

```bash
# Makefile方式 (推荐)
make test-quick    # 快速测试 (~1秒)
make test-config   # 配置管理测试
make test-debug    # 调试功能测试
make test-all      # 完整测试

# 测试脚本方式
./scripts/test-runner.sh quick -q     # 静默快速测试
./scripts/test-runner.sh config -v    # 详细配置测试
./scripts/test-runner.sh all -f       # 并行完整测试

# 原生cargo方式
cargo test --features config-management,interactive-debug
```

### 开发工作流

```bash
# 开发前检查
make dev-check     # 格式化 + 检查 + 快速测试

# 代码质量
make fmt          # 代码格式化
make lint         # 代码检查
make check        # 语法检查

# 构建
make build        # 开发构建
make release      # 发布构建
```

### 配置文件

项目支持多种配置方式，配置文件位于 `config/` 目录：

```bash
config/
├── examples/          # 配置示例
├── templates/         # 配置模板
└── profiles/          # 预定义配置
```

详细的开发指南请参考 [`docs/test-optimization.md`](docs/test-optimization.md)。

## 🔄 CI/CD 集成

项目已集成完整的 CI/CD 流程：

### GitHub Actions 工作流

- **CI**: 自动化测试、代码检查、跨平台测试
- **Release**: 多平台构建、自动发布到 crates.io

### 本地开发与 CI 一致性

```bash
# 模拟CI快速检查流程
make dev-check

# 模拟CI完整测试流程
make ci-check

# 模拟发布前检查
make pre-release
```

### CI 测试策略

1. **快速检查** (并行): 格式化、语法检查、代码质量
2. **分层测试** (并行): 核心测试、配置测试、调试测试
3. **完整测试**: 所有功能集成测试
4. **跨平台测试**: Linux、Windows、macOS
5. **发布检查**: 发布前完整验证

## 🤝 贡献

欢迎贡献代码和反馈！查看 [贡献指南](CONTRIBUTING.md) 了解详情。

## 📄 许可证

本项目采用 [Apache-2.0](LICENSE) 许可证。

---

**XQPath** - 让结构化数据处理变得简单高效 🚀
