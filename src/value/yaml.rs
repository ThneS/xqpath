use crate::value::format::FormatError;
use serde::Deserialize;
use serde_json::Value;

/// YAML 特定的便利函数和扩展
pub struct YamlSupport;

impl YamlSupport {
    /// 解析 YAML 字符串为 JSON Value
    pub fn parse(input: &str) -> Result<Value, FormatError> {
        // 先解析为 serde_yaml::Value
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(input)
            .map_err(|e| {
                FormatError::ParseError(format!("YAML parse error: {e}"))
            })?;

        // 转换为 JSON Value 以保持统一接口
        Self::yaml_to_json(yaml_value)
    }

    /// 将 JSON Value 转换为 YAML 字符串
    pub fn to_string(value: &Value) -> Result<String, FormatError> {
        serde_yaml::to_string(value).map_err(|e| {
            FormatError::SerializeError(format!("YAML serialize error: {e}"))
        })
    }

    /// 检查字符串是否为有效的 YAML
    pub fn is_valid_yaml(input: &str) -> bool {
        serde_yaml::from_str::<serde_yaml::Value>(input).is_ok()
    }

    /// 将 serde_yaml::Value 转换为 serde_json::Value
    fn yaml_to_json(
        yaml_value: serde_yaml::Value,
    ) -> Result<Value, FormatError> {
        match yaml_value {
            serde_yaml::Value::Null => Ok(Value::Null),
            serde_yaml::Value::Bool(b) => Ok(Value::Bool(b)),
            serde_yaml::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::Number(serde_json::Number::from(i)))
                } else if let Some(u) = n.as_u64() {
                    Ok(Value::Number(serde_json::Number::from(u)))
                } else if let Some(f) = n.as_f64() {
                    serde_json::Number::from_f64(f)
                        .map(Value::Number)
                        .ok_or_else(|| {
                            FormatError::ParseError(
                                "Invalid float number".to_string(),
                            )
                        })
                } else {
                    Err(FormatError::ParseError(
                        "Unsupported number format".to_string(),
                    ))
                }
            }
            serde_yaml::Value::String(s) => Ok(Value::String(s)),
            serde_yaml::Value::Sequence(seq) => {
                let mut json_array = Vec::new();
                for item in seq {
                    json_array.push(Self::yaml_to_json(item)?);
                }
                Ok(Value::Array(json_array))
            }
            serde_yaml::Value::Mapping(map) => {
                let mut json_object = serde_json::Map::new();
                for (key, value) in map {
                    let key_str = match key {
                        serde_yaml::Value::String(s) => s,
                        serde_yaml::Value::Number(n) => n.to_string(),
                        serde_yaml::Value::Bool(b) => b.to_string(),
                        _ => {
                            return Err(FormatError::ParseError(
                                "Invalid key type in YAML mapping".to_string(),
                            ))
                        }
                    };
                    json_object.insert(key_str, Self::yaml_to_json(value)?);
                }
                Ok(Value::Object(json_object))
            }
            serde_yaml::Value::Tagged(tagged) => {
                // 处理带标签的 YAML 值，这里简化处理，直接处理值部分
                Self::yaml_to_json(tagged.value)
            }
        }
    }

    /// 检测 YAML 文档分隔符
    pub fn has_document_separator(input: &str) -> bool {
        input.contains("---") || input.contains("...")
    }

    /// 解析多文档 YAML
    pub fn parse_multi_document(
        input: &str,
    ) -> Result<Vec<Value>, FormatError> {
        let mut documents = Vec::new();

        // 使用 serde_yaml 的多文档解析
        let deserializer = serde_yaml::Deserializer::from_str(input);
        for document in deserializer {
            let yaml_value =
                serde_yaml::Value::deserialize(document).map_err(|e| {
                    FormatError::ParseError(format!(
                        "YAML document parse error: {e}"
                    ))
                })?;
            documents.push(Self::yaml_to_json(yaml_value)?);
        }

        Ok(documents)
    }
}

/// YAML 特殊值处理
pub struct YamlSpecialValues;

impl YamlSpecialValues {
    /// 检查是否为 YAML 的特殊 null 值
    pub fn is_yaml_null(s: &str) -> bool {
        matches!(s.to_lowercase().as_str(), "null" | "~" | "nil" | "")
    }

    /// 检查是否为 YAML 的布尔值
    pub fn is_yaml_bool(s: &str) -> Option<bool> {
        match s.to_lowercase().as_str() {
            "true" | "yes" | "on" => Some(true),
            "false" | "no" | "off" => Some(false),
            _ => None,
        }
    }

    /// 尝试解析 YAML 数字
    pub fn parse_yaml_number(s: &str) -> Option<Value> {
        // 尝试整数
        if let Ok(i) = s.parse::<i64>() {
            return Some(Value::Number(serde_json::Number::from(i)));
        }

        // 尝试浮点数
        if let Ok(f) = s.parse::<f64>() {
            if let Some(n) = serde_json::Number::from_f64(f) {
                return Some(Value::Number(n));
            }
        }

        // 处理科学计数法
        if s.contains('e') || s.contains('E') {
            if let Ok(f) = s.parse::<f64>() {
                if let Some(n) = serde_json::Number::from_f64(f) {
                    return Some(Value::Number(n));
                }
            }
        }

        None
    }
}

/// YAML 格式化选项
pub struct YamlFormatter {
    pub indent: usize,
    pub width: usize,
}

impl Default for YamlFormatter {
    fn default() -> Self {
        Self {
            indent: 2,
            width: 80,
        }
    }
}

impl YamlFormatter {
    /// 创建新的格式化器
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置缩进
    pub fn with_indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }

    /// 设置行宽
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// 格式化 Value 为 YAML
    pub fn format(&self, value: &Value) -> Result<String, FormatError> {
        // 注意：serde_yaml 不直接支持自定义格式化选项
        // 这里提供基本的格式化功能
        YamlSupport::to_string(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_yaml_parse() {
        let input = r#"
name: Alice
age: 30
active: true
scores: [85, 92, 78]
address:
  street: 123 Main St
  city: Anytown
"#;

        let value = YamlSupport::parse(input).unwrap();

        assert_eq!(value["name"], "Alice");
        assert_eq!(value["age"], 30);
        assert_eq!(value["active"], true);
        assert_eq!(value["scores"][0], 85);
        assert_eq!(value["address"]["street"], "123 Main St");
    }

    #[test]
    fn test_yaml_serialize() {
        let value = json!({
            "name": "Alice",
            "age": 30,
            "scores": [85, 92, 78]
        });

        let yaml_str = YamlSupport::to_string(&value).unwrap();

        assert!(yaml_str.contains("name: Alice"));
        assert!(yaml_str.contains("age: 30"));
        assert!(yaml_str.contains("- 85"));
    }

    #[test]
    fn test_yaml_multi_document() {
        let input = r#"
---
name: Alice
age: 30
---
name: Bob
age: 25
"#;

        let documents = YamlSupport::parse_multi_document(input).unwrap();
        assert_eq!(documents.len(), 2);
        assert_eq!(documents[0]["name"], "Alice");
        assert_eq!(documents[1]["name"], "Bob");
    }

    #[test]
    fn test_yaml_special_values() {
        assert!(YamlSpecialValues::is_yaml_null("null"));
        assert!(YamlSpecialValues::is_yaml_null("~"));
        assert!(YamlSpecialValues::is_yaml_null(""));

        assert_eq!(YamlSpecialValues::is_yaml_bool("true"), Some(true));
        assert_eq!(YamlSpecialValues::is_yaml_bool("yes"), Some(true));
        assert_eq!(YamlSpecialValues::is_yaml_bool("false"), Some(false));
        assert_eq!(YamlSpecialValues::is_yaml_bool("no"), Some(false));
    }

    #[test]
    fn test_yaml_number_parsing() {
        assert_eq!(
            YamlSpecialValues::parse_yaml_number("42").unwrap(),
            json!(42)
        );
        assert_eq!(
            YamlSpecialValues::parse_yaml_number("3.15").unwrap(),
            json!(3.15)
        );
        assert_eq!(
            YamlSpecialValues::parse_yaml_number("1.23e4").unwrap(),
            json!(12300.0)
        );
    }

    #[test]
    fn test_yaml_formatter() {
        let value = json!({"name": "Alice", "age": 30});
        let formatter = YamlFormatter::new().with_indent(4);

        let formatted = formatter.format(&value).unwrap();
        assert!(formatted.contains("name: Alice"));
    }
}
