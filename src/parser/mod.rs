pub mod ast;
pub mod evaluation;
pub mod expression;
pub mod functions;
pub mod parsing;
pub mod path;

// Re-export commonly used items for backward compatibility
pub use ast::{ComparisonOp, ExpressionComplexity, LogicalOp, PathExpression};
pub use evaluation::{
    evaluate_path_expression, EvaluationError, ExpressionEvaluator,
};
pub use functions::{
    AdvancedBuiltinFunction, BuiltinFunction, FunctionRegistry,
};
pub use parsing::{parse_path_expression, ExpressionParser};
pub use path::{parse_path, ParseError, ParseResult, PathSegment};
