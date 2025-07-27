use crate::parser::path::{PathSegment, ParseResult, ParseError};
use serde_json::Value;
use std::fmt;
use winnow::{
    ascii::{alpha1, digit1},
    combinator::{alt, delimited, repeat},
    token::{take_while, take_until},
    PResult, Parser,
};

/// 路径表达式抽象语法树
#[derive(Debug, Clone, PartialEq)]
pub enum PathExpression {
    /// 简单路径段序列（向后兼容原有语法）
    Segments(Vec<PathSegment>),

    /// 管道操作: left | right
    Pipe {
        left: Box<PathExpression>,
        right: Box<PathExpression>,
    },

    /// 逗号操作: expr1, expr2, ...
    Comma(Vec<PathExpression>),

    /// 字面量值
    Literal(Value),

    /// 恒等表达式 "."
    Identity,
}

impl PathExpression {
    /// 检查表达式是否为简单路径（向后兼容）
    pub fn is_simple_path(&self) -> bool {
        matches!(self, PathExpression::Segments(_))
    }

    /// 转换为简单路径段（向后兼容）
    pub fn as_segments(&self) -> Option<&[PathSegment]> {
        match self {
            PathExpression::Segments(segments) => Some(segments),
            _ => None,
        }
    }

    /// 创建简单路径表达式
    pub fn from_segments(segments: Vec<PathSegment>) -> Self {
        PathExpression::Segments(segments)
    }

    /// 创建管道表达式
    pub fn pipe(left: PathExpression, right: PathExpression) -> Self {
        PathExpression::Pipe {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// 创建逗号表达式
    pub fn comma(expressions: Vec<PathExpression>) -> Self {
        if expressions.len() == 1 {
            expressions.into_iter().next().unwrap()
        } else {
            PathExpression::Comma(expressions)
        }
    }

    /// 获取表达式的字符串表示（用于调试）
    pub fn to_string(&self) -> String {
        match self {
            PathExpression::Segments(segments) => {
                segments.iter()
                    .map(|s| match s {
                        PathSegment::Field(name) => format!(".{}", name),
                        PathSegment::Index(idx) => format!("[{}]", idx),
                        PathSegment::Wildcard => "*".to_string(),
                        PathSegment::RecursiveWildcard => "**".to_string(),
                        PathSegment::TypeFilter(typ) => format!("| {}", typ),
                    })
                    .collect::<Vec<_>>()
                    .join("")
            }

            PathExpression::Pipe { left, right } => {
                format!("{} | {}", left.to_string(), right.to_string())
            }

            PathExpression::Comma(exprs) => {
                exprs.iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            }

            PathExpression::Literal(value) => {
                serde_json::to_string(value).unwrap_or_else(|_| "null".to_string())
            }

            PathExpression::Identity => ".".to_string(),
        }
    }
}

impl fmt::Display for PathExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// 表达式复杂度分析（用于性能优化）
#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionComplexity {
    /// 嵌套深度
    pub depth: usize,
    /// 管道操作数量
    pub pipe_count: usize,
    /// 逗号分支数量
    pub comma_branches: usize,
    /// 是否包含通配符
    pub has_wildcards: bool,
    /// 是否包含递归通配符
    pub has_recursive_wildcards: bool,
}

impl PathExpression {
    /// 分析表达式复杂度
    pub fn analyze_complexity(&self) -> ExpressionComplexity {
        self.analyze_complexity_with_depth(0)
    }

    fn analyze_complexity_with_depth(&self, current_depth: usize) -> ExpressionComplexity {
        match self {
            PathExpression::Segments(segments) => {
                let has_wildcards = segments.iter().any(|s| matches!(s, PathSegment::Wildcard));
                let has_recursive_wildcards = segments.iter().any(|s| matches!(s, PathSegment::RecursiveWildcard));

                ExpressionComplexity {
                    depth: current_depth + 1,
                    pipe_count: 0,
                    comma_branches: 1,
                    has_wildcards,
                    has_recursive_wildcards,
                }
            }

            PathExpression::Pipe { left, right } => {
                let left_complexity = left.analyze_complexity_with_depth(current_depth + 1);
                let right_complexity = right.analyze_complexity_with_depth(current_depth + 1);

                ExpressionComplexity {
                    depth: left_complexity.depth.max(right_complexity.depth),
                    pipe_count: left_complexity.pipe_count + right_complexity.pipe_count + 1,
                    comma_branches: left_complexity.comma_branches * right_complexity.comma_branches,
                    has_wildcards: left_complexity.has_wildcards || right_complexity.has_wildcards,
                    has_recursive_wildcards: left_complexity.has_recursive_wildcards || right_complexity.has_recursive_wildcards,
                }
            }

            PathExpression::Comma(exprs) => {
                let mut max_depth = current_depth + 1;
                let mut total_pipe_count = 0;
                let mut total_branches = 0;
                let mut has_wildcards = false;
                let mut has_recursive_wildcards = false;

                for expr in exprs {
                    let complexity = expr.analyze_complexity_with_depth(current_depth + 1);
                    max_depth = max_depth.max(complexity.depth);
                    total_pipe_count += complexity.pipe_count;
                    total_branches += complexity.comma_branches;
                    has_wildcards = has_wildcards || complexity.has_wildcards;
                    has_recursive_wildcards = has_recursive_wildcards || complexity.has_recursive_wildcards;
                }

                ExpressionComplexity {
                    depth: max_depth,
                    pipe_count: total_pipe_count,
                    comma_branches: total_branches,
                    has_wildcards,
                    has_recursive_wildcards,
                }
            }

            PathExpression::Literal(_) | PathExpression::Identity => {
                ExpressionComplexity {
                    depth: current_depth + 1,
                    pipe_count: 0,
                    comma_branches: 1,
                    has_wildcards: false,
                    has_recursive_wildcards: false,
                }
            }
        }
    }

    /// 判断表达式是否需要特殊优化
    pub fn needs_optimization(&self) -> bool {
        let complexity = self.analyze_complexity();
        complexity.depth > 5 ||
        complexity.pipe_count > 3 ||
        complexity.comma_branches > 10 ||
        complexity.has_recursive_wildcards
    }
}

/// 表达式解析器
pub struct ExpressionParser;

impl ExpressionParser {
    /// 主解析函数：解析完整的路径表达式
    pub fn parse_path_expression(input: &str) -> ParseResult<PathExpression> {
        let mut input_ref = input;
        match Self::parse_comma_expression.parse_next(&mut input_ref) {
            Ok(expr) => {
                match Self::skip_whitespace.parse_next(&mut input_ref) {
                    Ok(_) => {
                        if input_ref.is_empty() {
                            Ok(expr)
                        } else {
                            Err(ParseError {
                                message: format!("Unexpected characters: '{}'", input_ref),
                                position: input.len() - input_ref.len(),
                            })
                        }
                    }
                    Err(_) => Err(ParseError {
                        message: "Failed to skip whitespace".to_string(),
                        position: input.len() - input_ref.len(),
                    }),
                }
            }
            Err(e) => Err(ParseError {
                message: format!("Failed to parse expression: {:?}", e),
                position: input.len() - input_ref.len(),
            }),
        }
    }

    /// 解析逗号表达式（最低优先级）
    fn parse_comma_expression(input: &mut &str) -> PResult<PathExpression> {
        let first = Self::parse_pipe_expression.parse_next(input)?;

        // 检查是否有更多逗号分隔的表达式
        let mut expressions = vec![first];

        while Self::try_parse_comma.parse_next(input).is_ok() {
            let next = Self::parse_pipe_expression.parse_next(input)?;
            expressions.push(next);
        }

        Ok(if expressions.len() == 1 {
            expressions.into_iter().next().unwrap()
        } else {
            PathExpression::Comma(expressions)
        })
    }

    /// 解析管道表达式（中等优先级）
    fn parse_pipe_expression(input: &mut &str) -> PResult<PathExpression> {
        let mut left = Self::parse_primary_expression.parse_next(input)?;

        while Self::try_parse_pipe.parse_next(input).is_ok() {
            let right = Self::parse_primary_expression.parse_next(input)?;
            left = PathExpression::pipe(left, right);
        }

        Ok(left)
    }

    /// 解析基础表达式（最高优先级）
    fn parse_primary_expression(input: &mut &str) -> PResult<PathExpression> {
        let _ = Self::skip_whitespace.parse_next(input);

        alt((
            Self::parse_literal,
            Self::parse_parenthesized,
            Self::parse_path_or_identity,
        )).parse_next(input)
    }

    /// 解析路径或恒等表达式
    fn parse_path_or_identity(input: &mut &str) -> PResult<PathExpression> {
        // 先尝试解析路径段
        let segments = Self::parse_path_segments(input)?;

        // 如果只有一个段，并且是单独的点，则返回 Identity
        if segments.is_empty() {
            // 检查是否为单独的点
            if input.starts_with(".") {
                '.'.value(PathExpression::Identity).parse_next(input)
            } else {
                Err(winnow::error::ErrMode::Backtrack(
                    winnow::error::ParserError::from_error_kind(
                        input,
                        winnow::error::ErrorKind::Verify,
                    ),
                ))
            }
        } else {
            Ok(PathExpression::Segments(segments))
        }
    }

    /// 解析字面量值
    fn parse_literal(input: &mut &str) -> PResult<PathExpression> {
        alt((
            Self::parse_string_literal,
            Self::parse_number_literal,
            Self::parse_boolean_literal,
            Self::parse_null_literal,
        )).parse_next(input)
    }

    /// 解析字符串字面量
    fn parse_string_literal(input: &mut &str) -> PResult<PathExpression> {
        delimited(
            '"',
            take_until(0.., "\""),
            '"'
        )
        .map(|s: &str| PathExpression::Literal(Value::String(s.to_string())))
        .parse_next(input)
    }

    /// 解析数字字面量
    fn parse_number_literal(input: &mut &str) -> PResult<PathExpression> {
        // 简单的整数解析
        digit1
            .try_map(|s: &str| s.parse::<i64>())
            .map(|n| PathExpression::Literal(Value::Number(serde_json::Number::from(n))))
            .parse_next(input)
    }

    /// 解析布尔字面量
    fn parse_boolean_literal(input: &mut &str) -> PResult<PathExpression> {
        alt((
            "true".value(PathExpression::Literal(Value::Bool(true))),
            "false".value(PathExpression::Literal(Value::Bool(false))),
        )).parse_next(input)
    }

    /// 解析 null 字面量
    fn parse_null_literal(input: &mut &str) -> PResult<PathExpression> {
        "null".value(PathExpression::Literal(Value::Null)).parse_next(input)
    }

    /// 解析括号表达式
    fn parse_parenthesized(input: &mut &str) -> PResult<PathExpression> {
        delimited(
            ('(', Self::skip_whitespace),
            Self::parse_comma_expression,
            (Self::skip_whitespace, ')')
        ).parse_next(input)
    }

    /// 解析路径段序列的内部实现
    fn parse_path_segments(input: &mut &str) -> PResult<Vec<PathSegment>> {
        // 检查是否是单独的点
        if input.starts_with(".") && (input.len() == 1 ||
           input.chars().nth(1).map_or(true, |c| c.is_whitespace() || ")|,".contains(c))) {
            // 这是单独的恒等表达式，返回空段列表
            return Ok(vec![]);
        }

        repeat(1.., Self::parse_segment).parse_next(input)
    }

    /// 解析单个路径段（不包括类型过滤器，因为它会与管道操作符冲突）
    fn parse_segment(input: &mut &str) -> PResult<PathSegment> {
        alt((
            Self::parse_recursive_wildcard,
            // 注意：在表达式上下文中不解析类型过滤器，避免与管道操作符冲突
            Self::parse_field,
            Self::parse_index,
            Self::parse_wildcard,
        )).parse_next(input)
    }

    /// 解析字段访问
    fn parse_field(input: &mut &str) -> PResult<PathSegment> {
        alt((
            // 带点的字段访问 .field
            ('.', Self::parse_identifier).map(|(_, name)| PathSegment::Field(name)),
            // 裸字段名
            Self::parse_identifier.map(PathSegment::Field),
        )).parse_next(input)
    }

    /// 解析数组索引
    fn parse_index(input: &mut &str) -> PResult<PathSegment> {
        delimited(
            '[',
            alt((
                '*'.value(PathSegment::Wildcard),
                Self::parse_number.map(PathSegment::Index),
                winnow::combinator::empty.value(PathSegment::Wildcard),
            )),
            ']',
        ).parse_next(input)
    }

    /// 解析通配符
    fn parse_wildcard(input: &mut &str) -> PResult<PathSegment> {
        // 确保这不是 **
        if input.starts_with("**") {
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ParserError::from_error_kind(
                    input,
                    winnow::error::ErrorKind::Verify,
                ),
            ));
        }
        '*'.value(PathSegment::Wildcard).parse_next(input)
    }

    /// 解析递归通配符
    fn parse_recursive_wildcard(input: &mut &str) -> PResult<PathSegment> {
        "**".value(PathSegment::RecursiveWildcard).parse_next(input)
    }

    /// 解析类型过滤器（保留为备用，但在表达式解析中不使用）
    #[allow(dead_code)]
    fn parse_type_filter(input: &mut &str) -> PResult<PathSegment> {
        (Self::skip_whitespace, '|', Self::skip_whitespace, Self::parse_identifier)
            .map(|(_, _, _, type_name)| PathSegment::TypeFilter(type_name))
            .parse_next(input)
    }

    /// 解析标识符
    fn parse_identifier(input: &mut &str) -> PResult<String> {
        (
            alpha1,
            take_while(0.., |c: char| c.is_alphanumeric() || c == '_'),
        )
        .recognize()
        .map(|s: &str| s.to_string())
        .parse_next(input)
    }

    /// 解析数字
    fn parse_number(input: &mut &str) -> PResult<usize> {
        digit1.try_map(|s: &str| s.parse()).parse_next(input)
    }

    /// 跳过空白字符
    fn skip_whitespace(input: &mut &str) -> PResult<()> {
        take_while(0.., |c: char| c == ' ' || c == '\t' || c == '\n' || c == '\r')
            .void()
            .parse_next(input)
    }

    /// 尝试解析逗号
    fn try_parse_comma(input: &mut &str) -> PResult<()> {
        (Self::skip_whitespace, ',', Self::skip_whitespace)
            .void()
            .parse_next(input)
    }

    /// 尝试解析管道
    fn try_parse_pipe(input: &mut &str) -> PResult<()> {
        (Self::skip_whitespace, '|', Self::skip_whitespace)
            .void()
            .parse_next(input)
    }
}

/// 便利函数：解析路径表达式
pub fn parse_path_expression(input: &str) -> ParseResult<PathExpression> {
    ExpressionParser::parse_path_expression(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::path::{PathSegment};

    #[test]
    fn test_simple_path_creation() {
        let segments = vec![
            PathSegment::Field("user".to_string()),
            PathSegment::Field("name".to_string()),
        ];
        let expr = PathExpression::from_segments(segments.clone());

        assert!(expr.is_simple_path());
        assert_eq!(expr.as_segments(), Some(segments.as_slice()));
        assert_eq!(expr.to_string(), ".user.name");
    }

    #[test]
    fn test_pipe_expression_creation() {
        let left = PathExpression::from_segments(vec![PathSegment::Field("users".to_string())]);
        let right = PathExpression::from_segments(vec![PathSegment::Index(0)]);
        let pipe_expr = PathExpression::pipe(left, right);

        assert!(!pipe_expr.is_simple_path());
        assert_eq!(pipe_expr.to_string(), ".users | [0]");
    }

    #[test]
    fn test_comma_expression_creation() {
        let expr1 = PathExpression::from_segments(vec![PathSegment::Field("name".to_string())]);
        let expr2 = PathExpression::from_segments(vec![PathSegment::Field("age".to_string())]);
        let comma_expr = PathExpression::comma(vec![expr1, expr2]);

        assert!(!comma_expr.is_simple_path());
        assert_eq!(comma_expr.to_string(), ".name, .age");
    }

    #[test]
    fn test_identity_expression() {
        let identity = PathExpression::Identity;
        assert_eq!(identity.to_string(), ".");
    }

    #[test]
    fn test_literal_expression() {
        let literal = PathExpression::Literal(serde_json::json!("hello"));
        assert_eq!(literal.to_string(), "\"hello\"");
    }

    #[test]
    fn test_complexity_analysis() {
        // 简单路径
        let simple = PathExpression::from_segments(vec![PathSegment::Field("name".to_string())]);
        let complexity = simple.analyze_complexity();
        assert_eq!(complexity.depth, 1);
        assert_eq!(complexity.pipe_count, 0);
        assert_eq!(complexity.comma_branches, 1);
        assert!(!complexity.has_wildcards);

        // 带通配符的路径
        let wildcard = PathExpression::from_segments(vec![
            PathSegment::Field("users".to_string()),
            PathSegment::Wildcard,
            PathSegment::Field("name".to_string()),
        ]);
        let complexity = wildcard.analyze_complexity();
        assert!(complexity.has_wildcards);

        // 管道表达式
        let pipe = PathExpression::pipe(simple.clone(), simple.clone());
        let complexity = pipe.analyze_complexity();
        assert_eq!(complexity.pipe_count, 1);
        assert!(complexity.depth >= 2);
    }

    #[test]
    fn test_complex_expression_string() {
        let left = PathExpression::from_segments(vec![
            PathSegment::Field("users".to_string()),
            PathSegment::Wildcard,
        ]);
        let right = PathExpression::from_segments(vec![
            PathSegment::Field("name".to_string()),
        ]);
        let pipe = PathExpression::pipe(left, right);

        let age = PathExpression::from_segments(vec![PathSegment::Field("age".to_string())]);
        let comma = PathExpression::comma(vec![pipe, age]);

        assert_eq!(comma.to_string(), ".users* | .name, .age");
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;
    use crate::parser::path::PathSegment;

    #[test]
    fn test_parse_simple_path() {
        let result = parse_path_expression(".name").unwrap();
        assert_eq!(
            result,
            PathExpression::Segments(vec![PathSegment::Field("name".to_string())])
        );
    }

    #[test]
    fn test_parse_identity() {
        let result = parse_path_expression(".").unwrap();
        assert_eq!(result, PathExpression::Identity);
    }

    #[test]
    fn test_parse_literal_string() {
        let result = parse_path_expression("\"hello\"").unwrap();
        assert_eq!(
            result,
            PathExpression::Literal(Value::String("hello".to_string()))
        );
    }

    #[test]
    fn test_parse_literal_number() {
        let result = parse_path_expression("42").unwrap();
        assert_eq!(
            result,
            PathExpression::Literal(Value::Number(serde_json::Number::from(42)))
        );
    }

    #[test]
    fn test_parse_literal_boolean() {
        let result = parse_path_expression("true").unwrap();
        assert_eq!(result, PathExpression::Literal(Value::Bool(true)));

        let result = parse_path_expression("false").unwrap();
        assert_eq!(result, PathExpression::Literal(Value::Bool(false)));
    }

    #[test]
    fn test_parse_literal_null() {
        let result = parse_path_expression("null").unwrap();
        assert_eq!(result, PathExpression::Literal(Value::Null));
    }

    #[test]
    fn test_parse_simple_pipe() {
        let result = parse_path_expression(".name | .length").unwrap();
        let expected = PathExpression::pipe(
            PathExpression::Segments(vec![PathSegment::Field("name".to_string())]),
            PathExpression::Segments(vec![PathSegment::Field("length".to_string())]),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_simple_comma() {
        let result = parse_path_expression(".name, .age").unwrap();
        let expected = PathExpression::comma(vec![
            PathExpression::Segments(vec![PathSegment::Field("name".to_string())]),
            PathExpression::Segments(vec![PathSegment::Field("age".to_string())]),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_complex_expression() {
        let result = parse_path_expression(".users[*].name | length, .users | length").unwrap();
        let expected = PathExpression::comma(vec![
            PathExpression::pipe(
                PathExpression::Segments(vec![
                    PathSegment::Field("users".to_string()),
                    PathSegment::Wildcard,
                    PathSegment::Field("name".to_string()),
                ]),
                PathExpression::Segments(vec![PathSegment::Field("length".to_string())]),
            ),
            PathExpression::pipe(
                PathExpression::Segments(vec![PathSegment::Field("users".to_string())]),
                PathExpression::Segments(vec![PathSegment::Field("length".to_string())]),
            ),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_parenthesized() {
        let result = parse_path_expression("(.name | .length), .age").unwrap();
        let expected = PathExpression::comma(vec![
            PathExpression::pipe(
                PathExpression::Segments(vec![PathSegment::Field("name".to_string())]),
                PathExpression::Segments(vec![PathSegment::Field("length".to_string())]),
            ),
            PathExpression::Segments(vec![PathSegment::Field("age".to_string())]),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_identity_pipe() {
        let result = parse_path_expression(". | .name").unwrap();
        let expected = PathExpression::pipe(
            PathExpression::Identity,
            PathExpression::Segments(vec![PathSegment::Field("name".to_string())]),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_error_unclosed_string() {
        let result = parse_path_expression("\"unclosed");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_unexpected_chars() {
        let result = parse_path_expression(".name @");
        assert!(result.is_err());
    }
}

/// 表达式求值器
pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    /// 对给定值评估路径表达式
    pub fn evaluate(expression: &PathExpression, value: &Value) -> Result<Vec<Value>, EvaluationError> {
        match expression {
            PathExpression::Segments(segments) => {
                // 使用现有的路径段处理逻辑
                Self::evaluate_segments(segments, value)
            }

            PathExpression::Pipe { left, right } => {
                // 管道操作：将左表达式的结果作为右表达式的输入
                let left_results = Self::evaluate(left, value)?;
                let mut final_results = Vec::new();

                for left_result in left_results {
                    let right_results = Self::evaluate(right, &left_result)?;
                    final_results.extend(right_results);
                }

                Ok(final_results)
            }

            PathExpression::Comma(expressions) => {
                // 逗号操作：收集所有表达式的结果
                let mut all_results = Vec::new();

                for expr in expressions {
                    let results = Self::evaluate(expr, value)?;
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
        }
    }

    /// 评估路径段序列（重用现有逻辑）
    fn evaluate_segments(segments: &[PathSegment], value: &Value) -> Result<Vec<Value>, EvaluationError> {
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
    fn evaluate_segment(segment: &PathSegment, value: &Value) -> Result<Vec<Value>, EvaluationError> {
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
                    Value::Object(map) => {
                        Ok(map.values().cloned().collect())
                    }
                    Value::Array(arr) => {
                        Ok(arr.clone())
                    }
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

/// 求值错误类型
#[derive(Debug, Clone)]
pub struct EvaluationError {
    pub message: String,
}

impl std::fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Evaluation error: {}", self.message)
    }
}

impl std::error::Error for EvaluationError {}

/// 便利函数：评估路径表达式
pub fn evaluate_path_expression(expression: &PathExpression, value: &Value) -> Result<Vec<Value>, EvaluationError> {
    ExpressionEvaluator::evaluate(expression, value)
}

#[cfg(test)]
mod evaluator_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_evaluate_simple_path() {
        let expr = PathExpression::Segments(vec![PathSegment::Field("name".to_string())]);
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
            PathExpression::Segments(vec![PathSegment::Field("user".to_string())]),
            PathExpression::Segments(vec![PathSegment::Field("name".to_string())]),
        );
        let value = json!({"user": {"name": "Alice", "age": 30}});

        let result = evaluate_path_expression(&expr, &value).unwrap();
        assert_eq!(result, vec![json!("Alice")]);
    }

    #[test]
    fn test_evaluate_comma() {
        let expr = PathExpression::comma(vec![
            PathExpression::Segments(vec![PathSegment::Field("name".to_string())]),
            PathExpression::Segments(vec![PathSegment::Field("age".to_string())]),
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
