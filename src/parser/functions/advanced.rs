use super::AdvancedBuiltinFunction;
use crate::parser::{EvaluationError, ExpressionEvaluator, PathExpression};
use serde_json::Value;

/// map 函数 - 对数组每个元素应用表达式
pub struct MapFunction;

impl AdvancedBuiltinFunction for MapFunction {
    fn name(&self) -> &str {
        "map"
    }

    fn execute_with_expressions(
        &self,
        args: &[PathExpression],
        evaluator: &ExpressionEvaluator,
        input: &Value,
    ) -> Result<Vec<Value>, EvaluationError> {
        if args.len() != 1 {
            return Err(EvaluationError::InvalidArguments(
                "map function takes exactly one expression argument"
                    .to_string(),
            ));
        }

        match input {
            Value::Array(arr) => {
                let mut results = Vec::new();
                for item in arr {
                    let item_results = evaluator.evaluate(&args[0], item)?;
                    results.extend(item_results);
                }
                Ok(vec![Value::Array(results)])
            }
            _ => Err(EvaluationError::InvalidArguments(
                "map can only be applied to arrays".to_string(),
            )),
        }
    }

    fn description(&self) -> &str {
        "Applies an expression to each element of an array and collects the results"
    }
}

/// select 函数 - 过滤满足条件的元素
pub struct SelectFunction;

impl AdvancedBuiltinFunction for SelectFunction {
    fn name(&self) -> &str {
        "select"
    }

    fn execute_with_expressions(
        &self,
        args: &[PathExpression],
        evaluator: &ExpressionEvaluator,
        input: &Value,
    ) -> Result<Vec<Value>, EvaluationError> {
        if args.len() != 1 {
            return Err(EvaluationError::InvalidArguments(
                "select function takes exactly one expression argument"
                    .to_string(),
            ));
        }

        match input {
            Value::Array(arr) => {
                let mut results = Vec::new();
                for item in arr {
                    let condition_results =
                        evaluator.evaluate(&args[0], item)?;
                    let is_truthy = condition_results
                        .first()
                        .map(|v| evaluator.is_truthy(v))
                        .unwrap_or(false);
                    if is_truthy {
                        results.push(item.clone());
                    }
                }
                Ok(vec![Value::Array(results)])
            }
            _ => {
                // 对于非数组值，直接应用条件判断
                let condition_results = evaluator.evaluate(&args[0], input)?;
                let is_truthy = condition_results
                    .first()
                    .map(|v| evaluator.is_truthy(v))
                    .unwrap_or(false);
                if is_truthy {
                    Ok(vec![input.clone()])
                } else {
                    Ok(vec![])
                }
            }
        }
    }

    fn description(&self) -> &str {
        "Filters elements that satisfy the given condition expression"
    }
}

/// sort 函数 - 简单排序数组
pub struct SortFunction;

impl AdvancedBuiltinFunction for SortFunction {
    fn name(&self) -> &str {
        "sort"
    }

    fn execute_with_expressions(
        &self,
        args: &[PathExpression],
        _evaluator: &ExpressionEvaluator,
        input: &Value,
    ) -> Result<Vec<Value>, EvaluationError> {
        if !args.is_empty() {
            return Err(EvaluationError::InvalidArguments(
                "sort function takes no arguments".to_string(),
            ));
        }

        match input {
            Value::Array(arr) => {
                let mut sorted_arr = arr.clone();
                sorted_arr.sort_by(|a, b| {
                    // 简单的排序逻辑，按值类型优先级排序
                    use std::cmp::Ordering;
                    match (a, b) {
                        (Value::Number(n1), Value::Number(n2)) => n1
                            .as_f64()
                            .unwrap_or(0.0)
                            .partial_cmp(&n2.as_f64().unwrap_or(0.0))
                            .unwrap_or(Ordering::Equal),
                        (Value::String(s1), Value::String(s2)) => s1.cmp(s2),
                        (Value::Bool(b1), Value::Bool(b2)) => b1.cmp(b2),
                        (Value::Null, Value::Null) => Ordering::Equal,
                        (Value::Null, _) => Ordering::Less,
                        (_, Value::Null) => Ordering::Greater,
                        _ => Ordering::Equal,
                    }
                });
                Ok(vec![Value::Array(sorted_arr)])
            }
            _ => Err(EvaluationError::InvalidArguments(
                "sort can only be applied to arrays".to_string(),
            )),
        }
    }

    fn description(&self) -> &str {
        "Sorts array elements in ascending order"
    }
}

/// sort_by 函数 - 按表达式结果排序
pub struct SortByFunction;

impl AdvancedBuiltinFunction for SortByFunction {
    fn name(&self) -> &str {
        "sort_by"
    }

    fn execute_with_expressions(
        &self,
        args: &[PathExpression],
        evaluator: &ExpressionEvaluator,
        input: &Value,
    ) -> Result<Vec<Value>, EvaluationError> {
        if args.len() != 1 {
            return Err(EvaluationError::InvalidArguments(
                "sort_by function takes exactly one expression argument"
                    .to_string(),
            ));
        }

        match input {
            Value::Array(arr) => {
                let mut indexed_items: Vec<(Value, Value)> = Vec::new();

                // 计算每个元素的排序键
                for item in arr {
                    let key_results = evaluator.evaluate(&args[0], item)?;
                    let sort_key =
                        key_results.first().cloned().unwrap_or(Value::Null);
                    indexed_items.push((item.clone(), sort_key));
                }

                // 按排序键排序
                indexed_items.sort_by(|a, b| {
                    use std::cmp::Ordering;
                    match (&a.1, &b.1) {
                        (Value::Number(n1), Value::Number(n2)) => n1
                            .as_f64()
                            .unwrap_or(0.0)
                            .partial_cmp(&n2.as_f64().unwrap_or(0.0))
                            .unwrap_or(Ordering::Equal),
                        (Value::String(s1), Value::String(s2)) => s1.cmp(s2),
                        (Value::Bool(b1), Value::Bool(b2)) => b1.cmp(b2),
                        (Value::Null, Value::Null) => Ordering::Equal,
                        (Value::Null, _) => Ordering::Less,
                        (_, Value::Null) => Ordering::Greater,
                        _ => Ordering::Equal,
                    }
                });

                let sorted_items: Vec<Value> =
                    indexed_items.into_iter().map(|(item, _)| item).collect();
                Ok(vec![Value::Array(sorted_items)])
            }
            _ => Err(EvaluationError::InvalidArguments(
                "sort_by can only be applied to arrays".to_string(),
            )),
        }
    }

    fn description(&self) -> &str {
        "Sorts array elements by the result of applying the given expression"
    }
}

/// group_by 函数 - 按表达式结果分组
pub struct GroupByFunction;

impl AdvancedBuiltinFunction for GroupByFunction {
    fn name(&self) -> &str {
        "group_by"
    }

    fn execute_with_expressions(
        &self,
        args: &[PathExpression],
        evaluator: &ExpressionEvaluator,
        input: &Value,
    ) -> Result<Vec<Value>, EvaluationError> {
        if args.len() != 1 {
            return Err(EvaluationError::InvalidArguments(
                "group_by function takes exactly one expression argument"
                    .to_string(),
            ));
        }

        match input {
            Value::Array(arr) => {
                let mut groups: std::collections::HashMap<String, Vec<Value>> =
                    std::collections::HashMap::new();

                // 按分组键分组
                for item in arr {
                    let key_results = evaluator.evaluate(&args[0], item)?;
                    let group_key =
                        key_results.first().cloned().unwrap_or(Value::Null);
                    let key_str = match group_key {
                        Value::String(s) => s,
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Null => "null".to_string(),
                        _ => serde_json::to_string(&group_key)
                            .unwrap_or_else(|_| "unknown".to_string()),
                    };

                    groups.entry(key_str).or_default().push(item.clone());
                }

                // 转换为分组数组
                let mut group_arrays: Vec<Value> =
                    groups.into_values().map(Value::Array).collect();
                group_arrays.sort_by(|a, b| {
                    // 按第一个元素的分组键排序
                    match (a, b) {
                        (Value::Array(arr1), Value::Array(arr2)) => {
                            if let (Some(first1), Some(first2)) =
                                (arr1.first(), arr2.first())
                            {
                                let key1 = evaluator
                                    .evaluate(&args[0], first1)
                                    .ok()
                                    .and_then(|results| {
                                        results.first().cloned()
                                    })
                                    .unwrap_or(Value::Null);
                                let key2 = evaluator
                                    .evaluate(&args[0], first2)
                                    .ok()
                                    .and_then(|results| {
                                        results.first().cloned()
                                    })
                                    .unwrap_or(Value::Null);

                                match (&key1, &key2) {
                                    (Value::String(s1), Value::String(s2)) => {
                                        s1.cmp(s2)
                                    }
                                    (Value::Number(n1), Value::Number(n2)) => {
                                        n1.as_f64()
                                            .unwrap_or(0.0)
                                            .partial_cmp(
                                                &n2.as_f64().unwrap_or(0.0),
                                            )
                                            .unwrap_or(
                                                std::cmp::Ordering::Equal,
                                            )
                                    }
                                    _ => std::cmp::Ordering::Equal,
                                }
                            } else {
                                std::cmp::Ordering::Equal
                            }
                        }
                        _ => std::cmp::Ordering::Equal,
                    }
                });

                Ok(vec![Value::Array(group_arrays)])
            }
            _ => Err(EvaluationError::InvalidArguments(
                "group_by can only be applied to arrays".to_string(),
            )),
        }
    }

    fn description(&self) -> &str {
        "Groups array elements by the result of applying the given expression"
    }
}

/// unique 函数 - 去除重复元素
pub struct UniqueFunction;

impl AdvancedBuiltinFunction for UniqueFunction {
    fn name(&self) -> &str {
        "unique"
    }

    fn execute_with_expressions(
        &self,
        args: &[PathExpression],
        _evaluator: &ExpressionEvaluator,
        input: &Value,
    ) -> Result<Vec<Value>, EvaluationError> {
        if !args.is_empty() {
            return Err(EvaluationError::InvalidArguments(
                "unique function takes no arguments".to_string(),
            ));
        }

        match input {
            Value::Array(arr) => {
                let mut unique_items = Vec::new();
                let mut seen = std::collections::HashSet::new();

                for item in arr {
                    let key = serde_json::to_string(item).unwrap_or_default();
                    if seen.insert(key) {
                        unique_items.push(item.clone());
                    }
                }

                Ok(vec![Value::Array(unique_items)])
            }
            _ => Err(EvaluationError::InvalidArguments(
                "unique can only be applied to arrays".to_string(),
            )),
        }
    }

    fn description(&self) -> &str {
        "Removes duplicate elements from an array, preserving order"
    }
}

/// unique_by 函数 - 按表达式结果去重
pub struct UniqueByFunction;

impl AdvancedBuiltinFunction for UniqueByFunction {
    fn name(&self) -> &str {
        "unique_by"
    }

    fn execute_with_expressions(
        &self,
        args: &[PathExpression],
        evaluator: &ExpressionEvaluator,
        input: &Value,
    ) -> Result<Vec<Value>, EvaluationError> {
        if args.len() != 1 {
            return Err(EvaluationError::InvalidArguments(
                "unique_by function takes exactly one expression argument"
                    .to_string(),
            ));
        }

        match input {
            Value::Array(arr) => {
                let mut unique_items = Vec::new();
                let mut seen = std::collections::HashSet::new();

                for item in arr {
                    let key_results = evaluator.evaluate(&args[0], item)?;
                    let unique_key =
                        key_results.first().cloned().unwrap_or(Value::Null);
                    let key_str =
                        serde_json::to_string(&unique_key).unwrap_or_default();

                    if seen.insert(key_str) {
                        unique_items.push(item.clone());
                    }
                }

                Ok(vec![Value::Array(unique_items)])
            }
            _ => Err(EvaluationError::InvalidArguments(
                "unique_by can only be applied to arrays".to_string(),
            )),
        }
    }

    fn description(&self) -> &str {
        "Removes duplicate elements from an array based on the result of applying the given expression"
    }
}

/// reverse 函数 - 反转数组
pub struct ReverseFunction;

impl AdvancedBuiltinFunction for ReverseFunction {
    fn name(&self) -> &str {
        "reverse"
    }

    fn execute_with_expressions(
        &self,
        args: &[PathExpression],
        _evaluator: &ExpressionEvaluator,
        input: &Value,
    ) -> Result<Vec<Value>, EvaluationError> {
        if !args.is_empty() {
            return Err(EvaluationError::InvalidArguments(
                "reverse function takes no arguments".to_string(),
            ));
        }

        match input {
            Value::Array(arr) => {
                let mut reversed_arr = arr.clone();
                reversed_arr.reverse();
                Ok(vec![Value::Array(reversed_arr)])
            }
            _ => Err(EvaluationError::InvalidArguments(
                "reverse can only be applied to arrays".to_string(),
            )),
        }
    }

    fn description(&self) -> &str {
        "Reverses the order of elements in an array"
    }
}
