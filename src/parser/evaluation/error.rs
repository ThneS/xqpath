/// 求值错误类型
#[derive(Debug, Clone)]
pub enum EvaluationError {
    /// 一般性错误消息
    Message(String),
    /// 无效参数错误
    InvalidArguments(String),
    /// 未知函数错误
    UnknownFunction(String),
    /// 类型错误
    TypeError { expected: String, actual: String },
    /// 索引越界错误
    IndexOutOfBounds { index: i64, length: usize },
    /// 字段不存在错误
    FieldNotFound(String),
    /// 语法错误
    SyntaxError(String),
    /// 条件求值错误
    ConditionError(String),
    /// try-catch 表达式中的被捕获错误
    CaughtError(Box<EvaluationError>),
}

impl EvaluationError {
    /// 创建一般性错误
    pub fn new(message: String) -> Self {
        Self::Message(message)
    }
}

impl std::fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluationError::Message(msg) => {
                write!(f, "Evaluation error: {msg}")
            }
            EvaluationError::InvalidArguments(msg) => {
                write!(f, "Invalid arguments: {msg}")
            }
            EvaluationError::UnknownFunction(name) => {
                write!(f, "Unknown function: {name}")
            }
            EvaluationError::TypeError { expected, actual } => {
                write!(f, "Type error: expected {expected}, got {actual}")
            }
            EvaluationError::IndexOutOfBounds { index, length } => {
                write!(f, "Index out of bounds: index {index}, length {length}")
            }
            EvaluationError::FieldNotFound(field) => {
                write!(f, "Field not found: {field}")
            }
            EvaluationError::SyntaxError(msg) => {
                write!(f, "Syntax error: {msg}")
            }
            EvaluationError::ConditionError(msg) => {
                write!(f, "Condition error: {msg}")
            }
            EvaluationError::CaughtError(inner) => {
                write!(f, "Caught error: {inner}")
            }
        }
    }
}

impl std::error::Error for EvaluationError {}
