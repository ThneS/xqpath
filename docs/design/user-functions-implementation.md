# XQPath v1.3 用户自定义函数实现计划

**文档类型**: 设计计划
**创建日期**: 2024-07-27
**负责人**: XQPath 开发团队
**状态**: 规划中
**优先级**: 高（v1.3 首要功能）

## 🎯 概述

将用户自定义函数作为 v1.3 的**首要功能**优先开发，为后续的变量系统和模块系统奠定基础。

## 📋 详细实现计划

### Week 1: AST 设计和基础架构

#### 1.1 扩展 PathExpression 枚举

```rust
// src/parser/expression.rs
#[derive(Debug, Clone, PartialEq)]
pub enum PathExpression {
    // ...existing variants...
    FunctionDefinition {
        name: String,
        parameters: Vec<String>,
        body: Box<PathExpression>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<PathExpression>,
    },
}
```

#### 1.2 函数注册表扩展

```rust
// src/functions/user_defined.rs
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct UserDefinedFunction {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: PathExpression,
}

#[derive(Debug, Default)]
pub struct UserFunctionRegistry {
    functions: HashMap<String, UserDefinedFunction>,
}

impl UserFunctionRegistry {
    pub fn register(&mut self, func: UserDefinedFunction) -> Result<(), String> {
        if self.functions.contains_key(&func.name) {
            return Err(format!("Function '{}' already defined", func.name));
        }
        self.functions.insert(func.name.clone(), func);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&UserDefinedFunction> {
        self.functions.get(name)
    }

    pub fn list_functions(&self) -> Vec<&str> {
        self.functions.keys().map(|s| s.as_str()).collect()
    }
}
```

#### 1.3 求值器上下文扩展

```rust
// src/evaluator.rs
#[derive(Debug, Default)]
pub struct EvaluationContext {
    pub user_functions: UserFunctionRegistry,
    pub call_stack: Vec<String>, // 用于递归检测
    pub max_recursion_depth: usize,
}

impl EvaluationContext {
    pub fn new() -> Self {
        Self {
            user_functions: UserFunctionRegistry::default(),
            call_stack: Vec::new(),
            max_recursion_depth: 1000, // 默认最大递归深度
        }
    }
}
```

### Week 2: 解析器实现

#### 2.1 函数定义解析

```rust
// src/parser/expression.rs
impl ExpressionParser {
    fn parse_function_definition(&mut self) -> Result<PathExpression, ParseError> {
        // 解析 "def function_name(param1; param2): body;"
        self.expect_keyword("def")?;

        let name = self.parse_identifier()?;

        // 解析参数列表
        self.expect_char('(')?;
        let mut parameters = Vec::new();

        while !self.check_char(')') {
            parameters.push(self.parse_identifier()?);
            if self.check_char(';') {
                self.advance(); // 消费 ';'
            } else if !self.check_char(')') {
                return Err(ParseError::Expected("';' or ')'".to_string()));
            }
        }

        self.expect_char(')')?;
        self.expect_char(':')?;

        // 解析函数体
        let body = Box::new(self.parse_pipe_expression()?);

        self.expect_char(';')?;

        Ok(PathExpression::FunctionDefinition {
            name,
            parameters,
            body,
        })
    }

    fn parse_function_call(&mut self, name: String) -> Result<PathExpression, ParseError> {
        // 解析 "function_name(arg1; arg2)"
        self.expect_char('(')?;
        let mut arguments = Vec::new();

        while !self.check_char(')') {
            arguments.push(self.parse_pipe_expression()?);
            if self.check_char(';') {
                self.advance();
            } else if !self.check_char(')') {
                return Err(ParseError::Expected("';' or ')'".to_string()));
            }
        }

        self.expect_char(')')?;

        Ok(PathExpression::FunctionCall { name, arguments })
    }
}
```

#### 2.2 集成到主解析流程

```rust
impl ExpressionParser {
    fn parse_primary_expression(&mut self) -> Result<PathExpression, ParseError> {
        // ... existing code ...

        // 检查是否是函数定义
        if self.check_keyword("def") {
            return self.parse_function_definition();
        }

        // 检查是否是标识符（可能是函数调用）
        if let Some(identifier) = self.try_parse_identifier() {
            if self.check_char('(') {
                // 函数调用
                return self.parse_function_call(identifier);
            } else {
                // 其他标识符处理
                return Err(ParseError::UnexpectedIdentifier(identifier));
            }
        }

        // ... existing code ...
    }
}
```

### Week 3: 求值器实现

#### 3.1 函数定义求值

```rust
// src/evaluator.rs
impl ExpressionEvaluator {
    fn evaluate_function_definition(
        &self,
        name: &str,
        parameters: &[String],
        body: &PathExpression,
        context: &mut EvaluationContext,
        _input: &Value,
    ) -> Result<Vec<Value>, EvaluationError> {
        // 注册函数到上下文
        let user_func = UserDefinedFunction {
            name: name.to_string(),
            parameters: parameters.to_vec(),
            body: body.clone(),
        };

        context.user_functions.register(user_func)
            .map_err(|e| EvaluationError::FunctionDefinitionError(e))?;

        // 函数定义本身不产生输出
        Ok(vec![])
    }

    fn evaluate_function_call(
        &self,
        name: &str,
        arguments: &[PathExpression],
        context: &mut EvaluationContext,
        input: &Value,
    ) -> Result<Vec<Value>, EvaluationError> {
        // 递归深度检查
        if context.call_stack.len() >= context.max_recursion_depth {
            return Err(EvaluationError::RecursionLimitExceeded(
                context.max_recursion_depth
            ));
        }

        // 查找函数定义
        let func = context.user_functions.get(name)
            .ok_or_else(|| EvaluationError::UndefinedFunction(name.to_string()))?
            .clone();

        // 参数数量检查
        if arguments.len() != func.parameters.len() {
            return Err(EvaluationError::ArgumentCountMismatch {
                expected: func.parameters.len(),
                actual: arguments.len(),
            });
        }

        // 求值参数
        let mut arg_values = Vec::new();
        for arg in arguments {
            let values = self.evaluate_expression(arg, context, input)?;
            // 简化：每个参数取第一个值
            arg_values.push(values.into_iter().next().unwrap_or(Value::Null));
        }

        // 创建函数调用上下文
        context.call_stack.push(name.to_string());

        // TODO: 实现参数绑定到局部作用域
        // 现在先简单实现：将参数替换到函数体中
        let result = self.evaluate_expression(&func.body, context, input);

        // 恢复调用栈
        context.call_stack.pop();

        result
    }
}
```

#### 3.2 错误类型扩展

```rust
// src/evaluator.rs
#[derive(Debug, thiserror::Error)]
pub enum EvaluationError {
    // ... existing variants ...

    #[error("Function definition error: {0}")]
    FunctionDefinitionError(String),

    #[error("Undefined function: {0}")]
    UndefinedFunction(String),

    #[error("Argument count mismatch: expected {expected}, got {actual}")]
    ArgumentCountMismatch { expected: usize, actual: usize },

    #[error("Recursion limit exceeded: {0}")]
    RecursionLimitExceeded(usize),
}
```

### Week 4: 测试和优化

#### 4.1 基础功能测试

```rust
// tests/user_defined_functions.rs
use xqpath::{parse_path_expression, evaluate_path_expression_with_context, EvaluationContext};
use serde_json::json;

#[test]
fn test_simple_function_definition_and_call() {
    let expr = parse_path_expression("
        def double(x): x * 2;
        5 | double(.)
    ").expect("Failed to parse expression");

    let mut context = EvaluationContext::new();
    let data = json!(null);
    let result = evaluate_path_expression_with_context(&expr, &mut context, &data)
        .expect("Failed to evaluate expression");

    assert_eq!(result, vec![json!(10)]);
}

#[test]
fn test_recursive_function() {
    let expr = parse_path_expression("
        def factorial: if . <= 1 then 1 else . * ((. - 1) | factorial) end;
        5 | factorial
    ").expect("Failed to parse expression");

    let mut context = EvaluationContext::new();
    let data = json!(null);
    let result = evaluate_path_expression_with_context(&expr, &mut context, &data)
        .expect("Failed to evaluate expression");

    assert_eq!(result, vec![json!(120)]);
}

#[test]
fn test_multiple_parameter_function() {
    let expr = parse_path_expression("
        def add(a; b): a + b;
        add(3; 7)
    ").expect("Failed to parse expression");

    let mut context = EvaluationContext::new();
    let data = json!(null);
    let result = evaluate_path_expression_with_context(&expr, &mut context, &data)
        .expect("Failed to evaluate expression");

    assert_eq!(result, vec![json!(10)]);
}
```

#### 4.2 性能测试

```rust
// benches/user_functions_performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use xqpath::{parse_path_expression, evaluate_path_expression_with_context, EvaluationContext};
use serde_json::json;

fn benchmark_simple_function_call(c: &mut Criterion) {
    let expr = parse_path_expression("
        def double(x): x * 2;
        . | double(.)
    ").expect("Failed to parse expression");

    let mut context = EvaluationContext::new();
    let data = json!(42);

    c.bench_function("simple_function_call", |b| {
        b.iter(|| {
            let mut ctx = context.clone();
            evaluate_path_expression_with_context(
                black_box(&expr),
                black_box(&mut ctx),
                black_box(&data)
            )
        })
    });
}
```

#### 4.3 示例程序

```rust
// examples/user_defined_functions_demo.rs
use xqpath::{parse_path_expression, evaluate_path_expression_with_context, EvaluationContext};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 XQPath v1.3 用户自定义函数演示");
    println!("=" .repeat(50));

    // 示例 1: 简单函数定义和调用
    demo_simple_functions()?;

    // 示例 2: 递归函数
    demo_recursive_functions()?;

    // 示例 3: 数据处理函数
    demo_data_processing_functions()?;

    Ok(())
}

fn demo_simple_functions() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 1. 简单函数定义和调用");
    println!("-" .repeat(30));

    let expr = parse_path_expression("
        def double(x): x * 2;
        def add(a; b): a + b;
        def greet(name): \"Hello, \" + name + \"!\";

        [
            5 | double(.),
            add(3; 7),
            \"World\" | greet(.)
        ]
    ")?;

    let mut context = EvaluationContext::new();
    let data = json!(null);
    let result = evaluate_path_expression_with_context(&expr, &mut context, &data)?;

    println!("表达式: 定义多个简单函数并调用");
    println!("结果: {}", serde_json::to_string_pretty(&result[0])?);

    Ok(())
}

fn demo_recursive_functions() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 2. 递归函数");
    println!("-" .repeat(20));

    let expr = parse_path_expression("
        # 阶乘函数
        def factorial: if . <= 1 then 1 else . * ((. - 1) | factorial) end;

        # 斐波那契函数
        def fibonacci:
            if . <= 1 then .
            else ((. - 1) | fibonacci) + ((. - 2) | fibonacci)
            end;

        # 计算多个值
        {
            factorial_5: 5 | factorial,
            factorial_7: 7 | factorial,
            fibonacci_8: 8 | fibonacci,
            fibonacci_10: 10 | fibonacci
        }
    ")?;

    let mut context = EvaluationContext::new();
    let data = json!(null);
    let result = evaluate_path_expression_with_context(&expr, &mut context, &data)?;

    println!("表达式: 递归计算阶乘和斐波那契数列");
    println!("结果: {}", serde_json::to_string_pretty(&result[0])?);

    Ok(())
}
```

## 🎯 预期成果

完成这 4 周的开发后，XQPath 将支持：

### ✅ 核心功能

- **函数定义**: `def function_name(param1; param2): body;`
- **函数调用**: `function_name(arg1; arg2)`
- **递归支持**: 自动递归深度管理和栈溢出保护
- **多参数**: 支持任意数量的函数参数
- **错误处理**: 完善的函数相关错误报告

### ✅ 高级特性

- **尾递归优化**: 优化尾递归调用的性能
- **函数重载**: 支持相同名称不同参数数量的函数
- **高阶函数**: 函数作为参数传递（基础版本）
- **性能优化**: 函数调用的性能优化

### ✅ 质量保证

- **完整测试**: 单元测试、集成测试、性能测试
- **文档完善**: API 文档、用户指南、示例程序
- **错误处理**: 友好的错误信息和调试支持

## 🚀 后续计划

用户自定义函数完成后，将为后续功能奠定坚实基础：

1. **变量系统**: 函数参数绑定机制可扩展为通用变量系统
2. **模块系统**: 函数定义可以模块化组织和导入
3. **作用域管理**: 函数作用域是通用作用域管理的基础

这种优先开发顺序将确保 XQPath v1.3 能够更快地为用户提供强大的函数式编程能力！

## 📝 相关文档

- [RFC-004: 用户自定义函数系统](../rfcs/RFC-004-user-defined-functions.md)
- [v1.3 开发路线图](../planning/roadmap-v1.3.md)
