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

#### 使用便利宏

```rust
use xqpath::{query, query_one, exists, count};
use serde_json::json;

let data = r#"
{
  "users": [
    {"name": "Alice", "age": 30, "active": true},
    {"name": "Bob", "age": 25, "active": false}
  ]
}
"#;

// 查询多个值
let names = query!(data, "users[*].name").unwrap();
// ["Alice", "Bob"]

// 查询单个值
let first_name = query_one!(data, "users[0].name").unwrap();
// Some("Alice")

// 检查路径是否存在
let has_users = exists!(data, "users").unwrap();
// true

// 计算数量
let user_count = count!(data, "users[*]").unwrap();
// 2
```

#### 使用表达式 API

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

## � 便利宏

XQPath 提供了一套简洁易用的宏来简化常见操作：

### 基础查询宏

- `query!(data, path)` - 查询多个值，返回 `Vec<Value>`
- `query_one!(data, path)` - 查询单个值，返回 `Option<Value>`
- `query_or_default!(data, path, default)` - 查询值或返回默认值
- `query_as_type!(data, path, Type)` - 查询并转换为指定类型

### 多路径查询宏

- `query_multi!(data, path1, path2, ...)` - 同时查询多个路径
- `query_string!(data, path)` - 查询并转换为字符串
- `query_length!(data, path)` - 查询数组/对象长度

### 存在检查宏

- `exists!(data, path)` - 检查单个路径是否存在
- `exists_all!(data, path1, path2, ...)` - 检查所有路径是否都存在
- `exists_any!(data, path1, path2, ...)` - 检查是否存在任意一个路径

### 实用工具宏

- `count!(data, path)` - 计算匹配值的数量
- `get_type!(data, path)` - 获取值的类型信息
- `extract!(data, path, format)` - 提取并转换格式
- `update!(data, path, value)` - 更新值（需要 `update` feature）

## �📖 表达式语法

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
