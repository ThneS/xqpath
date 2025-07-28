use super::expression::PathExpression;
use crate::parser::path::PathSegment;

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

    fn analyze_complexity_with_depth(
        &self,
        current_depth: usize,
    ) -> ExpressionComplexity {
        match self {
            PathExpression::Segments(segments) => {
                let has_wildcards =
                    segments.iter().any(|s| matches!(s, PathSegment::Wildcard));
                let has_recursive_wildcards = segments
                    .iter()
                    .any(|s| matches!(s, PathSegment::RecursiveWildcard));

                ExpressionComplexity {
                    depth: current_depth + 1,
                    pipe_count: 0,
                    comma_branches: 1,
                    has_wildcards,
                    has_recursive_wildcards,
                }
            }

            PathExpression::Pipe { left, right } => {
                let left_complexity =
                    left.analyze_complexity_with_depth(current_depth + 1);
                let right_complexity =
                    right.analyze_complexity_with_depth(current_depth + 1);

                ExpressionComplexity {
                    depth: left_complexity.depth.max(right_complexity.depth),
                    pipe_count: left_complexity.pipe_count
                        + right_complexity.pipe_count
                        + 1,
                    comma_branches: left_complexity.comma_branches
                        * right_complexity.comma_branches,
                    has_wildcards: left_complexity.has_wildcards
                        || right_complexity.has_wildcards,
                    has_recursive_wildcards: left_complexity
                        .has_recursive_wildcards
                        || right_complexity.has_recursive_wildcards,
                }
            }

            PathExpression::Comma(exprs) => {
                let mut max_depth = current_depth + 1;
                let mut total_pipe_count = 0;
                let mut total_branches = 0;
                let mut has_wildcards = false;
                let mut has_recursive_wildcards = false;

                for expr in exprs {
                    let complexity =
                        expr.analyze_complexity_with_depth(current_depth + 1);
                    max_depth = max_depth.max(complexity.depth);
                    total_pipe_count += complexity.pipe_count;
                    total_branches += complexity.comma_branches;
                    has_wildcards = has_wildcards || complexity.has_wildcards;
                    has_recursive_wildcards = has_recursive_wildcards
                        || complexity.has_recursive_wildcards;
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

            PathExpression::FunctionCall { args, .. } => {
                let mut max_depth = current_depth + 1;
                let mut total_pipe_count = 0;
                let mut total_branches = 1;
                let mut has_wildcards = false;
                let mut has_recursive_wildcards = false;

                for arg in args {
                    let complexity =
                        arg.analyze_complexity_with_depth(current_depth + 1);
                    max_depth = max_depth.max(complexity.depth);
                    total_pipe_count += complexity.pipe_count;
                    total_branches *= complexity.comma_branches;
                    has_wildcards = has_wildcards || complexity.has_wildcards;
                    has_recursive_wildcards = has_recursive_wildcards
                        || complexity.has_recursive_wildcards;
                }

                ExpressionComplexity {
                    depth: max_depth,
                    pipe_count: total_pipe_count,
                    comma_branches: total_branches,
                    has_wildcards,
                    has_recursive_wildcards,
                }
            }

            PathExpression::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                let condition_complexity =
                    condition.analyze_complexity_with_depth(current_depth + 1);
                let then_complexity =
                    then_expr.analyze_complexity_with_depth(current_depth + 1);

                let else_complexity = if let Some(else_expr) = else_expr {
                    else_expr.analyze_complexity_with_depth(current_depth + 1)
                } else {
                    ExpressionComplexity {
                        depth: current_depth + 1,
                        pipe_count: 0,
                        comma_branches: 1,
                        has_wildcards: false,
                        has_recursive_wildcards: false,
                    }
                };

                ExpressionComplexity {
                    depth: condition_complexity
                        .depth
                        .max(then_complexity.depth)
                        .max(else_complexity.depth),
                    pipe_count: condition_complexity.pipe_count
                        + then_complexity.pipe_count
                        + else_complexity.pipe_count,
                    comma_branches: condition_complexity.comma_branches
                        + then_complexity.comma_branches
                        + else_complexity.comma_branches,
                    has_wildcards: condition_complexity.has_wildcards
                        || then_complexity.has_wildcards
                        || else_complexity.has_wildcards,
                    has_recursive_wildcards: condition_complexity
                        .has_recursive_wildcards
                        || then_complexity.has_recursive_wildcards
                        || else_complexity.has_recursive_wildcards,
                }
            }

            PathExpression::Comparison { left, right, .. } => {
                let left_complexity =
                    left.analyze_complexity_with_depth(current_depth + 1);
                let right_complexity =
                    right.analyze_complexity_with_depth(current_depth + 1);

                ExpressionComplexity {
                    depth: left_complexity.depth.max(right_complexity.depth),
                    pipe_count: left_complexity.pipe_count
                        + right_complexity.pipe_count,
                    comma_branches: left_complexity.comma_branches
                        * right_complexity.comma_branches,
                    has_wildcards: left_complexity.has_wildcards
                        || right_complexity.has_wildcards,
                    has_recursive_wildcards: left_complexity
                        .has_recursive_wildcards
                        || right_complexity.has_recursive_wildcards,
                }
            }

            PathExpression::Logical { operands, .. } => {
                let mut max_depth = current_depth + 1;
                let mut total_pipe_count = 0;
                let mut total_branches = 1;
                let mut has_wildcards = false;
                let mut has_recursive_wildcards = false;

                for operand in operands {
                    let complexity = operand
                        .analyze_complexity_with_depth(current_depth + 1);
                    max_depth = max_depth.max(complexity.depth);
                    total_pipe_count += complexity.pipe_count;
                    total_branches *= complexity.comma_branches;
                    has_wildcards = has_wildcards || complexity.has_wildcards;
                    has_recursive_wildcards = has_recursive_wildcards
                        || complexity.has_recursive_wildcards;
                }

                ExpressionComplexity {
                    depth: max_depth,
                    pipe_count: total_pipe_count,
                    comma_branches: total_branches,
                    has_wildcards,
                    has_recursive_wildcards,
                }
            }

            PathExpression::TryCatch {
                try_expr,
                catch_expr,
            } => {
                let try_complexity =
                    try_expr.analyze_complexity_with_depth(current_depth + 1);
                let catch_complexity = if let Some(catch_expr) = catch_expr {
                    catch_expr.analyze_complexity_with_depth(current_depth + 1)
                } else {
                    ExpressionComplexity {
                        depth: current_depth + 1,
                        pipe_count: 0,
                        comma_branches: 1,
                        has_wildcards: false,
                        has_recursive_wildcards: false,
                    }
                };

                ExpressionComplexity {
                    depth: try_complexity.depth.max(catch_complexity.depth),
                    pipe_count: try_complexity.pipe_count
                        + catch_complexity.pipe_count,
                    comma_branches: try_complexity.comma_branches
                        + catch_complexity.comma_branches,
                    has_wildcards: try_complexity.has_wildcards
                        || catch_complexity.has_wildcards,
                    has_recursive_wildcards: try_complexity
                        .has_recursive_wildcards
                        || catch_complexity.has_recursive_wildcards,
                }
            }

            PathExpression::Optional(expr) => {
                let inner_complexity =
                    expr.analyze_complexity_with_depth(current_depth + 1);
                ExpressionComplexity {
                    depth: inner_complexity.depth,
                    pipe_count: inner_complexity.pipe_count,
                    comma_branches: inner_complexity.comma_branches,
                    has_wildcards: inner_complexity.has_wildcards,
                    has_recursive_wildcards: inner_complexity
                        .has_recursive_wildcards,
                }
            }
        }
    }

    /// 判断表达式是否需要特殊优化
    pub fn needs_optimization(&self) -> bool {
        let complexity = self.analyze_complexity();
        complexity.depth > 5
            || complexity.pipe_count > 3
            || complexity.comma_branches > 10
            || complexity.has_recursive_wildcards
    }
}
