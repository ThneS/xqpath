// Backward compatibility re-exports for expression module
// All functionality has been moved to modularized structure

// Import from the parent modules since they are already properly exported
pub use super::{
    ast::{ComparisonOp, ExpressionComplexity, LogicalOp, PathExpression},
    evaluation::{
        evaluate_path_expression, EvaluationError, ExpressionEvaluator,
    },
    functions::{AdvancedBuiltinFunction, BuiltinFunction, FunctionRegistry},
    parsing::{parse_path_expression, ExpressionParser},
};

// Legacy aliases for maximum compatibility
pub use evaluate_path_expression as evaluate;
pub use parse_path_expression as parse;
