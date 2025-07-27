# DataPath

> A minimal jq-like path extractor and updater for structured data in Rust

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## 🎯 项目概述

DataPath 是一个用于结构化数据（JSON/YAML）路径提取与更新的高性能 Rust 工具，提供：

### 🧩 双重形态

- **命令行工具**：`datapath` CLI - 快速处理文件和管道数据
- **集成库**：`datapath` crate - 嵌入到 Rust 项目中

### ✨ 核心特性

| 功能         | 描述                                          | 状态             |
| ------------ | --------------------------------------------- | ---------------- | --- |
| **路径提取** | 支持 `.field`、`[index]`、`**` 等 jq 风格路径 | ✅               |
| **格式支持** | JSON/YAML 自动检测与解析                      | ✅               |
| **通配符**   | `*`、`**` 支持字段和递归匹配                  | ✅               |
| **类型断言** | 如 `.users[]                                  | string` 类型过滤 | ✅  |
| **字段更新** | 使用 `feature = "update"` 启用更新功能        | ⚙️               |
| **格式扩展** | 插件式支持 TOML、XML 等格式                   | ⚡️              |
| **高测试性** | 全模块单元测试，覆盖边界情况                  | 🧪               |
| **轻量依赖** | 最小依赖集（serde + winnow）                  | 📦               |

## 📦 项目架构

```
datapath/
├── Cargo.toml          # 项目配置
├── src/
│   ├── lib.rs          # 库入口，导出所有公共 API
│   ├── macros.rs       # 便利宏定义
│   ├── extractor.rs    # 路径提取核心逻辑
│   ├── updater.rs      # 路径更新逻辑（feature = "update"）
│   ├── parser/
│   │   └── path.rs     # 路径表达式解析（winnow 实现）
│   ├── value/
│   │   ├── format.rs   # ValueFormat trait 抽象
│   │   ├── json.rs     # JSON 格式支持
│   │   └── yaml.rs     # YAML 格式支持
│   └── cli.rs          # CLI 工具入口
└── tests/
    └── integration.rs  # 集成测试
```

## 🔧 核心模块设计

### 1. 路径解析器（parser::path）

使用 `winnow` 实现高性能路径解析，支持：

```rust
enum PathSegment {
    Field(String),          // .field
    Index(usize),           // [0]
    Wildcard,               // *
    RecursiveWildcard,      // **
    TypeFilter(String),     // | string
}
```

**支持的路径语法：**

- `.field` - 字段访问
- `[index]` - 数组索引访问
- `*` - 通配符匹配任意字段名
- `**` - 递归字段匹配
- `| type` - 类型过滤（可选）

### 2. 数据格式抽象（value::format）

统一接口设计，支持格式插件化扩展：

```rust
trait ValueFormat {
    fn parse(input: &str) -> Result<Value>;
    fn to_string(value: &Value) -> String;
}
```

**内置实现：**

- `JsonFormat` - 基于 `serde_json::Value`
- `YamlFormat` - 基于 `serde_yaml::Value`

### 3. 字段提取器（extractor.rs）

核心提取逻辑，支持：

```rust
fn extract<'a>(root: &'a Value, path: &[PathSegment]) -> Vec<&'a Value>
```

**功能特性：**

- 路径逐级匹配（Field, Index）
- 递归遍历（`**`）
- 通配符字段选择（`*`）
- 类型断言过滤（如 `| string`）

### 4. 字段更新器（updater.rs）

> ⚠️ **Feature Gate**: 需启用 `feature = "update"`

提供路径指定位置的更新功能：

```rust
fn update(root: &mut Value, path: &[PathSegment], new_value: Value) -> Result<()>
```

**更新能力：**

- 设置字段值
- 创建缺失路径
- 通配符批量更新（如 `.users[*].role = "admin"`）

## ⚙️ Feature 配置

在 `Cargo.toml` 中配置功能特性：

```toml
[features]
default = []
update = []  # 启用字段更新功能
```

## 🖥️ CLI 工具使用

### 基本命令

```bash
# 提取字段
datapath get -f input.yaml -p 'spec.template.spec.containers[0].image'

# 更新字段（需编译时启用 --features update）
datapath set -f input.yaml -p 'a.b[2].c' -v '"new_value"' > updated.yaml
```

### 参数说明

| 参数 | 长参数    | 说明                              |
| ---- | --------- | --------------------------------- |
| `-f` | `--file`  | 输入文件路径，省略时从 stdin 读取 |
| `-p` | `--path`  | 提取路径表达式（jq 风格语法）     |
| `-v` | `--value` | 要写入的新值（仅 `set` 命令使用） |

## 📚 使用示例

### 输入数据 (example.yaml)

```yaml
spec:
  template:
    spec:
      containers:
        - name: nginx
          image: nginx:1.25
        - name: redis
          image: redis:7.0
```

### 提取操作

```bash
# 提取单个值
$ datapath get -f example.yaml -p 'spec.template.spec.containers[0].image'
"nginx:1.25"

# 使用通配符提取多个值
$ datapath get -f example.yaml -p 'spec.template.spec.containers[*].image'
"nginx:1.25"
"redis:7.0"

# 递归查找所有 image 字段
$ datapath get -f example.yaml -p '**.image'
"nginx:1.25"
"redis:7.0"
```

## 🔌 格式扩展机制

通过 `ValueFormat` trait 和注册表机制支持新格式扩展：

```rust
// 示例：注册 TOML 格式支持
datapath.register_format("toml", TomlFormat::new());
```

## 🧪 测试策略

### 测试覆盖范围

| 测试类型     | 测试内容                                    | 状态 |
| ------------ | ------------------------------------------- | ---- |
| **单元测试** | PathParser、Extractor、Updater 模块独立测试 | ✅   |
| **集成测试** | CLI 输入输出、stdin 处理、文件编码等        | ✅   |
| **边界测试** | 空数组、null 值、混合类型结构处理           | ✅   |
| **错误测试** | 路径不存在、索引越界、类型不匹配等          | ✅   |

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test parser::path

# 带更新功能的测试
cargo test --features update
```

## 🔍 格式自动检测

为了提供更好的用户体验，DataPath 实现了智能格式检测：

```rust
fn detect_format(input: &str) -> Result<Box<dyn ValueFormat>> {
    let trimmed = input.trim_start();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        Ok(Box::new(JsonFormat))
    } else {
        Ok(Box::new(YamlFormat))
    }
}
```

## 📁 库模块组织

更新后的 `lib.rs` 结构：

```rust
#[macro_use]
mod macros;

pub mod extractor;
#[cfg(feature = "update")]
pub mod updater;
pub mod parser;
pub mod value;

// 重新导出便利接口
pub use macros::*;
pub use extractor::extract;
#[cfg(feature = "update")]
pub use updater::update;
```

## 🚀 快速开始

### 作为库使用

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
datapath = "0.1.0"
# 如需更新功能
datapath = { version = "0.1.0", features = ["update"] }
```

### 编译 CLI 工具

```bash
# 基本版本
cargo build --release

# 包含更新功能
cargo build --release --features update
```

## 🤝 贡献指南

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 打开 Pull Request

## 📄 许可证

本项目采用 MIT 或 Apache-2.0 双重许可证。详见 [LICENSE](LICENSE) 文件。

## 🔗 相关资源

- [jq 官方文档](https://stedolan.github.io/jq/) - 路径语法参考
- [serde 文档](https://serde.rs/) - Rust 序列化框架
- [winnow 文档](https://docs.rs/winnow/) - 解析器组合子
- [项目仓库](https://github.com/Thneonl/datapath) - GitHub 源码仓库

---

> **设计理念**: DataPath 致力于提供简单、高效、可扩展的结构化数据处理体验，无论是在命令行环境还是 Rust 应用程序中。
