use serde_json::Value;
use std::fmt;

/// 格式处理错误
#[derive(Debug, Clone)]
pub enum FormatError {
    ParseError(String),
    SerializeError(String),
    UnsupportedFormat(String),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            FormatError::SerializeError(msg) => {
                write!(f, "Serialize error: {msg}")
            }
            FormatError::UnsupportedFormat(format) => {
                write!(f, "Unsupported format: {format}")
            }
        }
    }
}

impl std::error::Error for FormatError {}

/// 数据格式处理统一接口
pub trait ValueFormat: Send + Sync {
    /// 解析输入字符串为 Value
    fn parse(&self, input: &str) -> Result<Value, FormatError>;

    /// 将 Value 序列化为字符串
    fn to_string(&self, value: &Value) -> Result<String, FormatError>;

    /// 获取格式名称
    fn name(&self) -> &'static str;
}

/// JSON 格式处理器
pub struct JsonFormat;

impl ValueFormat for JsonFormat {
    fn parse(&self, input: &str) -> Result<Value, FormatError> {
        serde_json::from_str(input).map_err(|e| {
            FormatError::ParseError(format!("JSON parse error: {e}"))
        })
    }

    fn to_string(&self, value: &Value) -> Result<String, FormatError> {
        serde_json::to_string_pretty(value).map_err(|e| {
            FormatError::SerializeError(format!("JSON serialize error: {e}"))
        })
    }

    fn name(&self) -> &'static str {
        "json"
    }
}

/// YAML 格式处理器
pub struct YamlFormat;

impl ValueFormat for YamlFormat {
    fn parse(&self, input: &str) -> Result<Value, FormatError> {
        // 先解析为 serde_yaml::Value，然后转换为 serde_json::Value
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(input)
            .map_err(|e| {
                FormatError::ParseError(format!("YAML parse error: {e}"))
            })?;

        // 转换为 JSON Value 以保持统一
        let json_str = serde_json::to_string(&yaml_value).map_err(|e| {
            FormatError::SerializeError(format!(
                "YAML to JSON conversion error: {e}"
            ))
        })?;

        serde_json::from_str(&json_str).map_err(|e| {
            FormatError::ParseError(format!(
                "JSON parse error during YAML conversion: {e}"
            ))
        })
    }

    fn to_string(&self, value: &Value) -> Result<String, FormatError> {
        serde_yaml::to_string(value).map_err(|e| {
            FormatError::SerializeError(format!("YAML serialize error: {e}"))
        })
    }

    fn name(&self) -> &'static str {
        "yaml"
    }
}

/// 自动检测输入格式并返回相应的格式处理器
pub fn detect_format(input: &str) -> Result<Box<dyn ValueFormat>, FormatError> {
    let trimmed = input.trim_start();

    if trimmed.is_empty() {
        return Err(FormatError::UnsupportedFormat("empty input".to_string()));
    }

    // 检测 JSON 格式
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        Ok(Box::new(JsonFormat))
    } else {
        // 默认尝试 YAML（更宽松），适用于所有其他情况
        Ok(Box::new(YamlFormat))
    }
}

/// 格式注册表，支持运行时格式扩展
pub struct FormatRegistry {
    formats: std::collections::HashMap<String, Box<dyn ValueFormat>>,
}

impl FormatRegistry {
    /// 创建新的格式注册表
    pub fn new() -> Self {
        let mut registry = Self {
            formats: std::collections::HashMap::new(),
        };

        // 注册内置格式
        registry.register("json".to_string(), Box::new(JsonFormat));
        registry.register("yaml".to_string(), Box::new(YamlFormat));
        registry.register("yml".to_string(), Box::new(YamlFormat));

        registry
    }

    /// 注册新格式
    pub fn register(&mut self, name: String, format: Box<dyn ValueFormat>) {
        self.formats.insert(name, format);
    }

    /// 获取格式处理器
    pub fn get(&self, name: &str) -> Option<&dyn ValueFormat> {
        self.formats.get(name).map(|f| f.as_ref())
    }

    /// 列出所有支持的格式
    pub fn list_formats(&self) -> Vec<&str> {
        self.formats.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for FormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_format() {
        let format = JsonFormat;
        let input = r#"{"name": "Alice", "age": 30}"#;

        let value = format.parse(input).unwrap();
        assert_eq!(value["name"], "Alice");
        assert_eq!(value["age"], 30);

        let output = format.to_string(&value).unwrap();
        assert!(output.contains("Alice"));
    }

    #[test]
    fn test_yaml_format() {
        let format = YamlFormat;
        let input = r#"
name: Alice
age: 30
"#;

        let value = format.parse(input).unwrap();
        assert_eq!(value["name"], "Alice");
        assert_eq!(value["age"], 30);

        let output = format.to_string(&value).unwrap();
        assert!(output.contains("Alice"));
    }

    #[test]
    fn test_detect_json_format() {
        let input = r#"{"name": "Alice"}"#;
        let format = detect_format(input).unwrap();
        assert_eq!(format.name(), "json");
    }

    #[test]
    fn test_detect_yaml_format() {
        let input = r#"name: Alice"#;
        let format = detect_format(input).unwrap();
        assert_eq!(format.name(), "yaml");
    }

    #[test]
    fn test_format_registry() {
        let registry = FormatRegistry::new();

        let json_format = registry.get("json").unwrap();
        assert_eq!(json_format.name(), "json");

        let yaml_format = registry.get("yaml").unwrap();
        assert_eq!(yaml_format.name(), "yaml");

        let formats = registry.list_formats();
        assert!(formats.contains(&"json"));
        assert!(formats.contains(&"yaml"));
    }
}
