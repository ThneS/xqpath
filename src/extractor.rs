use crate::parser::path::PathSegment;
use crate::value::json::JsonPath;
use serde_json::Value;

/// 提取错误类型
#[derive(Debug, Clone)]
pub enum ExtractError {
    PathNotFound(String),
    IndexOutOfBounds(usize, usize),
    TypeMismatch(String, String),
    InvalidPath(String),
}

impl std::fmt::Display for ExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractError::PathNotFound(path) => {
                write!(f, "Path not found: {path}")
            }
            ExtractError::IndexOutOfBounds(index, len) => {
                write!(
                    f,
                    "Index {index} out of bounds for array of length {len}"
                )
            }
            ExtractError::TypeMismatch(expected, actual) => {
                write!(f, "Type mismatch: expected {expected}, got {actual}")
            }
            ExtractError::InvalidPath(msg) => {
                write!(f, "Invalid path: {msg}")
            }
        }
    }
}

impl std::error::Error for ExtractError {}

/// 字段提取器，核心提取逻辑
pub struct Extractor;

impl Extractor {
    /// 从根值按照路径提取字段
    pub fn extract<'a>(
        root: &'a Value,
        path: &[PathSegment],
    ) -> Result<Vec<&'a Value>, ExtractError> {
        if path.is_empty() {
            return Ok(vec![root]);
        }

        let mut current_values = vec![root];

        for segment in path {
            current_values = Self::apply_segment(current_values, segment)?;
        }

        Ok(current_values)
    }

    /// 应用单个路径段到当前值集合
    fn apply_segment<'a>(
        values: Vec<&'a Value>,
        segment: &PathSegment,
    ) -> Result<Vec<&'a Value>, ExtractError> {
        let mut results = Vec::new();

        for value in values {
            match segment {
                PathSegment::Field(field_name) => {
                    results.extend(Self::extract_field(value, field_name)?);
                }
                PathSegment::Index(index) => {
                    results.extend(Self::extract_index(value, *index)?);
                }
                PathSegment::Wildcard => {
                    results.extend(Self::extract_wildcard(value)?);
                }
                PathSegment::RecursiveWildcard => {
                    results.extend(Self::extract_recursive(value)?);
                }
                PathSegment::TypeFilter(type_name) => {
                    results.extend(Self::apply_type_filter(
                        vec![value],
                        type_name,
                    )?);
                }
            }
        }

        Ok(results)
    }

    /// 提取对象字段
    fn extract_field<'a>(
        value: &'a Value,
        field_name: &str,
    ) -> Result<Vec<&'a Value>, ExtractError> {
        match value {
            Value::Object(map) => {
                if let Some(field_value) = map.get(field_name) {
                    Ok(vec![field_value])
                } else {
                    Ok(vec![]) // 字段不存在时返回空结果而不是错误
                }
            }
            _ => Err(ExtractError::TypeMismatch(
                "object".to_string(),
                Self::get_value_type_name(value).to_string(),
            )),
        }
    }

    /// 提取数组索引
    fn extract_index(
        value: &Value,
        index: usize,
    ) -> Result<Vec<&Value>, ExtractError> {
        match value {
            Value::Array(arr) => {
                if index < arr.len() {
                    Ok(vec![&arr[index]])
                } else {
                    Err(ExtractError::IndexOutOfBounds(index, arr.len()))
                }
            }
            _ => Err(ExtractError::TypeMismatch(
                "array".to_string(),
                Self::get_value_type_name(value).to_string(),
            )),
        }
    }

    /// 提取通配符匹配的所有值
    fn extract_wildcard(value: &Value) -> Result<Vec<&Value>, ExtractError> {
        match value {
            Value::Object(map) => Ok(map.values().collect()),
            Value::Array(arr) => Ok(arr.iter().collect()),
            _ => Ok(vec![]), // 对于其他类型，通配符不匹配任何内容
        }
    }

    /// 递归提取所有匹配的值
    fn extract_recursive(value: &Value) -> Result<Vec<&Value>, ExtractError> {
        let mut results = vec![value]; // 包含当前值本身

        match value {
            Value::Object(map) => {
                for field_value in map.values() {
                    results.extend(Self::extract_recursive(field_value)?);
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    results.extend(Self::extract_recursive(item)?);
                }
            }
            _ => {} // 叶子节点，不需要递归
        }

        Ok(results)
    }

    /// 应用类型过滤器
    fn apply_type_filter<'a>(
        values: Vec<&'a Value>,
        type_name: &str,
    ) -> Result<Vec<&'a Value>, ExtractError> {
        let filtered: Vec<&'a Value> = values
            .into_iter()
            .filter(|v| Self::matches_type(v, type_name))
            .collect();

        Ok(filtered)
    }

    /// 检查值是否匹配指定类型
    fn matches_type(value: &Value, type_name: &str) -> bool {
        JsonPath::is_type(value, type_name)
    }

    /// 获取值的类型名称
    fn get_value_type_name(value: &Value) -> &'static str {
        match value {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }
}

/// 便利函数，直接从根值提取路径
pub fn extract<'a>(
    root: &'a Value,
    path: &[PathSegment],
) -> Result<Vec<&'a Value>, ExtractError> {
    Extractor::extract(root, path)
}

/// 提取器配置选项
#[derive(Debug, Clone)]
pub struct ExtractorConfig {
    /// 是否在路径不存在时返回空结果而不是错误
    pub ignore_missing_paths: bool,
    /// 是否在类型不匹配时返回空结果而不是错误
    pub ignore_type_mismatches: bool,
    /// 最大递归深度（防止无限递归）
    pub max_recursion_depth: usize,
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        Self {
            ignore_missing_paths: true,
            ignore_type_mismatches: false,
            max_recursion_depth: 1000,
        }
    }
}

/// 可配置的提取器
pub struct ConfigurableExtractor {
    config: ExtractorConfig,
}

impl ConfigurableExtractor {
    /// 创建新的可配置提取器
    pub fn new(config: ExtractorConfig) -> Self {
        Self { config }
    }
}

impl Default for ConfigurableExtractor {
    fn default() -> Self {
        Self::new(ExtractorConfig::default())
    }
}

impl ConfigurableExtractor {
    /// 提取字段（带配置）
    pub fn extract<'a>(
        &self,
        root: &'a Value,
        path: &[PathSegment],
    ) -> Result<Vec<&'a Value>, ExtractError> {
        self.extract_with_depth(root, path, 0)
    }

    /// 带递归深度控制的提取
    fn extract_with_depth<'a>(
        &self,
        root: &'a Value,
        path: &[PathSegment],
        depth: usize,
    ) -> Result<Vec<&'a Value>, ExtractError> {
        if depth > self.config.max_recursion_depth {
            return Err(ExtractError::InvalidPath(
                "Maximum recursion depth exceeded".to_string(),
            ));
        }

        if path.is_empty() {
            return Ok(vec![root]);
        }

        let mut current_values = vec![root];

        for segment in path {
            current_values = self.apply_segment_with_config(
                current_values,
                segment,
                depth + 1,
            )?;
        }

        Ok(current_values)
    }

    /// 应用路径段（带配置）
    fn apply_segment_with_config<'a>(
        &self,
        values: Vec<&'a Value>,
        segment: &PathSegment,
        depth: usize,
    ) -> Result<Vec<&'a Value>, ExtractError> {
        let mut results = Vec::new();

        for value in values {
            let segment_result = match segment {
                PathSegment::Field(field_name) => {
                    self.extract_field_with_config(value, field_name)
                }
                PathSegment::Index(index) => {
                    self.extract_index_with_config(value, *index)
                }
                PathSegment::Wildcard => Extractor::extract_wildcard(value),
                PathSegment::RecursiveWildcard => {
                    if depth > self.config.max_recursion_depth {
                        return Err(ExtractError::InvalidPath(
                            "Maximum recursion depth exceeded in recursive wildcard".to_string(),
                        ));
                    }
                    Extractor::extract_recursive(value)
                }
                PathSegment::TypeFilter(type_name) => {
                    Extractor::apply_type_filter(vec![value], type_name)
                }
            };

            match segment_result {
                Ok(mut segment_values) => results.append(&mut segment_values),
                Err(e) => {
                    if !self.should_ignore_error(&e) {
                        return Err(e);
                    }
                    // 忽略错误，继续处理
                }
            }
        }

        Ok(results)
    }

    /// 带配置的字段提取
    fn extract_field_with_config<'a>(
        &self,
        value: &'a Value,
        field_name: &str,
    ) -> Result<Vec<&'a Value>, ExtractError> {
        match value {
            Value::Object(map) => {
                if let Some(field_value) = map.get(field_name) {
                    Ok(vec![field_value])
                } else if self.config.ignore_missing_paths {
                    Ok(vec![])
                } else {
                    Err(ExtractError::PathNotFound(field_name.to_string()))
                }
            }
            _ => {
                if self.config.ignore_type_mismatches {
                    Ok(vec![])
                } else {
                    Err(ExtractError::TypeMismatch(
                        "object".to_string(),
                        Extractor::get_value_type_name(value).to_string(),
                    ))
                }
            }
        }
    }

    /// 带配置的索引提取
    fn extract_index_with_config<'a>(
        &self,
        value: &'a Value,
        index: usize,
    ) -> Result<Vec<&'a Value>, ExtractError> {
        match value {
            Value::Array(arr) => {
                if index < arr.len() {
                    Ok(vec![&arr[index]])
                } else if self.config.ignore_missing_paths {
                    Ok(vec![])
                } else {
                    Err(ExtractError::IndexOutOfBounds(index, arr.len()))
                }
            }
            _ => {
                if self.config.ignore_type_mismatches {
                    Ok(vec![])
                } else {
                    Err(ExtractError::TypeMismatch(
                        "array".to_string(),
                        Extractor::get_value_type_name(value).to_string(),
                    ))
                }
            }
        }
    }

    /// 判断是否应该忽略错误
    fn should_ignore_error(&self, error: &ExtractError) -> bool {
        match error {
            ExtractError::PathNotFound(_) => self.config.ignore_missing_paths,
            ExtractError::TypeMismatch(_, _) => {
                self.config.ignore_type_mismatches
            }
            ExtractError::IndexOutOfBounds(_, _) => {
                self.config.ignore_missing_paths
            }
            ExtractError::InvalidPath(_) => false, // 不忽略路径无效错误
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::path::parse_path;
    use serde_json::json;

    #[test]
    fn test_extract_field() {
        let data = json!({"name": "Alice", "age": 30});
        let path = parse_path(".name").unwrap();

        let result = extract(&data, &path).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], &json!("Alice"));
    }

    #[test]
    fn test_extract_index() {
        let data = json!(["Alice", "Bob", "Charlie"]);
        let path = parse_path("[1]").unwrap();

        let result = extract(&data, &path).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], &json!("Bob"));
    }

    #[test]
    fn test_extract_wildcard() {
        let data = json!({"a": 1, "b": 2, "c": 3});
        let path = parse_path("*").unwrap();

        let result = extract(&data, &path).unwrap();
        assert_eq!(result.len(), 3);
        // 注意：结果顺序可能不确定
    }

    #[test]
    fn test_extract_complex_path() {
        let data = json!({
            "users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ]
        });

        let path = parse_path(".users[0].name").unwrap();
        let result = extract(&data, &path).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], &json!("Alice"));
    }

    #[test]
    fn test_extract_with_wildcard() {
        let data = json!({
            "users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ]
        });

        let path = parse_path(".users[*].name").unwrap();
        let result = extract(&data, &path).unwrap();
        assert_eq!(result.len(), 2);
        // 应该包含 "Alice" 和 "Bob"
    }

    #[test]
    fn test_configurable_extractor() {
        let config = ExtractorConfig {
            ignore_missing_paths: true,
            ignore_type_mismatches: true,
            max_recursion_depth: 100,
        };
        let extractor = ConfigurableExtractor::new(config);

        let data = json!({"name": "Alice"});
        let path = parse_path(".missing_field").unwrap();

        let result = extractor.extract(&data, &path).unwrap();
        assert_eq!(result.len(), 0); // 应该返回空结果而不是错误
    }

    #[test]
    fn test_type_filter() {
        let data = json!([1, "hello", true, std::f64::consts::PI]);
        let path = vec![
            PathSegment::Wildcard,
            PathSegment::TypeFilter("string".to_string()),
        ];

        let result = extract(&data, &path).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], &json!("hello"));
    }
}
