# XQPath v1.1 实现计划

**文档类型**: 实现计划
**版本**: v1.1
**创建日期**: 2024 年
**状态**: 已完成
**目标**: 实现管道与多路输出功能

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

    /// 恒等表达式 .
    Identity,
}
```

### 2. 解析器实现

#### 操作符优先级

1. **逗号操作符 `,`** - 最低优先级
2. **管道操作符 `|`** - 中等优先级
3. **基础表达式** - 最高优先级

#### 解析器结构

```rust
pub struct ExpressionParser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> ExpressionParser<'a> {
    // 顶层：解析逗号表达式
    fn parse_comma_expression() -> Result<PathExpression>

    // 中层：解析管道表达式
    fn parse_pipe_expression() -> Result<PathExpression>

    // 底层：解析基础表达式
    fn parse_primary_expression() -> Result<PathExpression>
}
```

### 3. 求值器实现

```rust
pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    pub fn evaluate(
        &self,
        expr: &PathExpression,
        data: &Value
    ) -> Result<Vec<Value>, EvaluationError> {
        match expr {
            PathExpression::Segments(segments) => {
                // 重用现有路径提取逻辑
                self.evaluate_segments(segments, data)
            }
            PathExpression::Pipe { left, right } => {
                // 管道求值：left 的输出作为 right 的输入
                self.evaluate_pipe(left, right, data)
            }
            PathExpression::Comma(expressions) => {
                // 逗号求值：收集所有表达式的输出
                self.evaluate_comma(expressions, data)
            }
            PathExpression::Literal(value) => {
                // 字面量直接返回
                Ok(vec![value.clone()])
            }
            PathExpression::Identity => {
                // 恒等表达式返回输入
                Ok(vec![data.clone()])
            }
        }
    }
}
```

## 🔧 实现步骤

### Phase 1: AST 和基础结构 (Week 1)

- [x] 创建 `PathExpression` 枚举
- [x] 实现 AST 节点的 Debug、Clone、PartialEq
- [x] 添加复杂度分析方法
- [x] 实现表达式字符串化

### Phase 2: 解析器实现 (Week 2)

- [x] 实现 `ExpressionParser` 结构体
- [x] 递归下降解析器实现
- [x] 操作符优先级处理
- [x] 错误处理和位置跟踪
- [x] 字面量解析支持

### Phase 3: 求值器实现 (Week 3)

- [x] 实现 `ExpressionEvaluator` 结构体
- [x] 管道操作求值逻辑
- [x] 逗号操作求值逻辑
- [x] 与现有路径提取器集成
- [x] 错误类型定义和处理

### Phase 4: 集成和测试 (Week 4)

- [x] API 集成和向后兼容性
- [x] 单元测试编写
- [x] 集成测试验证
- [x] 示例程序和文档
- [x] 性能测试和优化

## 🧪 测试策略

### 解析器测试

- 基本管道表达式解析
- 复杂嵌套表达式解析
- 逗号操作符解析
- 错误情况处理
- 操作符优先级验证

### 求值器测试

- 管道操作求值
- 逗号操作求值
- 字面量和恒等表达式
- 错误传播和处理
- 复杂表达式组合

### 集成测试

- 与现有功能兼容性
- API 向后兼容性
- 性能回归测试
- 实际用例验证

## 📊 预期成果

### 新增功能

- ✅ 管道操作符 `|` 支持
- ✅ 逗号操作符 `,` 支持
- ✅ 字面量值支持
- ✅ 恒等表达式 `.` 支持
- ✅ 复杂表达式组合

### 质量指标

- ✅ 100% 向后兼容性
- ✅ 完整测试覆盖（81 个测试用例）
- ✅ 零性能回归
- ✅ 详细错误信息

### API 扩展

```rust
// 新增 API
pub fn parse_path_expression(input: &str) -> Result<PathExpression, ParseError>;
pub fn evaluate_path_expression(
    expr: &PathExpression,
    data: &Value
) -> Result<Vec<Value>, EvaluationError>;

// 向后兼容 API 保持不变
pub fn parse_path(input: &str) -> Result<Vec<PathSegment>, ParseError>;
pub fn extract_path(segments: &[PathSegment], data: &Value) -> Vec<Value>;
```

## 🎯 成功指标

1. **功能完整性**：实现所有计划的管道和逗号操作符功能
2. **向后兼容性**：所有现有测试用例继续通过
3. **测试覆盖率**：新功能达到 100% 测试覆盖
4. **性能要求**：新功能不影响现有功能性能
5. **文档完整性**：完善的 API 文档和使用示例

## 🚀 后续发展

v1.1 完成后，为后续版本奠定基础：

- **v1.2**: 内置函数系统（length、select、map 等）
- **v1.3**: 变量绑定和用户自定义函数
- **v1.4**: 模块系统和高级特性

---

**状态**: ✅ 已完成
**完成时间**: 2024 年
**测试通过率**: 100% (81/81)

## 📝 相关文档

- [v1.1 进度报告](../releases/v1.1-progress-report.md)
- [RFC-002: v1.1 表达式系统设计](../rfcs/RFC-002-expression-system.md)
