#[cfg(feature = "update")]
use crate::parser::path::PathSegment;
#[cfg(feature = "update")]
use serde_json::Value;

#[cfg(feature = "update")]
/// 更新错误类型
#[derive(Debug, Clone)]
pub enum UpdateError {
    PathNotFound(String),
    IndexOutOfBounds(usize, usize),
    TypeMismatch(String, String),
    InvalidPath(String),
    InvalidOperation(String),
    CannotCreatePath(String),
}

#[cfg(feature = "update")]
impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateError::PathNotFound(path) => {
                write!(f, "Path not found: {path}")
            }
            UpdateError::IndexOutOfBounds(index, len) => {
                write!(
                    f,
                    "Index {index} out of bounds for array of length {len}"
                )
            }
            UpdateError::TypeMismatch(expected, actual) => {
                write!(f, "Type mismatch: expected {expected}, got {actual}")
            }
            UpdateError::InvalidPath(msg) => write!(f, "Invalid path: {msg}"),
            UpdateError::InvalidOperation(msg) => {
                write!(f, "Invalid operation: {msg}")
            }
            UpdateError::CannotCreatePath(msg) => {
                write!(f, "Cannot create path: {msg}")
            }
        }
    }
}

#[cfg(feature = "update")]
impl std::error::Error for UpdateError {}

#[cfg(feature = "update")]
/// 字段更新器
pub struct Updater;

#[cfg(feature = "update")]
impl Updater {
    /// 在指定路径更新值
    pub fn update(
        root: &mut Value,
        path: &[PathSegment],
        new_value: Value,
    ) -> Result<(), UpdateError> {
        if path.is_empty() {
            *root = new_value;
            return Ok(());
        }

        Self::update_recursive(root, path, new_value, 0)
    }

    /// 递归更新实现
    fn update_recursive(
        current: &mut Value,
        remaining_path: &[PathSegment],
        new_value: Value,
        depth: usize,
    ) -> Result<(), UpdateError> {
        if depth > 1000 {
            return Err(UpdateError::InvalidPath(
                "Maximum recursion depth exceeded".to_string(),
            ));
        }

        if remaining_path.is_empty() {
            *current = new_value;
            return Ok(());
        }

        let (current_segment, rest_path) =
            remaining_path.split_first().unwrap();

        match current_segment {
            PathSegment::Field(field_name) => Self::update_field(
                current,
                field_name,
                rest_path,
                new_value,
                depth + 1,
            ),
            PathSegment::Index(index) => Self::update_index(
                current,
                *index,
                rest_path,
                new_value,
                depth + 1,
            ),
            PathSegment::Wildcard => Self::update_wildcard(
                current,
                rest_path,
                new_value.clone(),
                depth + 1,
            ),
            PathSegment::RecursiveWildcard => {
                Err(UpdateError::InvalidOperation(
                    "Cannot update with recursive wildcard".to_string(),
                ))
            }
            PathSegment::TypeFilter(_) => Err(UpdateError::InvalidOperation(
                "Cannot update with type filter".to_string(),
            )),
        }
    }

    /// 更新对象字段
    fn update_field(
        current: &mut Value,
        field_name: &str,
        remaining_path: &[PathSegment],
        new_value: Value,
        depth: usize,
    ) -> Result<(), UpdateError> {
        // 确保当前值是对象
        if !current.is_object() {
            if current.is_null() {
                *current = serde_json::json!({});
            } else {
                return Err(UpdateError::TypeMismatch(
                    "object".to_string(),
                    Self::get_value_type_name(current).to_string(),
                ));
            }
        }

        let obj = current.as_object_mut().unwrap();

        if remaining_path.is_empty() {
            // 直接设置字段值
            obj.insert(field_name.to_string(), new_value);
            Ok(())
        } else {
            // 需要继续更新嵌套路径
            if !obj.contains_key(field_name) {
                // 创建缺失的中间节点
                let intermediate_value =
                    Self::create_intermediate_value(&remaining_path[0]);
                obj.insert(field_name.to_string(), intermediate_value);
            }

            let field_value = obj.get_mut(field_name).unwrap();
            Self::update_recursive(
                field_value,
                remaining_path,
                new_value,
                depth,
            )
        }
    }

    /// 更新数组索引
    fn update_index(
        current: &mut Value,
        index: usize,
        remaining_path: &[PathSegment],
        new_value: Value,
        depth: usize,
    ) -> Result<(), UpdateError> {
        // 确保当前值是数组
        if !current.is_array() {
            if current.is_null() {
                *current = serde_json::json!([]);
            } else {
                return Err(UpdateError::TypeMismatch(
                    "array".to_string(),
                    Self::get_value_type_name(current).to_string(),
                ));
            }
        }

        let arr = current.as_array_mut().unwrap();

        // 扩展数组到所需长度
        while arr.len() <= index {
            arr.push(Value::Null);
        }

        if remaining_path.is_empty() {
            // 直接设置数组元素
            arr[index] = new_value;
            Ok(())
        } else {
            // 需要继续更新嵌套路径
            if arr[index].is_null() {
                // 创建缺失的中间节点
                arr[index] =
                    Self::create_intermediate_value(&remaining_path[0]);
            }

            Self::update_recursive(
                &mut arr[index],
                remaining_path,
                new_value,
                depth,
            )
        }
    }

    /// 通配符批量更新
    fn update_wildcard(
        current: &mut Value,
        remaining_path: &[PathSegment],
        new_value: Value,
        depth: usize,
    ) -> Result<(), UpdateError> {
        match current {
            Value::Object(map) => {
                for (_, field_value) in map.iter_mut() {
                    Self::update_recursive(
                        field_value,
                        remaining_path,
                        new_value.clone(),
                        depth,
                    )?;
                }
                Ok(())
            }
            Value::Array(arr) => {
                for item in arr.iter_mut() {
                    Self::update_recursive(
                        item,
                        remaining_path,
                        new_value.clone(),
                        depth,
                    )?;
                }
                Ok(())
            }
            _ => Err(UpdateError::TypeMismatch(
                "object or array".to_string(),
                Self::get_value_type_name(current).to_string(),
            )),
        }
    }

    /// 根据路径段类型创建适当的中间值
    fn create_intermediate_value(next_segment: &PathSegment) -> Value {
        match next_segment {
            PathSegment::Field(_) => serde_json::json!({}),
            PathSegment::Index(_) => serde_json::json!([]),
            _ => Value::Null,
        }
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

#[cfg(feature = "update")]
/// 便利函数，直接更新路径
pub fn update(
    root: &mut Value,
    path: &[PathSegment],
    new_value: Value,
) -> Result<(), UpdateError> {
    Updater::update(root, path, new_value)
}

#[cfg(feature = "update")]
/// 更新器配置选项
#[derive(Debug, Clone)]
pub struct UpdaterConfig {
    /// 是否自动创建缺失的中间路径
    pub create_missing_paths: bool,
    /// 是否允许类型转换（如将非对象转换为对象）
    pub allow_type_conversion: bool,
    /// 最大递归深度
    pub max_recursion_depth: usize,
}

#[cfg(feature = "update")]
impl Default for UpdaterConfig {
    fn default() -> Self {
        Self {
            create_missing_paths: true,
            allow_type_conversion: true,
            max_recursion_depth: 1000,
        }
    }
}

#[cfg(feature = "update")]
/// 可配置的更新器
pub struct ConfigurableUpdater {
    config: UpdaterConfig,
}

#[cfg(feature = "update")]
impl ConfigurableUpdater {
    /// 创建新的可配置更新器
    pub fn new(config: UpdaterConfig) -> Self {
        Self { config }
    }
}

#[cfg(feature = "update")]
impl Default for ConfigurableUpdater {
    fn default() -> Self {
        Self::new(UpdaterConfig::default())
    }
}

#[cfg(feature = "update")]
impl ConfigurableUpdater {
    /// 更新字段（带配置）
    pub fn update(
        &self,
        root: &mut Value,
        path: &[PathSegment],
        new_value: Value,
    ) -> Result<(), UpdateError> {
        if path.is_empty() {
            *root = new_value;
            return Ok(());
        }

        self.update_with_depth(root, path, new_value, 0)
    }

    /// 带深度控制的更新
    fn update_with_depth(
        &self,
        current: &mut Value,
        remaining_path: &[PathSegment],
        new_value: Value,
        depth: usize,
    ) -> Result<(), UpdateError> {
        if depth > self.config.max_recursion_depth {
            return Err(UpdateError::InvalidPath(
                "Maximum recursion depth exceeded".to_string(),
            ));
        }

        if remaining_path.is_empty() {
            *current = new_value;
            return Ok(());
        }

        let (current_segment, rest_path) =
            remaining_path.split_first().unwrap();

        match current_segment {
            PathSegment::Field(field_name) => self.update_field_with_config(
                current,
                field_name,
                rest_path,
                new_value,
                depth + 1,
            ),
            PathSegment::Index(index) => self.update_index_with_config(
                current,
                *index,
                rest_path,
                new_value,
                depth + 1,
            ),
            PathSegment::Wildcard => self.update_wildcard_with_config(
                current,
                rest_path,
                new_value,
                depth + 1,
            ),
            PathSegment::RecursiveWildcard => {
                Err(UpdateError::InvalidOperation(
                    "Cannot update with recursive wildcard".to_string(),
                ))
            }
            PathSegment::TypeFilter(_) => Err(UpdateError::InvalidOperation(
                "Cannot update with type filter".to_string(),
            )),
        }
    }

    /// 带配置的字段更新
    fn update_field_with_config(
        &self,
        current: &mut Value,
        field_name: &str,
        remaining_path: &[PathSegment],
        new_value: Value,
        depth: usize,
    ) -> Result<(), UpdateError> {
        // 根据配置决定是否进行类型转换
        if !current.is_object() {
            if current.is_null() || self.config.allow_type_conversion {
                *current = serde_json::json!({});
            } else {
                return Err(UpdateError::TypeMismatch(
                    "object".to_string(),
                    Updater::get_value_type_name(current).to_string(),
                ));
            }
        }

        let obj = current.as_object_mut().unwrap();

        if remaining_path.is_empty() {
            obj.insert(field_name.to_string(), new_value);
            Ok(())
        } else {
            if !obj.contains_key(field_name) {
                if self.config.create_missing_paths {
                    let intermediate_value =
                        Updater::create_intermediate_value(&remaining_path[0]);
                    obj.insert(field_name.to_string(), intermediate_value);
                } else {
                    return Err(UpdateError::PathNotFound(
                        field_name.to_string(),
                    ));
                }
            }

            let field_value = obj.get_mut(field_name).unwrap();
            self.update_with_depth(
                field_value,
                remaining_path,
                new_value,
                depth,
            )
        }
    }

    /// 带配置的索引更新
    fn update_index_with_config(
        &self,
        current: &mut Value,
        index: usize,
        remaining_path: &[PathSegment],
        new_value: Value,
        depth: usize,
    ) -> Result<(), UpdateError> {
        if !current.is_array() {
            if current.is_null() || self.config.allow_type_conversion {
                *current = serde_json::json!([]);
            } else {
                return Err(UpdateError::TypeMismatch(
                    "array".to_string(),
                    Updater::get_value_type_name(current).to_string(),
                ));
            }
        }

        let arr = current.as_array_mut().unwrap();

        if self.config.create_missing_paths {
            while arr.len() <= index {
                arr.push(Value::Null);
            }
        } else if index >= arr.len() {
            return Err(UpdateError::IndexOutOfBounds(index, arr.len()));
        }

        if remaining_path.is_empty() {
            arr[index] = new_value;
            Ok(())
        } else {
            if arr[index].is_null() && self.config.create_missing_paths {
                arr[index] =
                    Updater::create_intermediate_value(&remaining_path[0]);
            }

            self.update_with_depth(
                &mut arr[index],
                remaining_path,
                new_value,
                depth,
            )
        }
    }

    /// 带配置的通配符更新
    fn update_wildcard_with_config(
        &self,
        current: &mut Value,
        remaining_path: &[PathSegment],
        new_value: Value,
        depth: usize,
    ) -> Result<(), UpdateError> {
        match current {
            Value::Object(map) => {
                for (_, field_value) in map.iter_mut() {
                    self.update_with_depth(
                        field_value,
                        remaining_path,
                        new_value.clone(),
                        depth,
                    )?;
                }
                Ok(())
            }
            Value::Array(arr) => {
                for item in arr.iter_mut() {
                    self.update_with_depth(
                        item,
                        remaining_path,
                        new_value.clone(),
                        depth,
                    )?;
                }
                Ok(())
            }
            _ => Err(UpdateError::TypeMismatch(
                "object or array".to_string(),
                Updater::get_value_type_name(current).to_string(),
            )),
        }
    }
}

// 当 update feature 未启用时，提供占位符
#[cfg(not(feature = "update"))]
pub struct UpdateError;

#[cfg(not(feature = "update"))]
impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Update functionality is not enabled. Compile with --features update")
    }
}

#[cfg(not(feature = "update"))]
impl std::error::Error for UpdateError {}

#[cfg(all(test, feature = "update"))]
mod tests {
    use super::*;
    use crate::parser::path::parse_path;
    use serde_json::json;

    #[test]
    fn test_update_field() {
        let mut data = json!({"name": "Alice", "age": 30});
        let path = parse_path(".name").unwrap();

        update(&mut data, &path, json!("Bob")).unwrap();
        assert_eq!(data["name"], "Bob");
    }

    #[test]
    fn test_update_index() {
        let mut data = json!(["Alice", "Bob", "Charlie"]);
        let path = parse_path("[1]").unwrap();

        update(&mut data, &path, json!("Robert")).unwrap();
        assert_eq!(data[1], "Robert");
    }

    #[test]
    fn test_update_nested_path() {
        let mut data = json!({
            "user": {
                "profile": {
                    "name": "Alice"
                }
            }
        });

        let path = parse_path(".user.profile.name").unwrap();
        update(&mut data, &path, json!("Bob")).unwrap();
        assert_eq!(data["user"]["profile"]["name"], "Bob");
    }

    #[test]
    fn test_create_missing_path() {
        let mut data = json!({});
        let path = parse_path(".user.profile.name").unwrap();

        update(&mut data, &path, json!("Alice")).unwrap();
        assert_eq!(data["user"]["profile"]["name"], "Alice");
    }

    #[test]
    fn test_update_wildcard() {
        let mut data = json!({
            "users": [
                {"active": false},
                {"active": false}
            ]
        });

        let path = parse_path(".users[*].active").unwrap();
        update(&mut data, &path, json!(true)).unwrap();

        assert_eq!(data["users"][0]["active"], true);
        assert_eq!(data["users"][1]["active"], true);
    }

    #[test]
    fn test_configurable_updater() {
        let config = UpdaterConfig {
            create_missing_paths: false,
            allow_type_conversion: false,
            max_recursion_depth: 100,
        };
        let updater = ConfigurableUpdater::new(config);

        let mut data = json!({"name": "Alice"});
        let path = parse_path(".missing.field").unwrap();

        let result = updater.update(&mut data, &path, json!("value"));
        assert!(result.is_err()); // 应该失败，因为不允许创建缺失路径
    }

    #[test]
    fn test_array_expansion() {
        let mut data = json!([1, 2]);
        let path = parse_path("[5]").unwrap();

        update(&mut data, &path, json!(6)).unwrap();

        // 数组应该被扩展，中间元素为 null
        assert_eq!(data.as_array().unwrap().len(), 6);
        assert_eq!(data[5], 6);
        assert_eq!(data[3], Value::Null);
    }
}
