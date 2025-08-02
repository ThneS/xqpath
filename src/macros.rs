/// 便利宏，用于从结构化数据中提取字段
///
/// # 参数
/// - `$data`: 输入的数据字符串（JSON 或 YAML 格式）
/// - `$path`: 路径表达式字符串（如 ".user.name" 或 ".users\[0\].email"）
///
/// # 返回值
/// 返回 `Result<Vec<serde_json::Value>, Box<dyn std::error::Error>>`
///
/// # 示例
/// ```rust
/// use xqpath::query;
/// use serde_json::json;
///
/// let yaml = r#"
/// user:
///   name: Alice
///   age: 30
/// "#;
///
/// let result = query!(yaml, "user.name").unwrap();
/// assert_eq!(result[0], json!("Alice"));
/// ```
#[macro_export]
macro_rules! query {
    ($data:expr, $path:expr) => {{
        use $crate::extractor::extract;
        use $crate::parser::path::parse_path;
        use $crate::value::format::detect_format;

        (|| -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
            let format = detect_format(&$data)?;
            let parsed = format.parse(&$data)?;
            let path = parse_path($path)?;
            let values = extract(&parsed, &path)?;

            // 将引用转换为拥有的值
            let owned_values: Vec<serde_json::Value> =
                values.into_iter().map(|v| v.clone()).collect();

            Ok(owned_values)
        })()
    }};
}

/// 便利宏，用于在结构化数据中设置字段值
///
/// ⚠️ **注意**: 此宏仅在启用 `update` feature 时可用
///
/// # 参数
/// - `$data`: 输入的数据字符串（JSON 或 YAML 格式）
/// - `$path`: 路径表达式字符串（如 ".user.name" 或 ".users\[0\].email"）
/// - `$value`: 要设置的新值（必须是 `serde_json::Value` 类型）
///
/// # 返回值
/// 返回 `Result<String, Box<dyn std::error::Error>>`，包含更新后的数据字符串
///
/// # 示例
/// ```rust
/// #[cfg(feature = "update")]
/// use xqpath::update;
/// use serde_json::json;
///
/// #[cfg(feature = "update")]
/// {
///     let yaml = r#"
///     user:
///       name: Alice
///       age: 30
///     "#;
///
///     let updated = update!(yaml, "user.age", json!(31)).unwrap();
///     // updated 现在包含更新后的 YAML 字符串
/// }
/// ```
#[cfg(feature = "update")]
#[macro_export]
macro_rules! update {
    ($data:expr, $path:expr, $value:expr) => {{
        use $crate::parser::path::parse_path;
        use $crate::updater::update;
        use $crate::value::format::detect_format;

        (|| -> Result<String, Box<dyn std::error::Error>> {
            let format = detect_format(&$data)?;
            let mut parsed = format.parse(&$data)?;
            let path = parse_path($path)?;
            update(&mut parsed, &path, $value)?;
            Ok(format.to_string(&parsed)?)
        })()
    }};
}

/// 便利宏，用于检查路径是否存在
///
/// # 参数
/// - `$data`: 输入的数据字符串（JSON 或 YAML 格式）
/// - `$path`: 路径表达式字符串
///
/// # 返回值
/// 返回 `Result<bool, Box<dyn std::error::Error>>`
///
/// # 示例
/// ```rust
/// use xqpath::exists;
///
/// let json = r#"{"user": {"name": "Alice"}}"#;
/// let exists = exists!(json, "user.name").unwrap();
/// assert_eq!(exists, true);
///
/// let missing = exists!(json, "user.email").unwrap();
/// assert_eq!(missing, false);
/// ```
#[macro_export]
macro_rules! exists {
    ($data:expr, $path:expr) => {{
        use $crate::extractor::extract;
        use $crate::parser::path::parse_path;
        use $crate::value::format::detect_format;

        (|| -> Result<bool, Box<dyn std::error::Error>> {
            let format = detect_format(&$data)?;
            let parsed = format.parse(&$data)?;
            let path = parse_path($path)?;
            let values = extract(&parsed, &path)?;
            Ok(!values.is_empty())
        })()
    }};
}

/// 便利宏，用于获取路径对应值的类型
///
/// # 参数
/// - `$data`: 输入的数据字符串（JSON 或 YAML 格式）
/// - `$path`: 路径表达式字符串
///
/// # 返回值
/// 返回 `Result<Vec<String>, Box<dyn std::error::Error>>`，包含每个匹配值的类型名称
///
/// # 示例
/// ```rust
/// use xqpath::get_type;
///
/// let json = r#"{"name": "Alice", "age": 30, "active": true}"#;
/// let types = get_type!(json, "*").unwrap();
/// // types 可能包含 ["string", "number", "boolean"]
/// ```
#[macro_export]
macro_rules! get_type {
    ($data:expr, $path:expr) => {{
        use $crate::extractor::extract;
        use $crate::parser::path::parse_path;
        use $crate::value::format::detect_format;
        use $crate::value::json::JsonSupport;

        (|| -> Result<Vec<String>, Box<dyn std::error::Error>> {
            let format = detect_format(&$data)?;
            let parsed = format.parse(&$data)?;
            let path = parse_path($path)?;
            let values = extract(&parsed, &path)?;

            let types: Vec<String> = values
                .into_iter()
                .map(|v| JsonSupport::get_type_name(v).to_string())
                .collect();

            Ok(types)
        })()
    }};
}

/// 便利宏，用于计算匹配路径的值的数量
///
/// # 参数
/// - `$data`: 输入的数据字符串（JSON 或 YAML 格式）
/// - `$path`: 路径表达式字符串
///
/// # 返回值
/// 返回 `Result<usize, Box<dyn std::error::Error>>`
///
/// # 示例
/// ```rust
/// use xqpath::count;
///
/// let json = r#"{"users": [{"name": "Alice"}, {"name": "Bob"}]}"#;
/// let count = count!(json, "users[*]").unwrap();
/// assert_eq!(count, 2);
/// ```
#[macro_export]
macro_rules! count {
    ($data:expr, $path:expr) => {{
        use $crate::extractor::extract;
        use $crate::parser::path::parse_path;
        use $crate::value::format::detect_format;

        (|| -> Result<usize, Box<dyn std::error::Error>> {
            let format = detect_format(&$data)?;
            let parsed = format.parse(&$data)?;
            let path = parse_path($path)?;
            let values = extract(&parsed, &path)?;
            Ok(values.len())
        })()
    }};
}

/// 便利宏，用于从多种格式的数据中提取值，并指定输出格式
///
/// # 参数
/// - `$data`: 输入的数据字符串
/// - `$path`: 路径表达式字符串
/// - `$output_format`: 输出格式 ("json" 或 "yaml")
///
/// # 返回值
/// 返回 `Result<String, Box<dyn std::error::Error>>`，包含指定格式的字符串
///
/// # 示例
/// ```rust
/// use xqpath::extract;
///
/// let yaml = r#"
/// user:
///   name: Alice
///   age: 30
/// "#;
///
/// let json_output = extract!(yaml, "user", "json").unwrap();
/// // json_output 包含 JSON 格式的用户数据
/// ```
#[macro_export]
macro_rules! extract {
    ($data:expr, $path:expr, $output_format:expr) => {{
        use $crate::extractor::extract;
        use $crate::parser::path::parse_path;
        use $crate::value::format::{
            detect_format, JsonFormat, ValueFormat, YamlFormat,
        };

        (|| -> Result<String, Box<dyn std::error::Error>> {
            let input_format = detect_format(&$data)?;
            let parsed = input_format.parse(&$data)?;
            let path = parse_path($path)?;
            let values = extract(&parsed, &path)?;

            if values.is_empty() {
                return Ok(String::new());
            }

            // 如果只有一个值，直接返回该值；否则返回数组
            let result_value = if values.len() == 1 {
                values[0].clone()
            } else {
                serde_json::Value::Array(
                    values.into_iter().map(|v| v.clone()).collect(),
                )
            };

            // 根据指定格式输出
            let output_format: Box<dyn ValueFormat> =
                match $output_format.to_lowercase().as_str() {
                    "json" => Box::new(JsonFormat),
                    "yaml" | "yml" => Box::new(YamlFormat),
                    _ => {
                        return Err(format!(
                            "Unsupported output format: {}",
                            $output_format
                        )
                        .into())
                    }
                };

            Ok(output_format.to_string(&result_value)?)
        })()
    }};
}

/// 便利宏，用于从结构化数据中提取单个值
///
/// # 参数
/// - `$data`: 输入的数据字符串（JSON 或 YAML 格式）
/// - `$path`: 路径表达式字符串
///
/// # 返回值
/// 返回 `Result<Option<serde_json::Value>, Box<dyn std::error::Error>>`
///
/// # 示例
/// ```rust
/// use xqpath::query_one;
/// use serde_json::json;
///
/// let json = r#"{"user": {"name": "Alice", "age": 30}}"#;
/// let name = query_one!(json, "user.name").unwrap();
/// assert_eq!(name, Some(json!("Alice")));
///
/// let missing = query_one!(json, "user.email").unwrap();
/// assert_eq!(missing, None);
/// ```
#[macro_export]
macro_rules! query_one {
    ($data:expr, $path:expr) => {{
        use $crate::extractor::extract;
        use $crate::parser::path::parse_path;
        use $crate::value::format::detect_format;

        (|| -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
            let format = detect_format(&$data)?;
            let parsed = format.parse(&$data)?;
            let path = parse_path($path)?;
            let values = extract(&parsed, &path)?;

            Ok(values.first().map(|v| (*v).clone()))
        })()
    }};
}

/// 便利宏，用于从结构化数据中提取值，如果不存在则返回默认值
///
/// # 参数
/// - `$data`: 输入的数据字符串（JSON 或 YAML 格式）
/// - `$path`: 路径表达式字符串
/// - `$default`: 默认值（必须是 `serde_json::Value` 类型）
///
/// # 返回值
/// 返回 `Result<serde_json::Value, Box<dyn std::error::Error>>`
///
/// # 示例
/// ```rust
/// use xqpath::query_or_default;
/// use serde_json::json;
///
/// let json = r#"{"user": {"name": "Alice"}}"#;
/// let name = query_or_default!(json, "user.name", json!("Unknown")).unwrap();
/// assert_eq!(name, json!("Alice"));
///
/// let email = query_or_default!(json, "user.email", json!("no-email")).unwrap();
/// assert_eq!(email, json!("no-email"));
/// ```
#[macro_export]
macro_rules! query_or_default {
    ($data:expr, $path:expr, $default:expr) => {{
        use $crate::extractor::extract;
        use $crate::parser::path::parse_path;
        use $crate::value::format::detect_format;

        (|| -> Result<serde_json::Value, Box<dyn std::error::Error>> {
            let format = detect_format(&$data)?;
            let parsed = format.parse(&$data)?;
            let path = parse_path($path)?;
            let values = extract(&parsed, &path)?;

            Ok(values.first().map(|v| (*v).clone()).unwrap_or($default))
        })()
    }};
}

/// 便利宏，用于从结构化数据中提取值并尝试转换为指定类型
///
/// # 参数
/// - `$data`: 输入的数据字符串（JSON 或 YAML 格式）
/// - `$path`: 路径表达式字符串
/// - `$type`: 目标类型
///
/// # 返回值
/// 返回 `Result<Option<$type>, Box<dyn std::error::Error>>`
///
/// # 示例
/// ```rust
/// use xqpath::query_as_type;
/// use serde_json::json;
///
/// let json = r#"{"user": {"name": "Alice", "age": 30}}"#;
/// let age: Option<u32> = query_as_type!(json, "user.age", u32).unwrap();
/// assert_eq!(age, Some(30));
///
/// let name: Option<String> = query_as_type!(json, "user.name", String).unwrap();
/// assert_eq!(name, Some("Alice".to_string()));
/// ```
#[macro_export]
macro_rules! query_as_type {
    ($data:expr, $path:expr, $type:ty) => {{
        use serde_json::from_value;
        use $crate::extractor::extract;
        use $crate::parser::path::parse_path;
        use $crate::value::format::detect_format;

        (|| -> Result<Option<$type>, Box<dyn std::error::Error>> {
            let format = detect_format(&$data)?;
            let parsed = format.parse(&$data)?;
            let path = parse_path($path)?;
            let values = extract(&parsed, &path)?;

            if let Some(value) = values.first() {
                match from_value::<$type>((*value).clone()) {
                    Ok(typed_value) => Ok(Some(typed_value)),
                    Err(_) => Ok(None),
                }
            } else {
                Ok(None)
            }
        })()
    }};
}

/// 便利宏，用于从结构化数据中查询多个路径
///
/// # 参数
/// - `$data`: 输入的数据字符串（JSON 或 YAML 格式）
/// - `$($path:expr),+`: 多个路径表达式字符串
///
/// # 返回值
/// 返回 `Result<Vec<Option<serde_json::Value>>, Box<dyn std::error::Error>>`
///
/// # 示例
/// ```rust
/// use xqpath::query_multi;
/// use serde_json::json;
///
/// let json = r#"{"user": {"name": "Alice", "age": 30, "email": "alice@example.com"}}"#;
/// let results = query_multi!(json, "user.name", "user.age", "user.email").unwrap();
/// assert_eq!(results[0], Some(json!("Alice")));
/// assert_eq!(results[1], Some(json!(30)));
/// assert_eq!(results[2], Some(json!("alice@example.com")));
/// ```
#[macro_export]
macro_rules! query_multi {
    ($data:expr, $($path:expr),+) => {{
        use $crate::extractor::extract;
        use $crate::parser::path::parse_path;
        use $crate::value::format::detect_format;

        (|| -> Result<Vec<Option<serde_json::Value>>, Box<dyn std::error::Error>> {
            let format = detect_format(&$data)?;
            let parsed = format.parse(&$data)?;
            let mut results = Vec::new();

            $(
                let path = parse_path($path)?;
                let values = extract(&parsed, &path)?;
                results.push(values.first().map(|v| (*v).clone()));
            )+

            Ok(results)
        })()
    }};
}

/// 便利宏，用于检查结构化数据中是否存在所有指定路径
///
/// # 参数
/// - `$data`: 输入的数据字符串（JSON 或 YAML 格式）
/// - `$($path:expr),+`: 多个路径表达式字符串
///
/// # 返回值
/// 返回 `Result<bool, Box<dyn std::error::Error>>`，只有所有路径都存在时才返回true
///
/// # 示例
/// ```rust
/// use xqpath::exists_all;
///
/// let json = r#"{"user": {"name": "Alice", "age": 30}}"#;
/// let all_exist = exists_all!(json, "user.name", "user.age").unwrap();
/// assert_eq!(all_exist, true);
///
/// let partial_exist = exists_all!(json, "user.name", "user.email").unwrap();
/// assert_eq!(partial_exist, false);
/// ```
#[macro_export]
macro_rules! exists_all {
    ($data:expr, $($path:expr),+) => {{
        use $crate::extractor::extract;
        use $crate::parser::path::parse_path;
        use $crate::value::format::detect_format;

        (|| -> Result<bool, Box<dyn std::error::Error>> {
            let format = detect_format(&$data)?;
            let parsed = format.parse(&$data)?;

            $(
                let path = parse_path($path)?;
                let values = extract(&parsed, &path)?;
                if values.is_empty() {
                    return Ok(false);
                }
            )+

            Ok(true)
        })()
    }};
}

/// 便利宏，用于检查结构化数据中是否存在任意一个指定路径
///
/// # 参数
/// - `$data`: 输入的数据字符串（JSON 或 YAML 格式）
/// - `$($path:expr),+`: 多个路径表达式字符串
///
/// # 返回值
/// 返回 `Result<bool, Box<dyn std::error::Error>>`，只要有一个路径存在就返回true
///
/// # 示例
/// ```rust
/// use xqpath::exists_any;
///
/// let json = r#"{"user": {"name": "Alice"}}"#;
/// let any_exist = exists_any!(json, "user.name", "user.email").unwrap();
/// assert_eq!(any_exist, true);
///
/// let none_exist = exists_any!(json, "user.phone", "user.address").unwrap();
/// assert_eq!(none_exist, false);
/// ```
#[macro_export]
macro_rules! exists_any {
    ($data:expr, $($path:expr),+) => {{
        use $crate::extractor::extract;
        use $crate::parser::path::parse_path;
        use $crate::value::format::detect_format;

        (|| -> Result<bool, Box<dyn std::error::Error>> {
            let format = detect_format(&$data)?;
            let parsed = format.parse(&$data)?;

            $(
                let path = parse_path($path)?;
                let values = extract(&parsed, &path)?;
                if !values.is_empty() {
                    return Ok(true);
                }
            )+

            Ok(false)
        })()
    }};
}

/// 便利宏，用于从结构化数据中提取值并转换为字符串
///
/// # 参数
/// - `$data`: 输入的数据字符串（JSON 或 YAML 格式）
/// - `$path`: 路径表达式字符串
///
/// # 返回值
/// 返回 `Result<Option<String>, Box<dyn std::error::Error>>`
///
/// # 示例
/// ```rust
/// use xqpath::query_string;
///
/// let json = r#"{"user": {"name": "Alice", "age": 30}}"#;
/// let name = query_string!(json, "user.name").unwrap();
/// assert_eq!(name, Some("Alice".to_string()));
///
/// let age = query_string!(json, "user.age").unwrap();
/// assert_eq!(age, Some("30".to_string()));
/// ```
#[macro_export]
macro_rules! query_string {
    ($data:expr, $path:expr) => {{
        use $crate::extractor::extract;
        use $crate::parser::path::parse_path;
        use $crate::value::format::detect_format;

        (|| -> Result<Option<String>, Box<dyn std::error::Error>> {
            let format = detect_format(&$data)?;
            let parsed = format.parse(&$data)?;
            let path = parse_path($path)?;
            let values = extract(&parsed, &path)?;

            if let Some(value) = values.first() {
                let string_value = match value {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Null => "null".to_string(),
                    _ => serde_json::to_string(value)?
                        .trim_matches('"')
                        .to_string(),
                };
                Ok(Some(string_value))
            } else {
                Ok(None)
            }
        })()
    }};
}

/// 便利宏，用于从结构化数据中查询数组长度
///
/// # 参数
/// - `$data`: 输入的数据字符串（JSON 或 YAML 格式）
/// - `$path`: 路径表达式字符串（应指向数组）
///
/// # 返回值
/// 返回 `Result<Option<usize>, Box<dyn std::error::Error>>`
///
/// # 示例
/// ```rust
/// use xqpath::query_length;
///
/// let json = r#"{"users": [{"name": "Alice"}, {"name": "Bob"}, {"name": "Carol"}]}"#;
/// let length = query_length!(json, "users").unwrap();
/// assert_eq!(length, Some(3));
///
/// let missing = query_length!(json, "groups").unwrap();
/// assert_eq!(missing, None);
/// ```
#[macro_export]
macro_rules! query_length {
    ($data:expr, $path:expr) => {{
        use $crate::extractor::extract;
        use $crate::parser::path::parse_path;
        use $crate::value::format::detect_format;

        (|| -> Result<Option<usize>, Box<dyn std::error::Error>> {
            let format = detect_format(&$data)?;
            let parsed = format.parse(&$data)?;
            let path = parse_path($path)?;
            let values = extract(&parsed, &path)?;

            if let Some(value) = values.first() {
                match value {
                    serde_json::Value::Array(arr) => Ok(Some(arr.len())),
                    serde_json::Value::Object(obj) => Ok(Some(obj.len())),
                    serde_json::Value::String(s) => Ok(Some(s.len())),
                    _ => Ok(None), // 其他类型没有长度概念
                }
            } else {
                Ok(None)
            }
        })()
    }};
}

// ===== 调试宏 (v1.4.1+) =====

/// 带调试信息的查询宏
///
/// ⚠️ **注意**: 此宏仅在启用 `debug` feature 时可用
///
/// # 参数
/// - `$data`: 输入的数据字符串（JSON 或 YAML 格式）
/// - `$path`: 路径表达式字符串
/// - `$debug_callback`: 调试信息回调函数
///
/// # 示例
/// ```rust
/// #[cfg(feature = "debug")]
/// use xqpath::query_debug;
/// use serde_json::json;
///
/// #[cfg(feature = "debug")]
/// fn example() {
///     let data = r#"{"users": [{"name": "Alice"}, {"name": "Bob"}]}"#;
///     let result = query_debug!(data, ".users[*].name", |debug_info: &xqpath::debug::DebugInfo| {
///         println!("解析耗时: {:?}", debug_info.parse_duration);
///         println!("执行路径: {}", debug_info.execution_path);
///     }).unwrap();
/// }
/// ```
#[cfg(feature = "debug")]
#[macro_export]
macro_rules! query_debug {
    ($data:expr, $path:expr, $debug_callback:expr) => {{
        use std::time::Instant;
        use $crate::debug::{DebugContext, DebugInfo};
        use $crate::extractor::extract;
        use $crate::parser::path::parse_path;
        use $crate::value::format::detect_format;

        (|| -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
            let _debug_ctx = DebugContext::new()
                .with_timing(true)
                .with_path_tracing(true);

            let start_time = Instant::now();

            let format = detect_format(&$data)?;
            let parsed = format.parse(&$data)?;
            let parse_duration = start_time.elapsed();

            let exec_start = Instant::now();
            let path = parse_path($path)?;
            let values = extract(&parsed, &path)?;
            let exec_duration = exec_start.elapsed();

            let mut debug_info = DebugInfo::default();
            debug_info.parse_duration = Some(parse_duration);
            debug_info.execution_duration = Some(exec_duration);
            debug_info.execution_path = format!("query({})", $path);
            debug_info.queries_executed = 1;

            $debug_callback(&debug_info);

            let owned_values: Vec<serde_json::Value> =
                values.into_iter().map(|v| v.clone()).collect();

            Ok(owned_values)
        })()
    }};
}

/// 带性能跟踪的查询宏
///
/// ⚠️ **注意**: 此宏仅在启用 `debug` feature 时可用
///
/// # 返回值
/// 返回 `(Result<Vec<serde_json::Value>, Error>, TimingStats)`
///
/// # 示例
/// ```rust
/// #[cfg(feature = "debug")]
/// use xqpath::trace_query;
/// use serde_json::json;
///
/// #[cfg(feature = "debug")]
/// fn example() {
///     let data = r#"{"users": [{"name": "Alice"}]}"#;
///     let (result, stats) = trace_query!(data, ".users[*].name").unwrap();
///     println!("总耗时: {:?}", stats.duration);
/// }
/// ```
#[cfg(feature = "debug")]
#[macro_export]
macro_rules! trace_query {
    ($data:expr, $path:expr) => {{
        use $crate::extractor::extract;
        use $crate::parser::path::parse_path;
        use $crate::value::format::detect_format;
        use $crate::debug::TimingStats;
        use std::time::Instant;

        (|| -> Result<(Vec<serde_json::Value>, TimingStats), Box<dyn std::error::Error>> {
            let start_time = Instant::now();
            let _start_memory = 0; // TODO: 实际内存跟踪

            let format = detect_format(&$data)?;
            let parsed = format.parse(&$data)?;
            let path = parse_path($path)?;
            let values = extract(&parsed, &path)?;

            let duration = start_time.elapsed();
            let memory_used = 0; // TODO: 计算实际内存使用

            let owned_values: Vec<serde_json::Value> =
                values.into_iter().map(|v| v.clone()).collect();

            let stats = TimingStats {
                duration,
                memory_used,
                peak_memory: memory_used,
            };

            Ok((owned_values, stats))
        })()
    }};
}

/// 带性能分析的查询宏
///
/// ⚠️ **注意**: 此宏仅在启用 `profiling` feature 时可用
///
/// # 返回值
/// 返回 `(Result<Vec<serde_json::Value>, Error>, ProfileReport)`
///
/// # 示例
/// ```rust
/// #[cfg(feature = "profiling")]
/// use xqpath::query_with_profile;
/// use serde_json::json;
///
/// #[cfg(feature = "profiling")]
/// fn example() {
///     let data = r#"{"users": [{"name": "Alice"}]}"#;
///     let (result, profile) = query_with_profile!(data, ".users[*].name").unwrap();
///     println!("执行时间: {:?}", profile.execution_time);
/// }
/// ```
#[cfg(feature = "profiling")]
#[macro_export]
macro_rules! query_with_profile {
    ($data:expr, $path:expr) => {{
        use $crate::extractor::extract;
        use $crate::parser::path::parse_path;
        use $crate::value::format::detect_format;
        use std::time::Instant;

        #[cfg(feature = "profiling")]
        {
            (|| -> Result<(Vec<serde_json::Value>, $crate::debug::profiler::ProfileReport), Box<dyn std::error::Error>> {
                use $crate::debug::profiler::PerformanceMonitor;

                let mut monitor = PerformanceMonitor::new();
                monitor.start();

                let format = detect_format(&$data)?;
                let parsed = format.parse(&$data)?;
                let path = parse_path($path)?;
                let values = extract(&parsed, &path)?;

                let profile = monitor.stop();

                let owned_values: Vec<serde_json::Value> =
                    values.into_iter().map(|v| v.clone()).collect();

                Ok((owned_values, profile))
            })()
        }

        #[cfg(not(feature = "profiling"))]
        {
            (|| -> Result<(Vec<serde_json::Value>, $crate::debug::TimingStats), Box<dyn std::error::Error>> {
                let start_time = Instant::now();

                let format = detect_format(&$data)?;
                let parsed = format.parse(&$data)?;
                let path = parse_path($path)?;
                let values = extract(&parsed, &path)?;

                let execution_time = start_time.elapsed();

                let owned_values: Vec<serde_json::Value> =
                    values.into_iter().map(|v| v.clone()).collect();

                // 当 profiling feature 未启用时，返回基础的 TimingStats
                let profile = $crate::debug::TimingStats {
                    duration: execution_time,
                    memory_used: 0,
                    peak_memory: 0,
                };

                Ok((owned_values, profile))
            })()
        }
    }};
}

// ===== v1.4.2 性能分析宏 =====

/// 内存分析宏 - 专注于内存使用监控
///
/// ⚠️ **注意**: 此宏仅在启用 `profiling` feature 时可用
///
/// # 示例
/// ```rust
/// #[cfg(feature = "profiling")]
/// use xqpath::query_memory;
/// use serde_json::json;
///
/// #[cfg(feature = "profiling")]
/// fn example() {
///     let data = r#"{"users": [{"name": "Alice"}]}"#;
///     let (result, memory_report) = query_memory!(data, ".users[*].name").unwrap();
///     println!("峰值内存: {} MB", memory_report.peak_memory_bytes as f64 / 1024.0 / 1024.0);
/// }
/// ```
#[cfg(feature = "profiling")]
#[macro_export]
macro_rules! query_memory {
    ($data:expr, $path:expr) => {{
        use $crate::debug::profiler::MemoryProfiler;
        use $crate::extractor::extract;
        use $crate::parser::path::parse_path;
        use $crate::value::format::detect_format;

        (|| -> Result<(Vec<serde_json::Value>, $crate::debug::profiler::ProfileReport), Box<dyn std::error::Error>> {
            let mut profiler = MemoryProfiler::new();
            profiler.start();

            let format = detect_format(&$data)?;
            let parsed = format.parse(&$data)?;
            let path = parse_path($path)?;
            let values = extract(&parsed, &path)?;

            let memory_report = profiler.stop();

            let owned_values: Vec<serde_json::Value> =
                values.into_iter().map(|v| v.clone()).collect();

            Ok((owned_values, memory_report))
        })()
    }};
}

/// 基准测试宏 - 自动运行性能基准测试
///
/// ⚠️ **注意**: 此宏仅在启用 `benchmark` feature 时可用
///
/// # 示例
/// ```rust
/// #[cfg(feature = "benchmark")]
/// use xqpath::benchmark_query;
/// use serde_json::json;
///
/// #[cfg(feature = "benchmark")]
/// fn example() {
///     let data = r#"{"users": [{"name": "Alice"}]}"#;
///     let (result, benchmark_result) = benchmark_query!(data, ".users[*].name", 100).unwrap();
///     println!("平均执行时间: {:?}", benchmark_result.mean_time);
/// }
/// ```
#[cfg(feature = "benchmark")]
#[macro_export]
macro_rules! benchmark_query {
    ($data:expr, $path:expr, $iterations:expr) => {{
        use $crate::debug::benchmark::{BenchmarkSuite, BenchmarkConfig};
        use $crate::extractor::extract;
        use $crate::parser::path::parse_path;
        use $crate::value::format::detect_format;
        use std::time::Duration;

        (|| -> Result<(Vec<serde_json::Value>, $crate::debug::benchmark::BenchmarkResult), Box<dyn std::error::Error>> {
            // 先执行一次获取结果
            let format = detect_format(&$data)?;
            let parsed = format.parse(&$data)?;
            let path = parse_path($path)?;
            let values = extract(&parsed, &path)?;
            let owned_values: Vec<serde_json::Value> =
                values.into_iter().map(|v| v.clone()).collect();

            // 设置基准测试
            let config = BenchmarkConfig {
                warmup_iterations: ($iterations as f64 * 0.1) as usize,
                test_iterations: $iterations,
                min_test_time: Duration::from_millis(10),
                max_test_time: Duration::from_secs(30),
            };

            let mut suite = BenchmarkSuite::with_config(config);

            let data_clone = $data.to_string();
            let path_clone = $path.to_string();

            suite.add_test("query_benchmark", move || {
                let format = detect_format(&data_clone)?;
                let parsed = format.parse(&data_clone)?;
                let path = parse_path(&path_clone)?;
                let _values = extract(&parsed, &path)?;
                Ok(())
            });

            let mut results = suite.run()?;
            let benchmark_result = results.pop().unwrap();

            Ok((owned_values, benchmark_result))
        })()
    }};
}

/// 完整性能分析宏 - 综合性能、内存、基准测试
///
/// ⚠️ **注意**: 此宏仅在启用 `profiling` feature 时可用
///
/// # 示例
/// ```rust
/// #[cfg(feature = "profiling")]
/// use xqpath::profile_complete;
/// use serde_json::json;
///
/// #[cfg(feature = "profiling")]
/// fn example() {
///     let data = r#"{"users": [{"name": "Alice"}]}"#;
///     let (result, profile) = profile_complete!(data, ".users[*].name").unwrap();
///     println!("完整性能报告: {}", profile.summary());
/// }
/// ```
#[cfg(feature = "profiling")]
#[macro_export]
macro_rules! profile_complete {
    ($data:expr, $path:expr) => {{
        use $crate::debug::profiler::PerformanceMonitor;
        use $crate::extractor::extract;
        use $crate::parser::path::parse_path;
        use $crate::value::format::detect_format;

        (|| -> Result<(Vec<serde_json::Value>, $crate::debug::profiler::ProfileReport), Box<dyn std::error::Error>> {
            let mut monitor = PerformanceMonitor::new();
            monitor.start();

            let format = detect_format(&$data)?;
            let parsed = format.parse(&$data)?;
            let path = parse_path($path)?;
            let values = extract(&parsed, &path)?;

            let mut profile = monitor.stop();

            // 添加路径复杂度分析
            let path_complexity = $path.split('.').count() + $path.matches('[').count() * 2;
            profile.add_metric("path_complexity", path_complexity as f64);

            // 添加数据大小分析
            let data_size = $data.len();
            profile.add_metric("data_size_kb", data_size as f64 / 1024.0);

            // 添加结果数量分析
            profile.add_metric("result_count", values.len() as f64);

            let owned_values: Vec<serde_json::Value> =
                values.into_iter().map(|v| v.clone()).collect();

            Ok((owned_values, profile))
        })()
    }};
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn test_query_macro() {
        let yaml = r#"
user:
  name: Alice
  age: 30
"#;

        let result = query!(yaml, "user.name").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], json!("Alice"));
    }

    #[test]
    fn test_exists_macro() {
        let json = r#"{"user": {"name": "Alice"}}"#;

        let exists = exists!(json, "user.name").unwrap();
        assert!(exists);

        let missing = exists!(json, "user.email").unwrap();
        assert!(!missing);
    }

    #[test]
    fn test_get_type_macro() {
        let json = r#"{"name": "Alice", "age": 30, "active": true}"#;

        let types = get_type!(json, "name").unwrap();
        assert_eq!(types, vec!["string"]);

        let age_types = get_type!(json, "age").unwrap();
        assert_eq!(age_types, vec!["number"]);
    }

    #[test]
    fn test_count_macro() {
        let json = r#"{"users": [{"name": "Alice"}, {"name": "Bob"}]}"#;

        let count = count!(json, "users[*]").unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_extract_macro() {
        let yaml = r#"
user:
  name: Alice
  age: 30
"#;

        match extract!(yaml, "user", "json") {
            Ok(json_output) => {
                assert!(json_output.contains("Alice"));
                assert!(json_output.contains("30"));
            }
            Err(e) => panic!("Test failed: {e}"),
        }
    }

    #[cfg(feature = "update")]
    #[test]
    fn test_update_macro() {
        let yaml = r#"
user:
  name: Alice
  age: 30
"#;

        let updated = update!(yaml, "user.age", json!(31)).unwrap();
        assert!(updated.contains("31"));
    }

    #[test]
    fn test_query_one_macro() {
        let json = r#"{"user": {"name": "Alice", "age": 30}}"#;

        let name = query_one!(json, "user.name").unwrap();
        assert_eq!(name, Some(json!("Alice")));

        let missing = query_one!(json, "user.email").unwrap();
        assert_eq!(missing, None);
    }

    #[test]
    fn test_query_or_default_macro() {
        let json = r#"{"user": {"name": "Alice"}}"#;

        let name =
            query_or_default!(json, "user.name", json!("Unknown")).unwrap();
        assert_eq!(name, json!("Alice"));

        let email =
            query_or_default!(json, "user.email", json!("no-email")).unwrap();
        assert_eq!(email, json!("no-email"));
    }

    #[test]
    fn test_query_as_type_macro() {
        let json = r#"{"user": {"name": "Alice", "age": 30, "score": 95.5}}"#;

        let age: Option<u32> = query_as_type!(json, "user.age", u32).unwrap();
        assert_eq!(age, Some(30));

        let name: Option<String> =
            query_as_type!(json, "user.name", String).unwrap();
        assert_eq!(name, Some("Alice".to_string()));

        let score: Option<f64> =
            query_as_type!(json, "user.score", f64).unwrap();
        assert_eq!(score, Some(95.5));

        let missing: Option<String> =
            query_as_type!(json, "user.email", String).unwrap();
        assert_eq!(missing, None);
    }

    #[test]
    fn test_query_multi_macro() {
        let json = r#"{"user": {"name": "Alice", "age": 30, "email": "alice@example.com"}}"#;

        let results =
            query_multi!(json, "user.name", "user.age", "user.email").unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], Some(json!("Alice")));
        assert_eq!(results[1], Some(json!(30)));
        assert_eq!(results[2], Some(json!("alice@example.com")));

        let partial_results =
            query_multi!(json, "user.name", "user.missing").unwrap();
        assert_eq!(partial_results.len(), 2);
        assert_eq!(partial_results[0], Some(json!("Alice")));
        assert_eq!(partial_results[1], None);
    }

    #[test]
    fn test_exists_all_macro() {
        let json = r#"{"user": {"name": "Alice", "age": 30}}"#;

        let all_exist = exists_all!(json, "user.name", "user.age").unwrap();
        assert!(all_exist);

        let partial_exist =
            exists_all!(json, "user.name", "user.email").unwrap();
        assert!(!partial_exist);

        let none_exist =
            exists_all!(json, "user.phone", "user.address").unwrap();
        assert!(!none_exist);
    }

    #[test]
    fn test_exists_any_macro() {
        let json = r#"{"user": {"name": "Alice"}}"#;

        let any_exist = exists_any!(json, "user.name", "user.email").unwrap();
        assert!(any_exist);

        let none_exist =
            exists_any!(json, "user.phone", "user.address").unwrap();
        assert!(!none_exist);

        let single_exist = exists_any!(json, "user.name").unwrap();
        assert!(single_exist);
    }

    #[test]
    fn test_query_string_macro() {
        let json = r#"{"user": {"name": "Alice", "age": 30, "active": true}}"#;

        let name = query_string!(json, "user.name").unwrap();
        assert_eq!(name, Some("Alice".to_string()));

        let age = query_string!(json, "user.age").unwrap();
        assert_eq!(age, Some("30".to_string()));

        let active = query_string!(json, "user.active").unwrap();
        assert_eq!(active, Some("true".to_string()));

        let missing = query_string!(json, "user.email").unwrap();
        assert_eq!(missing, None);
    }

    #[test]
    fn test_query_length_macro() {
        let json = r#"{"users": [{"name": "Alice"}, {"name": "Bob"}, {"name": "Carol"}], "name": "test", "config": {"key1": "val1", "key2": "val2"}}"#;

        let array_length = query_length!(json, "users").unwrap();
        assert_eq!(array_length, Some(3));

        let string_length = query_length!(json, "name").unwrap();
        assert_eq!(string_length, Some(4));

        let object_length = query_length!(json, "config").unwrap();
        assert_eq!(object_length, Some(2));

        let missing_length = query_length!(json, "groups").unwrap();
        assert_eq!(missing_length, None);
    }
}
