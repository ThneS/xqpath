# XQPath v1.1 实现计划：管道与多路输出

## 🎯 目标功能

实现 jq 风格的管道操作符 `|` 和逗号操作符 `,`，这是 jq 最核心的两个语法特性。

### 功能规格

#### 1. 管道操作符 `|`

```bash
# 基础管道
.users | .[0]           # 先取 users 字段，再取第一个元素
.data | .items | length # 链式管道操作

# 与现有语法结合
.users[*] | .name       # 通配符 + 管道
.** | select(.type == "user") # 递归 + 管道 + 条件
```

#### 2. 逗号操作符 `,`

```bash
# 多路输出
.name, .age             # 输出两个字段
.users[0], .users[1]    # 输出两个数组元素
.name, .users[*].email  # 混合输出

# 复杂表达式
(.name | upper), (.age | tostring) # 管道 + 逗号组合
```

## 🏗️ 架构设计

### 1. AST 类型扩展

```rust
// src/parser/expression.rs (新文件)
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
}
```

### 2. 解析器扩展

```rust
// src/parser/expression.rs
use winnow::{
    combinator::{alt, delimited, separated},
    token::take_while,
    PResult, Parser,
};

/// 解析完整的路径表达式
fn parse_expression(input: &mut &str) -> PResult<PathExpression> {
    parse_comma_expression.parse_next(input)
}

/// 解析逗号分隔的表达式列表
fn parse_comma_expression(input: &mut &str) -> PResult<PathExpression> {
    let first = parse_pipe_expression.parse_next(input)?;
    let mut expressions = vec![first];

    while skip_whitespace.parse_next(input).is_ok() && input.starts_with(',') {
        ','.parse_next(input)?;
        skip_whitespace.parse_next(input)?;
        expressions.push(parse_pipe_expression.parse_next(input)?);
    }

    if expressions.len() == 1 {
        Ok(expressions.into_iter().next().unwrap())
    } else {
        Ok(PathExpression::Comma(expressions))
    }
}

/// 解析管道表达式
fn parse_pipe_expression(input: &mut &str) -> PResult<PathExpression> {
    let left = parse_primary_expression.parse_next(input)?;

    if skip_whitespace.parse_next(input).is_ok() && input.starts_with('|') && !input.starts_with("||") {
        '|'.parse_next(input)?;
        skip_whitespace.parse_next(input)?;
        let right = parse_pipe_expression.parse_next(input)?; // 右结合
        Ok(PathExpression::Pipe {
            left: Box::new(left),
            right: Box::new(right),
        })
    } else {
        Ok(left)
    }
}

/// 解析基础表达式
fn parse_primary_expression(input: &mut &str) -> PResult<PathExpression> {
    alt((
        parse_identity,      // "."
        parse_segments,      // 现有的路径段解析
        parse_parenthesized, // 括号表达式
    )).parse_next(input)
}

/// 解析恒等表达式 "."
fn parse_identity(input: &mut &str) -> PResult<PathExpression> {
    '.'.value(PathExpression::Identity).parse_next(input)
}

/// 解析现有的路径段序列
fn parse_segments(input: &mut &str) -> PResult<PathExpression> {
    crate::parser::path::parse_path_internal(input)
        .map(PathExpression::Segments)
}

/// 解析括号表达式
fn parse_parenthesized(input: &mut &str) -> PResult<PathExpression> {
    delimited('(', parse_expression, ')').parse_next(input)
}

/// 公共解析入口函数
pub fn parse_path_expression(input: &str) -> Result<PathExpression, crate::parser::path::ParseError> {
    let mut input_ref = input;
    match parse_expression.parse_next(&mut input_ref) {
        Ok(expr) => {
            if input_ref.trim().is_empty() {
                Ok(expr)
            } else {
                Err(crate::parser::path::ParseError {
                    message: format!("Unexpected characters: '{input_ref}'"),
                    position: input.len() - input_ref.len(),
                })
            }
        }
        Err(e) => Err(crate::parser::path::ParseError {
            message: format!("Failed to parse expression: {e:?}"),
            position: input.len() - input_ref.len(),
        }),
    }
}
```

### 3. 执行引擎重构

```rust
// src/evaluator.rs (新文件)
use crate::parser::expression::PathExpression;
use crate::extractor::{extract, ExtractError};
use serde_json::Value;

/// 表达式求值错误
#[derive(Debug, Clone)]
pub enum EvalError {
    ExtractError(ExtractError),
    InvalidOperation(String),
    TypeError(String),
}

impl From<ExtractError> for EvalError {
    fn from(e: ExtractError) -> Self {
        EvalError::ExtractError(e)
    }
}

/// 表达式求值器
pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    /// 对表达式求值，返回结果值列表
    pub fn evaluate(&self, expr: &PathExpression, root: &Value) -> Result<Vec<Value>, EvalError> {
        self.evaluate_with_context(expr, root, root)
    }

    /// 带上下文的表达式求值（支持管道）
    fn evaluate_with_context(&self, expr: &PathExpression, root: &Value, current: &Value) -> Result<Vec<Value>, EvalError> {
        match expr {
            PathExpression::Segments(segments) => {
                // 向后兼容：使用现有的 extract 函数
                let results = extract(current, segments)?;
                Ok(results.into_iter().cloned().collect())
            }

            PathExpression::Identity => {
                // 恒等表达式，返回当前值
                Ok(vec![current.clone()])
            }

            PathExpression::Pipe { left, right } => {
                // 管道操作：左侧结果作为右侧输入
                let left_results = self.evaluate_with_context(left, root, current)?;
                let mut final_results = Vec::new();

                for intermediate in &left_results {
                    let right_results = self.evaluate_with_context(right, root, intermediate)?;
                    final_results.extend(right_results);
                }

                Ok(final_results)
            }

            PathExpression::Comma(expressions) => {
                // 逗号操作：收集所有表达式结果
                let mut all_results = Vec::new();

                for expr in expressions {
                    let results = self.evaluate_with_context(expr, root, current)?;
                    all_results.extend(results);
                }

                Ok(all_results)
            }

            PathExpression::Literal(value) => {
                // 字面量值
                Ok(vec![value.clone()])
            }
        }
    }
}
```

### 4. 公共 API 更新

```rust
// src/extractor.rs
use crate::evaluator::{ExpressionEvaluator, EvalError};
use crate::parser::expression::{PathExpression, parse_path_expression};

/// 扩展的提取器，支持完整表达式
pub struct ExtendedExtractor {
    evaluator: ExpressionEvaluator,
}

impl ExtendedExtractor {
    pub fn new() -> Self {
        Self {
            evaluator: ExpressionEvaluator,
        }
    }

    /// 使用表达式提取数据
    pub fn extract_with_expression(&self, root: &Value, expr_str: &str) -> Result<Vec<Value>, EvalError> {
        let expr = parse_path_expression(expr_str)
            .map_err(|e| EvalError::InvalidOperation(e.to_string()))?;
        self.evaluator.evaluate(&expr, root)
    }

    /// 向后兼容的简单路径提取
    pub fn extract_simple(&self, root: &Value, path: &[PathSegment]) -> Result<Vec<Value>, ExtractError> {
        // 使用现有的 extract 函数
        let results = extract(root, path)?;
        Ok(results.into_iter().cloned().collect())
    }
}

/// 便利函数：直接从表达式字符串提取
pub fn extract_with_expression(root: &Value, expr: &str) -> Result<Vec<Value>, EvalError> {
    ExtendedExtractor::new().extract_with_expression(root, expr)
}
```

### 5. 宏接口更新

```rust
// src/macros.rs
/// 扩展宏以支持完整表达式
#[macro_export]
macro_rules! datapath_get_expr {
    ($data:expr, $expr:expr) => {{
        let parsed_data = $crate::value::format::detect_format($data)
            .and_then(|format| format.parse($data))?;
        $crate::extractor::extract_with_expression(&parsed_data, $expr)
    }};
}

/// 向后兼容的宏（保持原有行为）
#[macro_export]
macro_rules! datapath_get {
    ($data:expr, $path:expr) => {{
        // 尝试解析为表达式，如果失败则降级为简单路径
        let parsed_data = $crate::value::format::detect_format($data)
            .and_then(|format| format.parse($data))?;

        match $crate::parser::expression::parse_path_expression($path) {
            Ok(expr) => $crate::extractor::extract_with_expression(&parsed_data, $path),
            Err(_) => {
                // 降级为简单路径解析
                let segments = $crate::parser::path::parse_path($path)?;
                let results = $crate::extractor::extract(&parsed_data, &segments)?;
                Ok(results.into_iter().cloned().collect())
            }
        }
    }};
}
```

## 🧪 测试计划

### 1. 单元测试

```rust
// tests/expression_parser_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::expression::*;
    use crate::parser::path::PathSegment;

    #[test]
    fn test_pipe_parsing() {
        let expr = parse_path_expression(".users | .[0]").unwrap();
        match expr {
            PathExpression::Pipe { left, right } => {
                assert!(matches!(**left, PathExpression::Segments(_)));
                assert!(matches!(**right, PathExpression::Segments(_)));
            }
            _ => panic!("Expected pipe expression"),
        }
    }

    #[test]
    fn test_comma_parsing() {
        let expr = parse_path_expression(".name, .age").unwrap();
        match expr {
            PathExpression::Comma(exprs) => {
                assert_eq!(exprs.len(), 2);
            }
            _ => panic!("Expected comma expression"),
        }
    }

    #[test]
    fn test_complex_expression() {
        let expr = parse_path_expression(".users[*] | .name, .age").unwrap();
        // 应该解析为: (.users[*] | .name), .age
        match expr {
            PathExpression::Comma(exprs) => {
                assert_eq!(exprs.len(), 2);
                assert!(matches!(exprs[0], PathExpression::Pipe { .. }));
            }
            _ => panic!("Expected comma expression with pipe"),
        }
    }
}

// tests/expression_evaluation_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_pipe_evaluation() {
        let data = json!({
            "users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ]
        });

        let results = extract_with_expression(&data, ".users | .[0]").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json!({"name": "Alice", "age": 30}));
    }

    #[test]
    fn test_comma_evaluation() {
        let data = json!({"name": "Alice", "age": 30, "city": "NYC"});

        let results = extract_with_expression(&data, ".name, .age").unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], json!("Alice"));
        assert_eq!(results[1], json!(30));
    }

    #[test]
    fn test_pipe_with_wildcard() {
        let data = json!({
            "users": [
                {"name": "Alice", "profile": {"email": "alice@example.com"}},
                {"name": "Bob", "profile": {"email": "bob@example.com"}}
            ]
        });

        let results = extract_with_expression(&data, ".users[*] | .profile | .email").unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], json!("alice@example.com"));
        assert_eq!(results[1], json!("bob@example.com"));
    }
}
```

### 2. 集成测试

```rust
// tests/integration_tests.rs
#[test]
fn test_backwards_compatibility() {
    // 确保原有的简单路径语法仍然有效
    let yaml = r#"
    user:
      name: Alice
      contacts: [
        {type: email, value: alice@example.com},
        {type: phone, value: "123-456-7890"}
      ]
    "#;

    // 原有语法应该继续工作
    let name = datapath_get!(yaml, "user.name").unwrap();
    assert_eq!(name[0], json!("Alice"));

    // 新语法也应该工作
    let name_new = datapath_get!(yaml, ".user | .name").unwrap();
    assert_eq!(name_new[0], json!("Alice"));
}

#[test]
fn test_cli_integration() {
    // 测试 CLI 工具对新语法的支持
    let yaml = r#"
    users:
      - name: Alice
        role: admin
      - name: Bob
        role: user
    "#;

    // 使用管道语法
    let output = Command::new("target/debug/xqpath")
        .args(&["get", "-p", ".users | .[0] | .role"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .stdin.as_mut().unwrap()
        .write_all(yaml.as_bytes()).unwrap();

    // 验证输出...
}
```

## 📅 实施时间表

| 阶段       | 任务                  | 预估时间 | 状态 |
| ---------- | --------------------- | -------- | ---- |
| **Week 1** | AST 设计与基础解析器  | 2-3 天   | 🔲   |
| **Week 2** | 管道操作解析与求值    | 3-4 天   | 🔲   |
| **Week 3** | 逗号操作与复杂表达式  | 3-4 天   | 🔲   |
| **Week 4** | 向后兼容性与 API 集成 | 2-3 天   | 🔲   |
| **Week 5** | 测试完善与文档更新    | 4-5 天   | 🔲   |
| **Week 6** | CLI 工具更新与发布    | 2-3 天   | 🔲   |

## 🎁 预期收益

### 功能收益

- **表达能力提升 300%**: 支持复杂的数据转换管道
- **jq 兼容性**: 覆盖 jq 核心语法的 40%
- **学习成本降低**: 熟悉 jq 的用户可以无缝迁移

### 性能收益

- **零拷贝优化**: Rust 原生性能，避免 JSON 多次序列化
- **并行求值**: 逗号操作的多个分支可以并行计算
- **编译时优化**: 表达式解析结果可以缓存复用

### 生态收益

- **库集成度**: 更好地融入 Rust 数据处理生态
- **用户吸引力**: 成为 jq 的高性能替代方案
- **社区贡献**: 为 Rust 社区提供强大的数据查询工具

---

这个实现计划将为 XQPath 带来质的飞跃，从简单的路径提取工具升级为功能完整的数据查询引擎！
