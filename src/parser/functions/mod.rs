use crate::parser::{EvaluationError, ExpressionEvaluator, PathExpression};
use serde_json::Value;
use std::collections::HashMap;

pub mod advanced;
pub mod basic;

pub use advanced::*;
pub use basic::*;

/// 内置函数 trait
pub trait BuiltinFunction: Send + Sync {
    /// 函数名称
    fn name(&self) -> &str;

    /// 执行函数
    fn execute(
        &self,
        args: &[Value],
        input: &Value,
    ) -> Result<Vec<Value>, EvaluationError>;

    /// 函数描述
    fn description(&self) -> &str {
        "No description available"
    }
}

/// 高级内置函数 trait（支持表达式参数）
pub trait AdvancedBuiltinFunction: Send + Sync {
    /// 函数名称
    fn name(&self) -> &str;

    /// 执行函数（支持表达式参数）
    fn execute_with_expressions(
        &self,
        args: &[PathExpression],
        evaluator: &ExpressionEvaluator,
        input: &Value,
    ) -> Result<Vec<Value>, EvaluationError>;

    /// 函数描述
    fn description(&self) -> &str {
        "No description available"
    }
}

/// 函数注册表
#[derive(Default)]
pub struct FunctionRegistry {
    functions: HashMap<String, Box<dyn BuiltinFunction>>,
    advanced_functions: HashMap<String, Box<dyn AdvancedBuiltinFunction>>,
}

impl FunctionRegistry {
    /// 创建新的函数注册表
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_builtin_functions();
        registry
    }

    /// 注册函数
    pub fn register(&mut self, function: Box<dyn BuiltinFunction>) {
        self.functions.insert(function.name().to_string(), function);
    }

    /// 注册高级函数
    pub fn register_advanced(
        &mut self,
        function: Box<dyn AdvancedBuiltinFunction>,
    ) {
        self.advanced_functions
            .insert(function.name().to_string(), function);
    }

    /// 获取函数
    pub fn get(&self, name: &str) -> Option<&dyn BuiltinFunction> {
        self.functions.get(name).map(|f| f.as_ref())
    }

    /// 获取高级函数
    pub fn get_advanced(
        &self,
        name: &str,
    ) -> Option<&dyn AdvancedBuiltinFunction> {
        self.advanced_functions.get(name).map(|f| f.as_ref())
    }

    /// 注册内置函数
    fn register_builtin_functions(&mut self) {
        // Phase 1: 基础函数
        self.register(Box::new(LengthFunction));
        self.register(Box::new(TypeFunction));
        self.register(Box::new(KeysFunction));
        self.register(Box::new(ValuesFunction));

        // Phase 3: 高级函数
        self.register_advanced(Box::new(MapFunction));
        self.register_advanced(Box::new(SelectFunction));
        self.register_advanced(Box::new(SortFunction));
        self.register_advanced(Box::new(SortByFunction));
        self.register_advanced(Box::new(GroupByFunction));
        self.register_advanced(Box::new(UniqueFunction));
        self.register_advanced(Box::new(UniqueByFunction));
        self.register_advanced(Box::new(ReverseFunction));
    }
}
