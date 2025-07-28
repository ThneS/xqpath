use super::BuiltinFunction;
use crate::parser::EvaluationError;
use serde_json::Value;

/// length 函数 - 获取数组/对象/字符串长度
pub struct LengthFunction;

impl BuiltinFunction for LengthFunction {
    fn name(&self) -> &str {
        "length"
    }

    fn execute(
        &self,
        args: &[Value],
        input: &Value,
    ) -> Result<Vec<Value>, EvaluationError> {
        if !args.is_empty() {
            return Err(EvaluationError::InvalidArguments(
                "length function takes no arguments".to_string(),
            ));
        }

        let length = match input {
            Value::Array(arr) => arr.len(),
            Value::Object(obj) => obj.len(),
            Value::String(s) => s.chars().count(),
            Value::Null => 0,
            _ => return Err(EvaluationError::InvalidArguments(
                "length can only be applied to arrays, objects, strings, or null".to_string()
            )),
        };

        Ok(vec![Value::Number(length.into())])
    }

    fn description(&self) -> &str {
        "Returns the length of arrays, objects, strings, or 0 for null"
    }
}

/// type 函数 - 获取值类型
pub struct TypeFunction;

impl BuiltinFunction for TypeFunction {
    fn name(&self) -> &str {
        "type"
    }

    fn execute(
        &self,
        args: &[Value],
        input: &Value,
    ) -> Result<Vec<Value>, EvaluationError> {
        if !args.is_empty() {
            return Err(EvaluationError::InvalidArguments(
                "type function takes no arguments".to_string(),
            ));
        }

        let type_name = match input {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        };

        Ok(vec![Value::String(type_name.to_string())])
    }

    fn description(&self) -> &str {
        "Returns the type of the input value"
    }
}

/// keys 函数 - 获取对象键名或数组索引
pub struct KeysFunction;

impl BuiltinFunction for KeysFunction {
    fn name(&self) -> &str {
        "keys"
    }

    fn execute(
        &self,
        args: &[Value],
        input: &Value,
    ) -> Result<Vec<Value>, EvaluationError> {
        if !args.is_empty() {
            return Err(EvaluationError::InvalidArguments(
                "keys function takes no arguments".to_string(),
            ));
        }

        match input {
            Value::Object(obj) => {
                let mut keys: Vec<String> = obj.keys().cloned().collect();
                keys.sort();
                let key_values: Vec<Value> =
                    keys.into_iter().map(Value::String).collect();
                Ok(vec![Value::Array(key_values)])
            }
            Value::Array(arr) => {
                let indices: Vec<Value> =
                    (0..arr.len()).map(|i| Value::Number(i.into())).collect();
                Ok(vec![Value::Array(indices)])
            }
            _ => Err(EvaluationError::InvalidArguments(
                "keys can only be applied to objects or arrays".to_string(),
            )),
        }
    }

    fn description(&self) -> &str {
        "Returns sorted keys of an object or indices of an array"
    }
}

/// values 函数 - 获取对象所有值
pub struct ValuesFunction;

impl BuiltinFunction for ValuesFunction {
    fn name(&self) -> &str {
        "values"
    }

    fn execute(
        &self,
        args: &[Value],
        input: &Value,
    ) -> Result<Vec<Value>, EvaluationError> {
        if !args.is_empty() {
            return Err(EvaluationError::InvalidArguments(
                "values function takes no arguments".to_string(),
            ));
        }

        match input {
            Value::Object(obj) => {
                let values: Vec<Value> = obj.values().cloned().collect();
                Ok(vec![Value::Array(values)])
            }
            Value::Array(arr) => Ok(vec![Value::Array(arr.clone())]),
            _ => Err(EvaluationError::InvalidArguments(
                "values can only be applied to objects or arrays".to_string(),
            )),
        }
    }

    fn description(&self) -> &str {
        "Returns all values of an object or array"
    }
}
