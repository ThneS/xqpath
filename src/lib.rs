//! # XQPath
//!
//! 一个用于结构化数据（JSON/YAML/TOML/CSV）路径提取与更新的高性能 Rust 工具库。
//!
//! ## 快速开始
//!
//! ### 基本使用
//!
//! ```rust
//! use xqpath::{query, exists};
//! use serde_json::json;
//!
//! let yaml = r#"
//! user:
//!   name: Alice
//!   age: 30
//!   scores: [85, 92, 78]
//! "#;
//!
//! // 提取字段值
//! let name = query!(yaml, "user.name").unwrap();
//! assert_eq!(name[0], json!("Alice"));
//!
//! // 检查路径是否存在
//! let exists = exists!(yaml, "user.email").unwrap();
//! assert_eq!(exists, false);
//! ```
//!
//! ### 更新操作（需要 update feature）
//!
//! ```rust
//! #[cfg(feature = "update")]
//! use xqpath::update;
//! use serde_json::json;
//!
//! #[cfg(feature = "update")]
//! fn example() {
//!     let yaml = r#"user: {name: Alice, age: 30}"#;
//!     let updated = update!(yaml, "user.age", json!(31)).unwrap();
//! }
//! ```
//!
//! ## 特性
//!
//! - **路径提取**: 支持 `.field`、`[index]`、`*`、`**` 等 jq 风格路径
//! - **格式支持**: 自动检测和解析 JSON/YAML 格式
//! - **通配符**: 支持字段和递归匹配
//! - **类型过滤**: 支持类型断言和过滤
//! - **更新功能**: 支持路径指定位置的更新（feature gate）
//! - **轻量级**: 最小依赖集，高性能

#[macro_use]
mod macros;

// 核心模块
pub mod extractor;
pub mod parser;
#[cfg(feature = "update")]
pub mod updater;
pub mod value;

// 重新导出主要类型和函数
pub use extractor::{
    extract, ConfigurableExtractor, ExtractError, Extractor, ExtractorConfig,
};

#[cfg(feature = "update")]
pub use updater::{
    update, ConfigurableUpdater, UpdateError, Updater, UpdaterConfig,
};

pub use parser::{
    ast::{ComparisonOp, ExpressionComplexity, LogicalOp, PathExpression},
    evaluation::{
        evaluate_path_expression, EvaluationError, ExpressionEvaluator,
    },
    functions::{AdvancedBuiltinFunction, BuiltinFunction, FunctionRegistry},
    parsing::{parse_path_expression, ExpressionParser},
    path::{parse_path, ParseError, PathSegment},
};

pub use value::format::{
    detect_format, FormatError, FormatRegistry, JsonFormat, ValueFormat,
    YamlFormat,
};

pub use value::json::{JsonPath, JsonSupport};
pub use value::yaml::{YamlFormatter, YamlSpecialValues, YamlSupport};

// Note: Macros are automatically available when using the crate

/// 库版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 检查是否启用了更新功能
pub const fn has_update_feature() -> bool {
    cfg!(feature = "update")
}
