use winnow::{
    ascii::{alpha1, digit1},
    combinator::{alt, delimited, empty, repeat},
    token::{take_until, take_while},
    PResult, Parser,
};

use crate::parser::{
    ast::{ComparisonOp, LogicalOp, PathExpression},
    path::{ParseError, ParseResult, PathSegment},
};
use serde_json::Value;

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
                                message: format!(
                                    "Unexpected characters: '{input_ref}'"
                                ),
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
                message: format!("Failed to parse expression: {e:?}"),
                position: input.len() - input_ref.len(),
            }),
        }
    }

    /// 解析逗号表达式（最低优先级）
    fn parse_comma_expression(input: &mut &str) -> PResult<PathExpression> {
        let first = Self::parse_conditional_expression.parse_next(input)?;

        // 检查是否有更多逗号分隔的表达式
        let mut expressions = vec![first];

        while Self::try_parse_comma.parse_next(input).is_ok() {
            let next = Self::parse_conditional_expression.parse_next(input)?;
            expressions.push(next);
        }

        Ok(if expressions.len() == 1 {
            expressions.into_iter().next().unwrap()
        } else {
            PathExpression::Comma(expressions)
        })
    }

    /// 解析条件表达式（if-then-else）和 try-catch 表达式
    fn parse_conditional_expression(
        input: &mut &str,
    ) -> PResult<PathExpression> {
        let _ = Self::skip_whitespace.parse_next(input);

        // 尝试解析 try 关键字
        if Self::try_parse_try.parse_next(input).is_ok() {
            let try_expr =
                Self::parse_logical_or_expression.parse_next(input)?;

            let catch_expr = if Self::try_parse_catch.parse_next(input).is_ok()
            {
                Some(Box::new(
                    Self::parse_logical_or_expression.parse_next(input)?,
                ))
            } else {
                None
            };

            return Ok(PathExpression::TryCatch {
                try_expr: Box::new(try_expr),
                catch_expr,
            });
        }

        // 尝试解析 if 关键字
        if Self::try_parse_if.parse_next(input).is_ok() {
            let condition =
                Self::parse_logical_or_expression.parse_next(input)?;

            Self::parse_then.parse_next(input)?;
            let then_expr =
                Self::parse_logical_or_expression.parse_next(input)?;

            let else_expr = if Self::try_parse_else.parse_next(input).is_ok() {
                Some(Box::new(
                    Self::parse_logical_or_expression.parse_next(input)?,
                ))
            } else {
                None
            };

            Self::parse_end.parse_next(input)?;

            Ok(PathExpression::Conditional {
                condition: Box::new(condition),
                then_expr: Box::new(then_expr),
                else_expr,
            })
        } else {
            Self::parse_logical_or_expression.parse_next(input)
        }
    }

    /// 解析逻辑or表达式
    fn parse_logical_or_expression(
        input: &mut &str,
    ) -> PResult<PathExpression> {
        let mut left = Self::parse_logical_and_expression.parse_next(input)?;

        while Self::try_parse_or.parse_next(input).is_ok() {
            let right = Self::parse_logical_and_expression.parse_next(input)?;
            left = PathExpression::Logical {
                op: LogicalOp::Or,
                operands: vec![left, right],
            };
        }

        Ok(left)
    }

    /// 解析逻辑and表达式
    fn parse_logical_and_expression(
        input: &mut &str,
    ) -> PResult<PathExpression> {
        let mut left = Self::parse_logical_not_expression.parse_next(input)?;

        while Self::try_parse_and.parse_next(input).is_ok() {
            let right = Self::parse_logical_not_expression.parse_next(input)?;
            left = PathExpression::Logical {
                op: LogicalOp::And,
                operands: vec![left, right],
            };
        }

        Ok(left)
    }

    /// 解析逻辑not表达式
    fn parse_logical_not_expression(
        input: &mut &str,
    ) -> PResult<PathExpression> {
        let _ = Self::skip_whitespace.parse_next(input);

        if Self::try_parse_not.parse_next(input).is_ok() {
            let operand =
                Self::parse_comparison_expression.parse_next(input)?;
            Ok(PathExpression::Logical {
                op: LogicalOp::Not,
                operands: vec![operand],
            })
        } else {
            Self::parse_comparison_expression.parse_next(input)
        }
    }

    /// 解析比较表达式
    fn parse_comparison_expression(
        input: &mut &str,
    ) -> PResult<PathExpression> {
        let mut left = Self::parse_pipe_expression.parse_next(input)?;

        loop {
            let _ = Self::skip_whitespace.parse_next(input);

            let op = if Self::try_parse_lte.parse_next(input).is_ok() {
                ComparisonOp::LessThanOrEqual
            } else if Self::try_parse_gte.parse_next(input).is_ok() {
                ComparisonOp::GreaterThanOrEqual
            } else if Self::try_parse_eq.parse_next(input).is_ok() {
                ComparisonOp::Equal
            } else if Self::try_parse_ne.parse_next(input).is_ok() {
                ComparisonOp::NotEqual
            } else if Self::try_parse_lt.parse_next(input).is_ok() {
                ComparisonOp::LessThan
            } else if Self::try_parse_gt.parse_next(input).is_ok() {
                ComparisonOp::GreaterThan
            } else {
                break;
            };

            let right = Self::parse_pipe_expression.parse_next(input)?;
            left = PathExpression::Comparison {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// 解析管道表达式
    fn parse_pipe_expression(input: &mut &str) -> PResult<PathExpression> {
        let mut left = Self::parse_primary_expression.parse_next(input)?;

        while Self::try_parse_pipe.parse_next(input).is_ok() {
            let right = Self::parse_primary_expression.parse_next(input)?;
            left = PathExpression::pipe(left, right);
        }

        // 检查是否有可选操作符 ? (用于管道表达式后)
        let _ = Self::skip_whitespace.parse_next(input);
        if input.starts_with('?') {
            '?'.parse_next(input)?;
            left = PathExpression::Optional(Box::new(left));
        }

        Ok(left)
    }

    /// 解析基础表达式（最高优先级）
    fn parse_primary_expression(input: &mut &str) -> PResult<PathExpression> {
        let _ = Self::skip_whitespace.parse_next(input);

        let mut expr = alt((
            Self::parse_literal,
            Self::parse_parenthesized,
            Self::parse_function_call,
            Self::parse_path_or_identity,
        ))
        .parse_next(input)?;

        // 检查是否有可选操作符 ?
        let _ = Self::skip_whitespace.parse_next(input);
        if input.starts_with('?') {
            '?'.parse_next(input)?;
            expr = PathExpression::Optional(Box::new(expr));
        }

        Ok(expr)
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

    /// 解析函数调用
    fn parse_function_call(input: &mut &str) -> PResult<PathExpression> {
        // 函数名（字母开头，后跟字母数字或下划线）
        let function_name = (
            alpha1,
            take_while(0.., |c: char| c.is_alphanumeric() || c == '_'),
        )
            .recognize()
            .parse_next(input)?;

        let _ = Self::skip_whitespace.parse_next(input);

        // 检查是否有左括号
        if !input.starts_with('(') {
            // 如果没有括号，可能是无参数函数，但这里先要求必须有括号
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ParserError::from_error_kind(
                    input,
                    winnow::error::ErrorKind::Verify,
                ),
            ));
        }

        '('.parse_next(input)?;
        let _ = Self::skip_whitespace.parse_next(input);

        // 解析参数列表
        let mut args = Vec::new();

        // 检查是否是空参数列表
        if !input.starts_with(')') {
            // 解析第一个参数
            args.push(Self::parse_comma_expression.parse_next(input)?);
            let _ = Self::skip_whitespace.parse_next(input);

            // 解析后续参数
            while input.starts_with(',') {
                ','.parse_next(input)?;
                let _ = Self::skip_whitespace.parse_next(input);
                args.push(Self::parse_comma_expression.parse_next(input)?);
                let _ = Self::skip_whitespace.parse_next(input);
            }
        }

        ')'.parse_next(input)?;

        Ok(PathExpression::FunctionCall {
            name: function_name.to_string(),
            args,
        })
    }

    /// 解析字面量值
    fn parse_literal(input: &mut &str) -> PResult<PathExpression> {
        alt((
            Self::parse_array_literal,
            Self::parse_object_literal,
            Self::parse_string_literal,
            Self::parse_number_literal,
            Self::parse_boolean_literal,
            Self::parse_null_literal,
        ))
        .parse_next(input)
    }

    /// 解析数组字面量
    fn parse_array_literal(input: &mut &str) -> PResult<PathExpression> {
        let _ = Self::skip_whitespace.parse_next(input);
        '['.parse_next(input)?;
        let _ = Self::skip_whitespace.parse_next(input);

        let mut elements = Vec::new();

        // 检查是否是空数组
        if !input.starts_with(']') {
            // 解析第一个元素
            if let Ok(literal) = Self::parse_simple_literal(input) {
                elements.push(literal);
                let _ = Self::skip_whitespace.parse_next(input);

                // 解析后续元素
                while input.starts_with(',') {
                    ','.parse_next(input)?;
                    let _ = Self::skip_whitespace.parse_next(input);
                    let literal = Self::parse_simple_literal(input)?;
                    elements.push(literal);
                    let _ = Self::skip_whitespace.parse_next(input);
                }
            }
        }

        ']'.parse_next(input)?;
        Ok(PathExpression::Literal(Value::Array(elements)))
    }

    /// 解析对象字面量（简化版本）
    fn parse_object_literal(input: &mut &str) -> PResult<PathExpression> {
        let _ = Self::skip_whitespace.parse_next(input);
        '{'.parse_next(input)?;
        let _ = Self::skip_whitespace.parse_next(input);

        let mut object = serde_json::Map::new();

        // 检查是否是空对象
        if !input.starts_with('}') {
            // 简化实现，只支持字符串键
            loop {
                // 解析键
                let key = delimited('"', take_until(0.., "\""), '"')
                    .parse_next(input)?;
                let _ = Self::skip_whitespace.parse_next(input);
                ':'.parse_next(input)?;
                let _ = Self::skip_whitespace.parse_next(input);

                // 解析值
                let value = Self::parse_simple_literal(input)?;
                object.insert(key.to_string(), value);

                let _ = Self::skip_whitespace.parse_next(input);
                if input.starts_with(',') {
                    ','.parse_next(input)?;
                    let _ = Self::skip_whitespace.parse_next(input);
                } else {
                    break;
                }
            }
        }

        '}'.parse_next(input)?;
        Ok(PathExpression::Literal(Value::Object(object)))
    }

    /// 解析简单字面量值（用于数组和对象内部）
    fn parse_simple_literal(input: &mut &str) -> PResult<Value> {
        let _ = Self::skip_whitespace.parse_next(input);
        alt((
            // 字符串
            delimited('"', take_until(0.., "\""), '"')
                .map(|s: &str| Value::String(s.to_string())),
            // 数字
            digit1
                .try_map(|s: &str| s.parse::<i64>())
                .map(|n| Value::Number(serde_json::Number::from(n))),
            // 布尔值
            alt((
                "true".value(Value::Bool(true)),
                "false".value(Value::Bool(false)),
            )),
            // null
            "null".value(Value::Null),
        ))
        .parse_next(input)
    }

    /// 解析字符串字面量
    fn parse_string_literal(input: &mut &str) -> PResult<PathExpression> {
        delimited('"', take_until(0.., "\""), '"')
            .map(|s: &str| {
                PathExpression::Literal(Value::String(s.to_string()))
            })
            .parse_next(input)
    }

    /// 解析数字字面量
    fn parse_number_literal(input: &mut &str) -> PResult<PathExpression> {
        // 简单的整数解析
        digit1
            .try_map(|s: &str| s.parse::<i64>())
            .map(|n| {
                PathExpression::Literal(Value::Number(
                    serde_json::Number::from(n),
                ))
            })
            .parse_next(input)
    }

    /// 解析布尔字面量
    fn parse_boolean_literal(input: &mut &str) -> PResult<PathExpression> {
        alt((
            "true".value(PathExpression::Literal(Value::Bool(true))),
            "false".value(PathExpression::Literal(Value::Bool(false))),
        ))
        .parse_next(input)
    }

    /// 解析 null 字面量
    fn parse_null_literal(input: &mut &str) -> PResult<PathExpression> {
        "null"
            .value(PathExpression::Literal(Value::Null))
            .parse_next(input)
    }

    /// 解析括号表达式
    fn parse_parenthesized(input: &mut &str) -> PResult<PathExpression> {
        delimited(
            ('(', Self::skip_whitespace),
            Self::parse_comma_expression,
            (Self::skip_whitespace, ')'),
        )
        .parse_next(input)
    }

    /// 解析路径段序列的内部实现
    fn parse_path_segments(input: &mut &str) -> PResult<Vec<PathSegment>> {
        // 检查是否是单独的点
        if input.starts_with(".")
            && (input.len() == 1
                || input
                    .chars()
                    .nth(1)
                    .is_none_or(|c| c.is_whitespace() || ")|,".contains(c)))
        {
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
        ))
        .parse_next(input)
    }

    /// 解析字段访问
    fn parse_field(input: &mut &str) -> PResult<PathSegment> {
        alt((
            // 带点的字段访问 .field
            ('.', Self::parse_identifier)
                .map(|(_, name)| PathSegment::Field(name)),
            // 裸字段名
            Self::parse_identifier.map(PathSegment::Field),
        ))
        .parse_next(input)
    }

    /// 解析数组索引
    fn parse_index(input: &mut &str) -> PResult<PathSegment> {
        delimited(
            '[',
            alt((
                '*'.value(PathSegment::Wildcard),
                Self::parse_number.map(PathSegment::Index),
                empty.value(PathSegment::Wildcard),
            )),
            ']',
        )
        .parse_next(input)
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
        (
            Self::skip_whitespace,
            '|',
            Self::skip_whitespace,
            Self::parse_identifier,
        )
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
        take_while(0.., |c: char| {
            c == ' ' || c == '\t' || c == '\n' || c == '\r'
        })
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

    // 条件表达式关键字解析器
    fn try_parse_if(input: &mut &str) -> PResult<()> {
        (Self::skip_whitespace, "if", Self::skip_whitespace)
            .void()
            .parse_next(input)
    }

    fn parse_then(input: &mut &str) -> PResult<()> {
        (Self::skip_whitespace, "then", Self::skip_whitespace)
            .void()
            .parse_next(input)
    }

    fn try_parse_else(input: &mut &str) -> PResult<()> {
        (Self::skip_whitespace, "else", Self::skip_whitespace)
            .void()
            .parse_next(input)
    }

    fn parse_end(input: &mut &str) -> PResult<()> {
        (Self::skip_whitespace, "end", Self::skip_whitespace)
            .void()
            .parse_next(input)
    }

    // try-catch 表达式关键字解析器
    fn try_parse_try(input: &mut &str) -> PResult<()> {
        (Self::skip_whitespace, "try", Self::skip_whitespace)
            .void()
            .parse_next(input)
    }

    fn try_parse_catch(input: &mut &str) -> PResult<()> {
        (Self::skip_whitespace, "catch", Self::skip_whitespace)
            .void()
            .parse_next(input)
    }

    // 逻辑操作符解析器
    fn try_parse_or(input: &mut &str) -> PResult<()> {
        (Self::skip_whitespace, "or", Self::skip_whitespace)
            .void()
            .parse_next(input)
    }

    fn try_parse_and(input: &mut &str) -> PResult<()> {
        (Self::skip_whitespace, "and", Self::skip_whitespace)
            .void()
            .parse_next(input)
    }

    fn try_parse_not(input: &mut &str) -> PResult<()> {
        (Self::skip_whitespace, "not", Self::skip_whitespace)
            .void()
            .parse_next(input)
    }

    // 比较操作符解析器
    fn try_parse_lte(input: &mut &str) -> PResult<()> {
        (Self::skip_whitespace, "<=", Self::skip_whitespace)
            .void()
            .parse_next(input)
    }

    fn try_parse_gte(input: &mut &str) -> PResult<()> {
        (Self::skip_whitespace, ">=", Self::skip_whitespace)
            .void()
            .parse_next(input)
    }

    fn try_parse_eq(input: &mut &str) -> PResult<()> {
        (Self::skip_whitespace, "==", Self::skip_whitespace)
            .void()
            .parse_next(input)
    }

    fn try_parse_ne(input: &mut &str) -> PResult<()> {
        (Self::skip_whitespace, "!=", Self::skip_whitespace)
            .void()
            .parse_next(input)
    }

    fn try_parse_lt(input: &mut &str) -> PResult<()> {
        (Self::skip_whitespace, "<", Self::skip_whitespace)
            .void()
            .parse_next(input)
    }

    fn try_parse_gt(input: &mut &str) -> PResult<()> {
        (Self::skip_whitespace, ">", Self::skip_whitespace)
            .void()
            .parse_next(input)
    }
}

/// 便利函数：解析路径表达式
pub fn parse_path_expression(input: &str) -> ParseResult<PathExpression> {
    ExpressionParser::parse_path_expression(input)
}
