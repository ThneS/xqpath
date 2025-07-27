/// 便利宏，用于从结构化数据中提取字段
///
/// # 参数
/// - `$data`: 输入的数据字符串（JSON 或 YAML 格式）
/// - `$path`: 路径表达式字符串（如 ".user.name" 或 ".users[0].email"）
///
/// # 返回值
/// 返回 `Result<Vec<serde_json::Value>, Box<dyn std::error::Error>>`
///
/// # 示例
/// ```rust
/// use datapath::datapath_get;
/// use serde_json::json;
///
/// let yaml = r#"
/// user:
///   name: Alice
///   age: 30
/// "#;
///
/// let result = datapath_get!(yaml, "user.name").unwrap();
/// assert_eq!(result[0], json!("Alice"));
/// ```
#[macro_export]
macro_rules! datapath_get {
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
/// - `$path`: 路径表达式字符串（如 ".user.name" 或 ".users[0].email"）
/// - `$value`: 要设置的新值（必须是 `serde_json::Value` 类型）
///
/// # 返回值
/// 返回 `Result<String, Box<dyn std::error::Error>>`，包含更新后的数据字符串
///
/// # 示例
/// ```rust
/// #[cfg(feature = "update")]
/// use datapath::datapath_set;
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
///     let updated = datapath_set!(yaml, "user.age", json!(31)).unwrap();
///     // updated 现在包含更新后的 YAML 字符串
/// }
/// ```
#[cfg(feature = "update")]
#[macro_export]
macro_rules! datapath_set {
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
/// use datapath::datapath_exists;
///
/// let json = r#"{"user": {"name": "Alice"}}"#;
/// let exists = datapath_exists!(json, "user.name").unwrap();
/// assert_eq!(exists, true);
///
/// let missing = datapath_exists!(json, "user.email").unwrap();
/// assert_eq!(missing, false);
/// ```
#[macro_export]
macro_rules! datapath_exists {
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
/// use datapath::datapath_type;
///
/// let json = r#"{"name": "Alice", "age": 30, "active": true}"#;
/// let types = datapath_type!(json, "*").unwrap();
/// // types 可能包含 ["string", "number", "boolean"]
/// ```
#[macro_export]
macro_rules! datapath_type {
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
/// use datapath::datapath_count;
///
/// let json = r#"{"users": [{"name": "Alice"}, {"name": "Bob"}]}"#;
/// let count = datapath_count!(json, "users[*]").unwrap();
/// assert_eq!(count, 2);
/// ```
#[macro_export]
macro_rules! datapath_count {
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
/// use datapath::datapath_extract;
///
/// let yaml = r#"
/// user:
///   name: Alice
///   age: 30
/// "#;
///
/// let json_output = datapath_extract!(yaml, "user", "json").unwrap();
/// // json_output 包含 JSON 格式的用户数据
/// ```
#[macro_export]
macro_rules! datapath_extract {
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn test_datapath_get_macro() {
        let yaml = r#"
user:
  name: Alice
  age: 30
"#;

        let result = datapath_get!(yaml, "user.name").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], json!("Alice"));
    }

    #[test]
    fn test_datapath_exists_macro() {
        let json = r#"{"user": {"name": "Alice"}}"#;

        let exists = datapath_exists!(json, "user.name").unwrap();
        assert!(exists);

        let missing = datapath_exists!(json, "user.email").unwrap();
        assert!(!missing);
    }

    #[test]
    fn test_datapath_type_macro() {
        let json = r#"{"name": "Alice", "age": 30, "active": true}"#;

        let types = datapath_type!(json, "name").unwrap();
        assert_eq!(types, vec!["string"]);

        let age_types = datapath_type!(json, "age").unwrap();
        assert_eq!(age_types, vec!["number"]);
    }

    #[test]
    fn test_datapath_count_macro() {
        let json = r#"{"users": [{"name": "Alice"}, {"name": "Bob"}]}"#;

        let count = datapath_count!(json, "users[*]").unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_datapath_extract_macro() {
        let yaml = r#"
user:
  name: Alice
  age: 30
"#;

        match datapath_extract!(yaml, "user", "json") {
            Ok(json_output) => {
                assert!(json_output.contains("Alice"));
                assert!(json_output.contains("30"));
            }
            Err(e) => panic!("Test failed: {e}"),
        }
    }

    #[cfg(feature = "update")]
    #[test]
    fn test_datapath_set_macro() {
        let yaml = r#"
user:
  name: Alice
  age: 30
"#;

        let updated = datapath_set!(yaml, "user.age", json!(31)).unwrap();
        assert!(updated.contains("31"));
    }
}
