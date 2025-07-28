use super::operators::{ComparisonOp, LogicalOp};
use crate::parser::path::PathSegment;
use serde_json::Value;

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

    /// 函数调用: function_name(arg1, arg2, ...)
    FunctionCall {
        name: String,
        args: Vec<PathExpression>,
    },

    /// 条件表达式: if condition then expr1 else expr2 end
    Conditional {
        condition: Box<PathExpression>,
        then_expr: Box<PathExpression>,
        else_expr: Option<Box<PathExpression>>,
    },

    /// 比较操作: left op right
    Comparison {
        left: Box<PathExpression>,
        op: ComparisonOp,
        right: Box<PathExpression>,
    },

    /// 逻辑操作: left op right 或 not expr
    Logical {
        op: LogicalOp,
        operands: Vec<PathExpression>,
    },

    /// try-catch 表达式: try expr catch handler
    TryCatch {
        try_expr: Box<PathExpression>,
        catch_expr: Option<Box<PathExpression>>,
    },

    /// 可选操作符: expr?
    Optional(Box<PathExpression>),
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
    pub fn pipe(left: PathExpression, right: PathExpression) -> PathExpression {
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
    pub fn as_string(&self) -> String {
        match self {
            PathExpression::Segments(segments) => segments
                .iter()
                .map(|s| match s {
                    PathSegment::Field(name) => format!(".{name}"),
                    PathSegment::Index(idx) => format!("[{idx}]"),
                    PathSegment::Wildcard => "*".to_string(),
                    PathSegment::RecursiveWildcard => "**".to_string(),
                    PathSegment::TypeFilter(typ) => format!("| {typ}"),
                })
                .collect::<Vec<_>>()
                .join(""),

            PathExpression::Pipe { left, right } => {
                format!("{} | {}", left.as_string(), right.as_string())
            }

            PathExpression::Comma(exprs) => exprs
                .iter()
                .map(|e| e.as_string())
                .collect::<Vec<_>>()
                .join(", "),

            PathExpression::Literal(value) => serde_json::to_string(value)
                .unwrap_or_else(|_| "null".to_string()),

            PathExpression::Identity => ".".to_string(),

            PathExpression::FunctionCall { name, args } => {
                if args.is_empty() {
                    format!("{name}()")
                } else {
                    let arg_strings: Vec<String> =
                        args.iter().map(|arg| arg.as_string()).collect();
                    format!("{}({})", name, arg_strings.join(", "))
                }
            }

            PathExpression::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                if let Some(else_expr) = else_expr {
                    format!(
                        "if {} then {} else {} end",
                        condition.as_string(),
                        then_expr.as_string(),
                        else_expr.as_string()
                    )
                } else {
                    format!(
                        "if {} then {} end",
                        condition.as_string(),
                        then_expr.as_string()
                    )
                }
            }

            PathExpression::Comparison { left, op, right } => {
                let op_str = match op {
                    ComparisonOp::Equal => "==",
                    ComparisonOp::NotEqual => "!=",
                    ComparisonOp::LessThan => "<",
                    ComparisonOp::LessThanOrEqual => "<=",
                    ComparisonOp::GreaterThan => ">",
                    ComparisonOp::GreaterThanOrEqual => ">=",
                };
                format!("{} {} {}", left.as_string(), op_str, right.as_string())
            }

            PathExpression::Logical { op, operands } => match op {
                LogicalOp::And => operands
                    .iter()
                    .map(|e| e.as_string())
                    .collect::<Vec<_>>()
                    .join(" and "),
                LogicalOp::Or => operands
                    .iter()
                    .map(|e| e.as_string())
                    .collect::<Vec<_>>()
                    .join(" or "),
                LogicalOp::Not => {
                    if operands.len() == 1 {
                        format!("not {}", operands[0].as_string())
                    } else {
                        "not (invalid)".to_string()
                    }
                }
            },

            PathExpression::TryCatch {
                try_expr,
                catch_expr,
            } => {
                if let Some(catch_expr) = catch_expr {
                    format!(
                        "try {} catch {}",
                        try_expr.as_string(),
                        catch_expr.as_string()
                    )
                } else {
                    format!("try {}", try_expr.as_string())
                }
            }

            PathExpression::Optional(expr) => {
                format!("{}?", expr.as_string())
            }
        }
    }
}

impl std::fmt::Display for PathExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}
