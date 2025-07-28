pub mod error;
pub mod evaluator;

pub use error::EvaluationError;
pub use evaluator::{evaluate_path_expression, ExpressionEvaluator};
