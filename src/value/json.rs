use crate::value::format::FormatError;
use serde_json::Value;

/// JSON 特定的便利函数和扩展
pub struct JsonSupport;

impl JsonSupport {
    /// 解析 JSON 字符串
    pub fn parse(input: &str) -> Result<Value, FormatError> {
        serde_json::from_str(input).map_err(|e| {
            FormatError::ParseError(format!("JSON parse error: {e}"))
        })
    }

    /// 将 Value 转换为格式化的 JSON 字符串
    pub fn to_pretty_string(value: &Value) -> Result<String, FormatError> {
        serde_json::to_string_pretty(value).map_err(|e| {
            FormatError::SerializeError(format!("JSON serialize error: {e}"))
        })
    }

    /// 将 Value 转换为压缩的 JSON 字符串
    pub fn to_compact_string(value: &Value) -> Result<String, FormatError> {
        serde_json::to_string(value).map_err(|e| {
            FormatError::SerializeError(format!("JSON serialize error: {e}"))
        })
    }

    /// 检查字符串是否为有效的 JSON
    pub fn is_valid_json(input: &str) -> bool {
        serde_json::from_str::<Value>(input).is_ok()
    }

    /// 获取值的 JSON 类型名称
    pub fn get_type_name(value: &Value) -> &'static str {
        match value {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }

    /// 深度克隆 JSON 值
    pub fn deep_clone(value: &Value) -> Value {
        value.clone()
    }
}

/// JSON 路径操作辅助函数
pub struct JsonPath;

impl JsonPath {
    /// 安全地从对象获取字段
    pub fn get_field<'a>(value: &'a Value, field: &str) -> Option<&'a Value> {
        value.as_object()?.get(field)
    }

    /// 安全地从数组获取索引
    pub fn get_index(value: &Value, index: usize) -> Option<&Value> {
        value.as_array()?.get(index)
    }

    /// 获取数组长度
    pub fn array_len(value: &Value) -> Option<usize> {
        value.as_array().map(|arr| arr.len())
    }

    /// 获取对象的所有键
    pub fn object_keys(value: &Value) -> Option<Vec<&str>> {
        value
            .as_object()
            .map(|obj| obj.keys().map(|s| s.as_str()).collect())
    }

    /// 检查值是否为指定类型
    pub fn is_type(value: &Value, type_name: &str) -> bool {
        match type_name.to_lowercase().as_str() {
            "null" => value.is_null(),
            "bool" | "boolean" => value.is_boolean(),
            "number" | "num" => value.is_number(),
            "string" | "str" => value.is_string(),
            "array" | "list" => value.is_array(),
            "object" | "map" => value.is_object(),
            _ => false,
        }
    }
}

/// JSON 修改操作（用于更新功能）
#[cfg(feature = "update")]
pub struct JsonModifier;

#[cfg(feature = "update")]
impl JsonModifier {
    /// 在对象中设置字段
    pub fn set_field(
        value: &mut Value,
        field: &str,
        new_value: Value,
    ) -> Result<(), FormatError> {
        match value {
            Value::Object(map) => {
                map.insert(field.to_string(), new_value);
                Ok(())
            }
            _ => Err(FormatError::SerializeError(
                "Cannot set field on non-object value".to_string(),
            )),
        }
    }

    /// 在数组中设置索引
    pub fn set_index(
        value: &mut Value,
        index: usize,
        new_value: Value,
    ) -> Result<(), FormatError> {
        match value {
            Value::Array(arr) => {
                if index < arr.len() {
                    arr[index] = new_value;
                    Ok(())
                } else if index == arr.len() {
                    arr.push(new_value);
                    Ok(())
                } else {
                    Err(FormatError::SerializeError(format!(
                        "Array index {index} out of bounds"
                    )))
                }
            }
            _ => Err(FormatError::SerializeError(
                "Cannot set index on non-array value".to_string(),
            )),
        }
    }

    /// 确保路径存在，创建缺失的中间对象/数组
    pub fn ensure_path<'a>(
        root: &'a mut Value,
        path: &[crate::parser::path::PathSegment],
    ) -> Result<&'a mut Value, FormatError> {
        let mut current = root;

        for segment in path {
            match segment {
                crate::parser::path::PathSegment::Field(field) => {
                    // 确保当前值是对象
                    if !current.is_object() {
                        *current = serde_json::json!({});
                    }

                    // 获取或创建字段
                    let obj = current.as_object_mut().unwrap();
                    if !obj.contains_key(field) {
                        obj.insert(field.clone(), Value::Null);
                    }
                    current = obj.get_mut(field).unwrap();
                }
                crate::parser::path::PathSegment::Index(index) => {
                    // 确保当前值是数组
                    if !current.is_array() {
                        *current = serde_json::json!([]);
                    }

                    // 扩展数组到所需长度
                    let arr = current.as_array_mut().unwrap();
                    while arr.len() <= *index {
                        arr.push(Value::Null);
                    }
                    current = &mut arr[*index];
                }
                _ => {
                    return Err(FormatError::SerializeError(
                        "Cannot ensure path with wildcards or type filters"
                            .to_string(),
                    ));
                }
            }
        }

        Ok(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_support_parse() {
        let input = r#"{"name": "Alice", "age": 30}"#;
        let value = JsonSupport::parse(input).unwrap();

        assert_eq!(value["name"], "Alice");
        assert_eq!(value["age"], 30);
    }

    #[test]
    fn test_json_support_serialize() {
        let value = json!({"name": "Alice", "age": 30});

        let pretty = JsonSupport::to_pretty_string(&value).unwrap();
        assert!(pretty.contains("Alice"));
        assert!(pretty.contains("30"));

        let compact = JsonSupport::to_compact_string(&value).unwrap();
        assert!(compact.contains("Alice"));
        assert!(!compact.contains("  ")); // 没有格式化空格
    }

    #[test]
    fn test_json_path_operations() {
        let value = json!({
            "users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ]
        });

        // 测试字段访问
        let users = JsonPath::get_field(&value, "users").unwrap();
        assert!(users.is_array());

        // 测试数组访问
        let first_user = JsonPath::get_index(users, 0).unwrap();
        assert_eq!(first_user["name"], "Alice");

        // 测试类型检查
        assert!(JsonPath::is_type(&value["users"], "array"));
        assert!(JsonPath::is_type(&value["users"][0]["name"], "string"));
    }

    #[test]
    fn test_get_type_name() {
        assert_eq!(JsonSupport::get_type_name(&json!(null)), "null");
        assert_eq!(JsonSupport::get_type_name(&json!(true)), "boolean");
        assert_eq!(JsonSupport::get_type_name(&json!(42)), "number");
        assert_eq!(JsonSupport::get_type_name(&json!("hello")), "string");
        assert_eq!(JsonSupport::get_type_name(&json!([])), "array");
        assert_eq!(JsonSupport::get_type_name(&json!({})), "object");
    }

    #[cfg(feature = "update")]
    #[test]
    fn test_json_modifier() {
        let mut value = json!({"users": []});

        // 设置字段
        JsonModifier::set_field(&mut value, "version", json!("1.0")).unwrap();
        assert_eq!(value["version"], "1.0");

        // 设置数组元素
        let mut arr = json!([1, 2, 3]);
        JsonModifier::set_index(&mut arr, 1, json!(42)).unwrap();
        assert_eq!(arr[1], 42);
    }
}
