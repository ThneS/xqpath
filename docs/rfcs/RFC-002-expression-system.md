# RFC-002: v1.1 表达式系统设计

**状态**: 已实现 (Implemented)
**作者**: XQPath 项目组
**创建日期**: 2025 年 6 月
**最后更新**: 2025 年 7 月 27 日

## 摘要

本 RFC 定义了 XQPath v1.1 表达式系统的设计，引入管道操作符和逗号操作符，为后续高级功能奠定基础。

## 动机

XQPath v1.0 仅支持简单的路径段访问，缺乏表达式组合能力。为了向 jq 语法靠拢并提供更强大的数据处理能力，需要引入表达式系统。

## 详细设计

### AST 设计

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum PathExpression {
    // 向后兼容的路径段
    Segments(Vec<PathSegment>),

    // 管道操作符
    Pipe {
        left: Box<PathExpression>,
        right: Box<PathExpression>,
    },

    // 逗号操作符
    Comma(Vec<PathExpression>),

    // 字面量值
    Literal(Value),

    // 恒等表达式
    Identity,
}
```

### 操作符优先级

1. **最高优先级**: 基础表达式（字面量、路径段、括号表达式）
2. **中等优先级**: 管道操作符 `|`
3. **最低优先级**: 逗号操作符 `,`

### 解析器设计

采用递归下降解析器，支持正确的操作符优先级：

```rust
impl ExpressionParser {
    // 入口函数，处理最低优先级的逗号操作符
    fn parse_expression(&mut self) -> Result<PathExpression> {
        self.parse_comma_expression()
    }

    // 逗号操作符（最低优先级）
    fn parse_comma_expression(&mut self) -> Result<PathExpression> {
        let mut expressions = vec![self.parse_pipe_expression()?];

        while self.consume_char(',') {
            expressions.push(self.parse_pipe_expression()?);
        }

        if expressions.len() == 1 {
            Ok(expressions.into_iter().next().unwrap())
        } else {
            Ok(PathExpression::Comma(expressions))
        }
    }

    // 管道操作符（中等优先级）
    fn parse_pipe_expression(&mut self) -> Result<PathExpression> {
        let mut left = self.parse_primary_expression()?;

        while self.consume_char('|') {
            let right = self.parse_primary_expression()?;
            left = PathExpression::Pipe {
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // 基础表达式（最高优先级）
    fn parse_primary_expression(&mut self) -> Result<PathExpression> {
        // 处理括号、字面量、路径段等
    }
}
```

### 求值器设计

```rust
impl ExpressionEvaluator {
    fn evaluate_expression(&self, expr: &PathExpression, input: &Value)
        -> Result<Vec<Value>, EvaluationError>
    {
        match expr {
            PathExpression::Segments(segments) => {
                // 重用现有的路径段求值逻辑
                self.evaluate_segments(segments, input)
            }

            PathExpression::Pipe { left, right } => {
                // 管道操作：左侧结果作为右侧输入
                let left_results = self.evaluate_expression(left, input)?;
                let mut final_results = Vec::new();

                for intermediate in left_results {
                    let right_results = self.evaluate_expression(right, &intermediate)?;
                    final_results.extend(right_results);
                }

                Ok(final_results)
            }

            PathExpression::Comma(expressions) => {
                // 逗号操作：收集所有表达式的结果
                let mut all_results = Vec::new();

                for expr in expressions {
                    let results = self.evaluate_expression(expr, input)?;
                    all_results.extend(results);
                }

                Ok(all_results)
            }

            PathExpression::Literal(value) => {
                // 字面量直接返回
                Ok(vec![value.clone()])
            }

            PathExpression::Identity => {
                // 恒等表达式返回输入
                Ok(vec![input.clone()])
            }
        }
    }
}
```

### 错误处理

```rust
#[derive(Debug, thiserror::Error)]
pub enum EvaluationError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Path not found: {0}")]
    PathNotFound(String),

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("Index out of bounds: {index}")]
    IndexOutOfBounds { index: usize },
}
```

## 实现计划

### Week 1: AST 设计

- [x] 定义 PathExpression 枚举
- [x] 实现 Display 和 Debug trait
- [x] 添加复杂度分析方法

### Week 2: 解析器实现

- [x] 实现递归下降解析器
- [x] 支持正确的操作符优先级
- [x] 处理括号表达式和错误恢复

### Week 3: 求值器实现

- [x] 实现表达式求值逻辑
- [x] 集成现有的路径段求值器
- [x] 错误处理和边界情况

### Week 4: 测试和优化

- [x] 单元测试和集成测试
- [x] 性能基准测试
- [x] 文档和示例

## 测试策略

### 单元测试

```rust
#[test]
fn test_pipe_operator() {
    let expr = parse_path_expression(".users | [0] | .name").unwrap();
    let data = json!({"users": [{"name": "Alice"}]});
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!("Alice")]);
}

#[test]
fn test_comma_operator() {
    let expr = parse_path_expression(".name, .age").unwrap();
    let data = json!({"name": "Alice", "age": 30});
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!("Alice"), json!(30)]);
}
```

### 性能测试

```rust
#[bench]
fn bench_complex_expression(b: &mut Bencher) {
    let expr = parse_path_expression("(.users | [*] | .name), \"summary\"").unwrap();
    let data = generate_test_data(1000);

    b.iter(|| {
        evaluate_path_expression(&expr, &data)
    });
}
```

## 向后兼容性

### API 兼容性

- 保持现有 `parse_path()` 函数不变
- 新增 `parse_path_expression()` 函数
- 现有路径语法完全兼容

### 性能影响

- 简单路径访问性能无影响
- 复杂表达式性能优于多次调用
- 内存使用略有增加（<5%）

## 成功指标

### 功能指标

- [x] 支持管道和逗号操作符
- [x] 正确的操作符优先级
- [x] 完整的错误处理
- [x] 100% 向后兼容

### 性能指标

- [x] 解析性能: <1ms for typical expressions
- [x] 求值性能: 与 v1.0 相当或更好
- [x] 内存开销: <5% 增长

### 质量指标

- [x] 测试覆盖率: >95%
- [x] 文档完整性: 100%
- [x] 零回归问题

## 未解决的问题

### 设计决策

1. **类型过滤器优先级**: 在管道操作符存在时，如何处理类型过滤器？
2. **错误传播**: 管道中间环节出错时的行为定义
3. **性能优化**: 是否需要表达式优化（如常量折叠）？

### 后续扩展

1. **更多操作符**: 如何扩展支持算术操作符？
2. **函数调用**: 如何集成函数调用语法？
3. **变量绑定**: 如何为变量系统预留设计空间？

## 参考资料

- [winnow 解析器文档](https://docs.rs/winnow/)
- [jq 表达式语法](https://stedolan.github.io/jq/manual/#pipe)
- [Rust 递归下降解析器模式](https://craftinginterpreters.com/parsing-expressions.html)

---

**变更历史**:

- 2025 年 7 月 27 日: 标记为已实现状态
- 2025 年 6 月: 初始设计版本
