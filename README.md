# XQPath

> A modern jq-inspired path extractor and updater for structured data in Rust

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![Crates.io](https://img.shields.io/crates/v/xqpath.svg)](https://crates.io/crates/xqpath)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

## 🎯 概述

XQPath v1.3.2 是一个用于结构化数据（JSON/YAML）路径提取与操作的高性能 Rust 工具，提供 jq 风格的表达式语法和现代化的命令行体验。

### 双重形态

- **Rust 库**：嵌入到 Rust 项目中使用，提供丰富的宏和 API
- **命令行工具**：功能强大的 CLI，支持 10+ 命令和彩色输出

## ✨ 核心特性

- **🚀 高性能路径提取**：支持 `.field`、`[index]`、`*`、`**` 等路径语法
- **⚡ 现代化 CLI**：10+ 专用命令 (get, set, exists, type, count, length, keys, validate, convert, examples)
- **🎨 彩色输出**：智能着色和格式化，提升使用体验
- **🔧 多格式支持**：JSON、YAML 无缝切换和转换
- **📖 jq 兼容语法**：熟悉的表达式语言，学习成本低
- **🛡️ 类型安全**：完整的 Rust 类型系统支持
- **🔍 智能检测**：自动格式检测和验证

## � 安装

### Cargo 安装 (推荐)

```bash
# 安装库
cargo add xqpath

# 安装命令行工具
cargo install xqpath
```

### 从源码编译

```bash
git clone https://github.com/ThneS/xqpath.git
cd xqpath
cargo build --release
```

## 🚀 快速开始

### 库用法 - 便利宏

```rust
use xqpath::{query, query_one, exists, count};
use serde_json::json;

let data = json!({
  "users": [
    {"name": "Alice", "age": 30, "active": true},
    {"name": "Bob", "age": 25, "active": false}
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

// 计算数量
let user_count = count!(data, "users[*]").unwrap();
// 2
```

## 🖥️ 命令行工具 (v1.3.2 新特性)

XQPath 提供了功能强大的命令行工具，支持 10+ 专用命令：

### 核心命令

```bash
# 基本查询
echo '{"users": [{"name": "Alice"}]}' | xqpath get 'users[*].name'
xqpath get '.config.version' -f config.yaml

# 路径检查
xqpath exists '.user.email' -f data.json

# 类型获取
xqpath type '.users' -f data.json

# 计数和长度
xqpath count '.users[*]' -f data.json
xqpath length '.users' -f data.json

# 获取键名
xqpath keys '.config' -f settings.json
```

### 高级功能

```bash
# 格式验证
xqpath validate -f data.json

# 格式转换
xqpath convert yaml -f config.json --pretty
xqpath convert json -f config.yaml

# 显示用法示例
xqpath examples
```

### 更新操作 (需要 update feature)

```bash
# 更新字段值
xqpath set '.version' '"2.0"' -f config.json
```

### 输出格式控制

```bash
# 指定输出格式
xqpath get '.data' -f file.json --output yaml
xqpath get '.data' -f file.json --output json-pretty

# 控制颜色和详细度
xqpath get '.data' -f file.json --no-color --verbose
```

## 📖 表达式语法

### 基础语法

```bash
# 字段访问
.field              # 获取字段
.nested.field       # 嵌套字段访问
.field?             # 可选字段（不存在时返回 null）

# 数组操作
[0]                 # 数组索引
[*]                 # 数组通配符
[-1]                # 负索引（最后一个元素）

# 组合操作
.users[*].name      # 获取所有用户名
.config.**          # 递归搜索
```

### 高级表达式

```bash
# 管道操作
.users | length()           # 获取数组长度
.users | map(.name)         # 映射操作
.users | select(.active)    # 条件过滤

# 条件表达式
if .age >= 30 then "senior" else "junior" end

# 比较操作
.users | select(.age > 25)
.items | select(.price <= 100)
```

### 内置函数

- `length()` - 获取长度（数组、对象、字符串）
- `keys()` - 获取对象键名或数组索引
- `type()` - 获取值类型
- `map(expr)` - 数组映射
- `select(condition)` - 条件过滤
- `sort()`, `sort_by(expr)` - 排序操作

## 🔧 实用宏系统

XQPath 提供了丰富的宏来简化常见操作：

### 基础查询宏

```rust
// 单值查询
let name = query_one!(data, ".user.name")?;
let age = query_as_type!(data, ".user.age", i32)?;

// 多值查询
let names = query!(data, ".users[*].name")?;
let emails = query_string!(data, ".users[*].email")?;

// 存在检查
let has_email = exists!(data, ".user.email")?;
let has_all = exists_all!(data, ".name", ".email", ".age")?;

// 计数和类型
```

## 🔧 高级用法示例

### 复杂数据处理

```rust
use xqpath::{parse_path_expression, evaluate_path_expression};

// 聚合查询示例
let expr = parse_path_expression("
    .orders
    | select(.status == "completed")
    | map(.amount)
    | add
")?;

// 条件过滤和映射
let active_users = parse_path_expression("
    .users
    | select(.active)
    | map(.name)
")?;
```

### 错误处理

```rust
// 使用 try-catch 处理可能不存在的路径
let expr = parse_path_expression("
    try .config.database.url
    catch "sqlite://default.db"
")?;

// 使用可选操作符
let optional_field = query_one!(data, ".user.email?")?;
```

## 🎯 v1.3.2 新特性总结

- **🚀 现代化 CLI**: 10+ 专用命令，彩色输出，智能格式检测
- **⚡ 增强的命令**: get, set, exists, type, count, length, keys, validate, convert, examples
- **🎨 更好的 UX**: 详细/简洁输出模式，格式转换，交互式帮助
- **🔧 更强的 API**: 丰富的宏系统，类型安全的查询接口

## 📚 更多资源

- **[完整文档](docs/README.md)** - 详细的 API 文档和指南
- **[功能示例](examples/)** - 各种使用示例
- **[GitHub 仓库](https://github.com/ThneS/xqpath)** - 源码和问题反馈

## 🤝 贡献与许可证

欢迎贡献代码和反馈！本项目采用 [Apache-2.0](LICENSE) 许可证。

---

**XQPath v1.3.2** - 让结构化数据处理变得简单高效 🚀

```

```
