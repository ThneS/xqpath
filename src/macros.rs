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
