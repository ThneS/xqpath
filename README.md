# XQPath

> 高性能的 jq 风格结构化数据路径提取库 | 提供 Rust API 和命令行工具

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![Crates.io](https://img.shields.io/crates/v/xqpath.svg)](https://crates.io/crates/xqpath)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

## ✨ 特性

- **🚀 高性能**：快速路径提取和数据查询
- **📖 jq 兼容语法**：支持 `.field`、`[index]`、`*`、`**` 等常用路径
- **🔧 多格式支持**：JSON、YAML 格式无缝处理
- **⚡ 现代化 CLI**：10+ 专用命令，彩色输出
- **🛡️ 类型安全**：完整的 Rust 类型系统支持
- **🔍 调试工具**：v1.4.1 提供完整的调试和性能分析功能

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

let data = json!({
  "users": [
    {"name": "Alice", "age": 30},
    {"name": "Bob", "age": 25}
  ]
});

// 查询多个值
let names = query!(data, "users[*].name").unwrap();
// ["Alice", "Bob"]

// 查询单个值
let first_name = query_one!(data, "users[0].name").unwrap();
// Some("Alice")

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

# 调试和性能分析 (v1.4.1)
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

### 调试和性能分析 (v1.4.1)

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

## 📚 版本功能

### v1.4.1 - 调试和错误分析

- ✅ **完整的调试系统**：`debug` 和 `trace` 命令
- ✅ **智能错误分析**：自动检测错误类型并提供修复建议
- ✅ **全局调试选项**：支持所有命令的调试模式
- ✅ **执行时间统计**：详细的性能分析信息

### v1.4.2 - 性能监控 (当前版本)

- ✅ **性能分析**：`profile` 命令，内存和执行指标
- ✅ **基准测试**：`benchmark` 命令，量化性能对比
- ✅ **实时监控**：`monitor` 命令，长时间性能观察
- ✅ **多格式报告**：支持 HTML、JSON、CSV 输出

## 🤝 贡献

欢迎贡献代码和反馈！查看 [贡献指南](CONTRIBUTING.md) 了解详情。

## 📄 许可证

本项目采用 [Apache-2.0](LICENSE) 许可证。

---

**XQPath** - 让结构化数据处理变得简单高效 🚀
