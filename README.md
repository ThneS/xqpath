# XQPath

> A jq-inspired expression parser and evaluator for structured data in Rust

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

## 🎯 概述

XQPath 是一个用于结构化数据（JSON/YAML）路径提取与操作的高性能 Rust 工具，提供 jq 风格的表达式语法。

### 双重形态

- **Rust 库**：嵌入到 Rust 项目中使用
- **命令行工具**：处理文件和管道数据

## ✨ 主要特性

- **路径提取**：支持 `.field`、`[index]`、`*`、`**` 等路径语法
- **管道操作**：`expr1 | expr2` 管道操作符
- **逗号操作**：`expr1, expr2` 多路选择
- **内置函数**：`length()`, `keys()`, `map()`, `select()`, `sort_by()` 等
- **条件表达式**：`if-then-else` 条件判断
- **比较与逻辑**：`==`, `!=`, `>`, `<`, `and`, `or`, `not`
- **错误处理**：`try-catch` 表达式和 `?` 操作符
- **字面量**：支持字符串、数字、数组、对象字面量

## 🚀 快速开始

### 安装

```toml
[dependencies]
xqpath = "1.2.1"
```

### 基本用法

```rust
use xqpath::{parse_path_expression, evaluate_path_expression};
use serde_json::json;

let data = json!({
    "users": [
        {"name": "Alice", "age": 30, "active": true},
        {"name": "Bob", "age": 25, "active": false}
    ]
});

// 基本路径提取
let expr = parse_path_expression(".users[0].name")?;
let result = evaluate_path_expression(&expr, &data)?;
// 结果: ["Alice"]

// 管道操作
let expr = parse_path_expression(".users | length()")?;
let result = evaluate_path_expression(&expr, &data)?;
// 结果: [2]

// 条件过滤
let expr = parse_path_expression(".users | select(.active) | map(.name)")?;
let result = evaluate_path_expression(&expr, &data)?;
// 结果: [["Alice"]]

// 条件表达式
let expr = parse_path_expression("
    .users | map(if .age >= 30 then \"senior\" else \"junior\" end)
")?;
let result = evaluate_path_expression(&expr, &data)?;
// 结果: [["senior", "junior"]]
```

## 📖 表达式语法

### 路径语法
- `.field` - 字段访问
- `[0]` - 数组索引
- `[*]` - 数组通配符
- `**` - 递归通配符

### 操作符
- `|` - 管道：将左侧结果传递给右侧
- `,` - 逗号：收集多个表达式结果
- `==`, `!=`, `>`, `<`, `>=`, `<=` - 比较操作符
- `and`, `or`, `not` - 逻辑操作符

### 内置函数
- `length()` - 获取数组长度
- `keys()` - 获取对象键名
- `type()` - 获取值类型
- `map(expr)` - 数组映射
- `select(condition)` - 条件过滤
- `sort()`, `sort_by(expr)` - 排序
- `unique()`, `reverse()` - 数组操作

### 条件与错误处理
```bash
# 条件表达式
if condition then expr1 else expr2 end

# 错误处理
try expr catch fallback
expr?  # 可选操作符
```

## 🖥️ 命令行工具

```bash
# 安装命令行工具
cargo install xqpath

# 基本用法
echo '{"name": "Alice", "age": 30}' | xqpath '.name'

# 从文件读取
xqpath '.users | length()' data.json

# 复杂查询
cat data.json | xqpath '.users | select(.active) | map(.name)'
```

## 🔧 高级用法

### 复杂数据处理
```rust
let expr = parse_path_expression("
    .orders
    | select(.status == \"completed\")
    | group_by(.customer_id)
    | map({
        customer: .[0].customer_id,
        total: map(.amount) | add,
        count: length()
      })
    | sort_by(.total)
    | reverse()
")?;
```

### 错误处理
```rust
let expr = parse_path_expression("
    try .config.database.url 
    catch \"sqlite://default.db\"
")?;
```

## 📚 文档

- **[完整文档](docs/README.md)** - 详细的 API 文档和指南
- **[功能示例](examples/)** - 各种使用示例
- **[开发路线图](docs/planning/)** - 项目发展计划

## 🤝 贡献

我们欢迎各种形式的贡献！请查看 [文档目录](docs/) 了解项目详情。

## 📄 许可证

本项目采用 [Apache-2.0](LICENSE) 许可证。

---

**设计理念**: XQPath 致力于提供简单、高效、可扩展的结构化数据处理体验，无论是在命令行环境还是 Rust 应用程序中。
