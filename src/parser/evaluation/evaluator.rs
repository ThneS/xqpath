use crate::parser::{
    ast::{ComparisonOp, LogicalOp, PathExpression},
    functions::FunctionRegistry,
    path::PathSegment,
};
use serde_json::Value;
use std::cmp::Ordering;

use super::error::EvaluationError;

/// 表达式求值器
pub struct ExpressionEvaluator {
    function_registry: FunctionRegistry,
}

impl Default for ExpressionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionEvaluator {
    /// 创建新的求值器
    pub fn new() -> Self {
        Self {
            function_registry: FunctionRegistry::new(),
        }
    }

    /// 对给定值评估路径表达式
    pub fn evaluate(
        &self,
        expression: &PathExpression,
        value: &Value,
    ) -> Result<Vec<Value>, EvaluationError> {
        match expression {
            PathExpression::Segments(segments) => {
                // 使用现有的路径段处理逻辑
                Self::evaluate_segments(segments, value)
            }

            PathExpression::Pipe { left, right } => {
                // 管道操作：将左表达式的结果作为右表达式的输入
                let left_results = self.evaluate(left, value)?;
                let mut final_results = Vec::new();

                for left_result in left_results {
                    let right_results = self.evaluate(right, &left_result)?;
                    final_results.extend(right_results);
                }

                Ok(final_results)
            }

            PathExpression::Comma(expressions) => {
                // 逗号操作：收集所有表达式的结果
                let mut all_results = Vec::new();

                for expr in expressions {
                    let results = self.evaluate(expr, value)?;
                    all_results.extend(results);
                }

                Ok(all_results)
            }

            PathExpression::Literal(literal) => {
                // 字面量直接返回
                Ok(vec![literal.clone()])
            }

            PathExpression::Identity => {
                // 恒等表达式返回输入值
                Ok(vec![value.clone()])
            }

            PathExpression::FunctionCall { name, args } => {
                // 首先尝试高级函数（支持表达式参数）
                if let Some(advanced_function) =
                    self.function_registry.get_advanced(name)
                {
                    return advanced_function
                        .execute_with_expressions(args, self, value);
                }

                // 然后尝试基础函数
                if let Some(function) = self.function_registry.get(name) {
                    // 评估函数参数
                    let mut evaluated_args = Vec::new();
                    for arg in args {
                        let arg_results = self.evaluate(arg, value)?;
                        // 对于函数参数，我们通常只取第一个结果
                        // 更复杂的函数可能需要处理多个结果
                        if let Some(first_result) = arg_results.first() {
                            evaluated_args.push(first_result.clone());
                        }
                    }

                    return function.execute(&evaluated_args, value);
                }

                // 如果都找不到，返回未知函数错误
                Err(EvaluationError::UnknownFunction(name.clone()))
            }

            PathExpression::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                // 条件表达式：if condition then expr1 else expr2 end
                let condition_results = self.evaluate(condition, value)?;

                // 检查第一个条件结果的真值
                let is_truthy = condition_results
                    .first()
                    .map(|v| self.is_truthy(v))
                    .unwrap_or(false);

                if is_truthy {
                    self.evaluate(then_expr, value)
                } else if let Some(else_expr) = else_expr {
                    self.evaluate(else_expr, value)
                } else {
                    Ok(vec![Value::Null])
                }
            }

            PathExpression::Comparison { left, op, right } => {
                // 比较操作：left op right
                let left_results = self.evaluate(left, value)?;
                let right_results = self.evaluate(right, value)?;

                let left_value = left_results.first().unwrap_or(&Value::Null);
                let right_value = right_results.first().unwrap_or(&Value::Null);

                let result =
                    self.compare_values(left_value, op, right_value)?;
                Ok(vec![Value::Bool(result)])
            }

            PathExpression::Logical { op, operands } => {
                // 逻辑操作：operand1 op operand2 或 not operand
                match op {
                    LogicalOp::And => {
                        for operand in operands {
                            let results = self.evaluate(operand, value)?;
                            let is_truthy = results
                                .first()
                                .map(|v| self.is_truthy(v))
                                .unwrap_or(false);
                            if !is_truthy {
                                return Ok(vec![Value::Bool(false)]);
                            }
                        }
                        Ok(vec![Value::Bool(true)])
                    }
                    LogicalOp::Or => {
                        for operand in operands {
                            let results = self.evaluate(operand, value)?;
                            let is_truthy = results
                                .first()
                                .map(|v| self.is_truthy(v))
                                .unwrap_or(false);
                            if is_truthy {
                                return Ok(vec![Value::Bool(true)]);
                            }
                        }
                        Ok(vec![Value::Bool(false)])
                    }
                    LogicalOp::Not => {
                        if operands.len() != 1 {
                            return Err(EvaluationError::InvalidArguments(
                                "not operator requires exactly one operand"
                                    .to_string(),
                            ));
                        }
                        let results = self.evaluate(&operands[0], value)?;
                        let is_truthy = results
                            .first()
                            .map(|v| self.is_truthy(v))
                            .unwrap_or(false);
                        Ok(vec![Value::Bool(!is_truthy)])
                    }
                }
            }

            PathExpression::TryCatch {
                try_expr,
                catch_expr,
            } => {
                // try-catch 表达式：尝试执行 try_expr，如果失败则执行 catch_expr
                match self.evaluate(try_expr, value) {
                    Ok(results) => Ok(results),
                    Err(_error) => {
                        if let Some(catch_expr) = catch_expr {
                            // 执行 catch 表达式
                            self.evaluate(catch_expr, value)
                        } else {
                            // 如果没有 catch 表达式，返回 null
                            Ok(vec![Value::Null])
                        }
                    }
                }
            }

            PathExpression::Optional(expr) => {
                // 可选操作符：如果表达式执行失败，返回 null 而不是错误
                match self.evaluate(expr, value) {
                    Ok(results) => {
                        // 如果结果为空，返回 null
                        if results.is_empty() {
                            Ok(vec![Value::Null])
                        } else {
                            Ok(results)
                        }
                    }
                    Err(_) => Ok(vec![Value::Null]),
                }
            }
        }
    }

    /// 判断值是否为真值（jq-style truthiness）
    pub fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Object(obj) => !obj.is_empty(),
        }
    }

    /// 比较两个值
    fn compare_values(
        &self,
        left: &Value,
        op: &ComparisonOp,
        right: &Value,
    ) -> Result<bool, EvaluationError> {
        let comparison = match (left, right) {
            // 相同类型比较
            (Value::Number(l), Value::Number(r)) => {
                let l_f64 = l.as_f64().unwrap_or(0.0);
                let r_f64 = r.as_f64().unwrap_or(0.0);
                l_f64.partial_cmp(&r_f64).unwrap_or(Ordering::Equal)
            }
            (Value::String(l), Value::String(r)) => l.cmp(r),
            (Value::Bool(l), Value::Bool(r)) => l.cmp(r),

            // null 与任何值比较
            (Value::Null, Value::Null) => Ordering::Equal,
            (Value::Null, _) => Ordering::Less,
            (_, Value::Null) => Ordering::Greater,

            // 不同类型比较：转换为字符串比较
            _ => {
                let l_str = serde_json::to_string(left).map_err(|_| {
                    EvaluationError::Message(
                        "Failed to serialize left value".to_string(),
                    )
                })?;
                let r_str = serde_json::to_string(right).map_err(|_| {
                    EvaluationError::Message(
                        "Failed to serialize right value".to_string(),
                    )
                })?;
                l_str.cmp(&r_str)
            }
        };

        let result = match op {
            ComparisonOp::Equal => comparison == Ordering::Equal,
            ComparisonOp::NotEqual => comparison != Ordering::Equal,
            ComparisonOp::LessThan => comparison == Ordering::Less,
            ComparisonOp::LessThanOrEqual => comparison != Ordering::Greater,
            ComparisonOp::GreaterThan => comparison == Ordering::Greater,
            ComparisonOp::GreaterThanOrEqual => comparison != Ordering::Less,
        };

        Ok(result)
    }

    /// 评估路径段序列（重用现有逻辑）
    fn evaluate_segments(
        segments: &[PathSegment],
        value: &Value,
    ) -> Result<Vec<Value>, EvaluationError> {
        if segments.is_empty() {
            return Ok(vec![value.clone()]);
        }

        let mut current_values = vec![value.clone()];

        for segment in segments {
            let mut next_values = Vec::new();

            for current_value in current_values {
                let results = Self::evaluate_segment(segment, &current_value)?;
                next_values.extend(results);
            }

            current_values = next_values;
        }

        Ok(current_values)
    }

    /// 评估单个路径段
    fn evaluate_segment(
        segment: &PathSegment,
        value: &Value,
    ) -> Result<Vec<Value>, EvaluationError> {
        match segment {
            PathSegment::Field(field_name) => {
                match value {
                    Value::Object(map) => {
                        if let Some(field_value) = map.get(field_name) {
                            Ok(vec![field_value.clone()])
                        } else {
                            Ok(vec![]) // 字段不存在，返回空结果
                        }
                    }
                    _ => Ok(vec![]), // 非对象类型，返回空结果
                }
            }

            PathSegment::Index(index) => {
                match value {
                    Value::Array(arr) => {
                        if *index < arr.len() {
                            Ok(vec![arr[*index].clone()])
                        } else {
                            Ok(vec![]) // 索引越界，返回空结果
                        }
                    }
                    _ => Ok(vec![]), // 非数组类型，返回空结果
                }
            }

            PathSegment::Wildcard => {
                match value {
                    Value::Object(map) => Ok(map.values().cloned().collect()),
                    Value::Array(arr) => Ok(arr.clone()),
                    _ => Ok(vec![]), // 非容器类型，返回空结果
                }
            }

            PathSegment::RecursiveWildcard => {
                // 递归收集所有值
                Ok(Self::collect_recursive(value))
            }

            PathSegment::TypeFilter(type_name) => {
                // 类型过滤
                if Self::matches_type(value, type_name) {
                    Ok(vec![value.clone()])
                } else {
                    Ok(vec![])
                }
            }
        }
    }

    /// 递归收集所有值
    fn collect_recursive(value: &Value) -> Vec<Value> {
        let mut results = vec![value.clone()];

        match value {
            Value::Object(map) => {
                for field_value in map.values() {
                    results.extend(Self::collect_recursive(field_value));
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    results.extend(Self::collect_recursive(item));
                }
            }
            _ => {} // 基本类型，只包含自身
        }

        results
    }

    /// 检查值是否匹配类型
    fn matches_type(value: &Value, type_name: &str) -> bool {
        match type_name {
            "null" => value.is_null(),
            "boolean" | "bool" => value.is_boolean(),
            "number" => value.is_number(),
            "string" => value.is_string(),
            "array" => value.is_array(),
            "object" => value.is_object(),
            _ => false,
        }
    }
}

/// 便利函数：评估路径表达式
pub fn evaluate_path_expression(
    expression: &PathExpression,
    value: &Value,
) -> Result<Vec<Value>, EvaluationError> {
    let evaluator = ExpressionEvaluator::new();
    evaluator.evaluate(expression, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::PathExpression;
    use crate::parser::path::PathSegment;
    use serde_json::json;

    #[test]
    fn test_evaluate_simple_path() {
        let expr = PathExpression::Segments(vec![PathSegment::Field(
            "name".to_string(),
        )]);
        let value = json!({"name": "Alice", "age": 30});

        let result = evaluate_path_expression(&expr, &value).unwrap();
        assert_eq!(result, vec![json!("Alice")]);
    }

    #[test]
    fn test_evaluate_identity() {
        let expr = PathExpression::Identity;
        let value = json!({"name": "Alice"});

        let result = evaluate_path_expression(&expr, &value).unwrap();
        assert_eq!(result, vec![json!({"name": "Alice"})]);
    }

    #[test]
    fn test_evaluate_literal() {
        let expr = PathExpression::Literal(json!("hello"));
        let value = json!({"name": "Alice"});

        let result = evaluate_path_expression(&expr, &value).unwrap();
        assert_eq!(result, vec![json!("hello")]);
    }

    #[test]
    fn test_evaluate_pipe() {
        let expr = PathExpression::pipe(
            PathExpression::Segments(vec![PathSegment::Field(
                "user".to_string(),
            )]),
            PathExpression::Segments(vec![PathSegment::Field(
                "name".to_string(),
            )]),
        );
        let value = json!({"user": {"name": "Alice", "age": 30}});

        let result = evaluate_path_expression(&expr, &value).unwrap();
        assert_eq!(result, vec![json!("Alice")]);
    }

    #[test]
    fn test_evaluate_comma() {
        let expr = PathExpression::comma(vec![
            PathExpression::Segments(vec![PathSegment::Field(
                "name".to_string(),
            )]),
            PathExpression::Segments(vec![PathSegment::Field(
                "age".to_string(),
            )]),
        ]);
        let value = json!({"name": "Alice", "age": 30});

        let result = evaluate_path_expression(&expr, &value).unwrap();
        assert_eq!(result, vec![json!("Alice"), json!(30)]);
    }

    #[test]
    fn test_evaluate_wildcard() {
        let expr = PathExpression::Segments(vec![
            PathSegment::Field("users".to_string()),
            PathSegment::Wildcard,
            PathSegment::Field("name".to_string()),
        ]);
        let value = json!({"users": [{"name": "Alice"}, {"name": "Bob"}]});

        let result = evaluate_path_expression(&expr, &value).unwrap();
        assert_eq!(result, vec![json!("Alice"), json!("Bob")]);
    }

    #[test]
    fn test_evaluate_complex_expression() {
        let expr = PathExpression::comma(vec![
            PathExpression::pipe(
                PathExpression::Segments(vec![
                    PathSegment::Field("users".to_string()),
                    PathSegment::Wildcard,
                    PathSegment::Field("name".to_string()),
                ]),
                PathExpression::Identity, // 管道到恒等表达式
            ),
            PathExpression::Literal(json!("total")),
        ]);
        let value = json!({"users": [{"name": "Alice"}, {"name": "Bob"}]});

        let result = evaluate_path_expression(&expr, &value).unwrap();
        assert_eq!(result, vec![json!("Alice"), json!("Bob"), json!("total")]);
    }
}
