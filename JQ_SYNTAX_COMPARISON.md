# XQPath vs jq 语法对比分析

## 📊 功能覆盖现状

### 🟢 已支持的语法（XQPath 1.0）

| 语法分类     | XQPath 语法 | jq 等价语法  | 支持状态 | 描述         |
| ------------ | ----------- | ------------ | -------- | ------------ |
| **基础访问** | `.field`    | `.field`     | ✅       | 字段访问     |
| **数组索引** | `[0]`       | `[0]`        | ✅       | 数组元素访问 |
| **通配符**   | `*`         | `.[]`        | ✅       | 迭代所有元素 |
| **递归访问** | `**`        | `..`         | ✅       | 递归下降匹配 |
| **类型过滤** | `\| string` | `\| strings` | ✅       | 基础类型过滤 |

### 🔴 缺失的核心语法（高优先级）

| jq 语法分类    | jq 语法              | 功能描述        | 复杂度  | 优先级 |
| -------------- | -------------------- | --------------- | ------- | ------ |
| **管道操作**   | `\|`                 | 数据流管道      | 🔵 中等 | 🔥 P0  |
| **逗号分隔**   | `,`                  | 多路输出        | 🔵 中等 | 🔥 P0  |
| **数组切片**   | `[start:end]`        | 数组/字符串切片 | 🟡 简单 | 🔥 P0  |
| **条件表达式** | `if-then-else`       | 条件分支        | 🟠 复杂 | 🔥 P0  |
| **比较操作**   | `==`, `!=`, `<`, `>` | 数值/字符串比较 | 🟡 简单 | 🔥 P0  |
| **逻辑操作**   | `and`, `or`, `not`   | 布尔逻辑        | 🟡 简单 | 🔥 P0  |
| **可选操作**   | `.field?`            | 错误抑制        | 🟡 简单 | 🔥 P0  |

### 🟡 中等优先级缺失功能

| jq 语法分组    | 代表语法                | 功能范围      | 优先级 |
| -------------- | ----------------------- | ------------- | ------ |
| **数据构造**   | `{}`, `[]`              | 对象/数组构造 | 🟡 P1  |
| **映射函数**   | `map()`, `select()`     | 数据变换      | 🟡 P1  |
| **聚合函数**   | `add`, `length`, `keys` | 数据统计      | 🟡 P1  |
| **字符串操作** | `split()`, `join()`     | 字符串处理    | 🟡 P1  |
| **赋值操作**   | `=`, `\|=`, `+=`        | 数据更新      | 🟡 P1  |
| **变量绑定**   | `as $var`               | 变量系统      | 🟡 P1  |

### 🔵 低优先级高级功能

| jq 高级特性    | 代表功能            | 复杂度     | 优先级 |
| -------------- | ------------------- | ---------- | ------ |
| **函数定义**   | `def func():`       | 自定义函数 | 🔥 P2  |
| **模块系统**   | `import`, `include` | 代码重用   | 🔥 P2  |
| **正则表达式** | `test()`, `match()` | 文本匹配   | 🔥 P2  |
| **流式处理**   | `foreach`, `reduce` | 数据流控制 | 🔥 P2  |
| **错误处理**   | `try-catch`         | 异常处理   | 🔥 P2  |
| **递归函数**   | 尾递归优化          | 高性能递归 | 🔥 P2  |

## 🛣️ XQPath 发展路线图

### Phase 1: 核心语法兼容 (v1.1-v1.3)

#### v1.1 - 管道与多路输出

```rust
// 新增语法支持
PathExpression::Pipe(Box<PathExpression>, Box<PathExpression>)
PathExpression::Comma(Vec<PathExpression>)

// 示例用法
".users | .name"        // 管道操作
".name, .age"           // 多路输出
".users[].name, .age"   // 混合使用
```

#### v1.2 - 切片与条件

```rust
// 数组切片支持
PathSegment::Slice { start: Option<isize>, end: Option<isize> }

// 条件表达式
PathExpression::Conditional {
    condition: Box<PathExpression>,
    then_expr: Box<PathExpression>,
    else_expr: Option<Box<PathExpression>>,
}

// 示例用法
".[1:3]"                           // 数组切片
"if .age > 18 then .name else null"  // 条件表达式
```

#### v1.3 - 比较与逻辑操作

```rust
// 比较操作符
enum ComparisonOp { Eq, Ne, Lt, Gt, Le, Ge }
PathExpression::Comparison {
    left: Box<PathExpression>,
    op: ComparisonOp,
    right: Box<PathExpression>,
}

// 逻辑操作符
enum LogicalOp { And, Or, Not }
PathExpression::Logical {
    op: LogicalOp,
    operands: Vec<PathExpression>,
}
```

### Phase 2: 数据操作与构造 (v1.4-v1.6)

#### v1.4 - 对象/数组构造

```rust
// 数据构造表达式
PathExpression::ObjectConstruction(Vec<(String, PathExpression)>)
PathExpression::ArrayConstruction(Vec<PathExpression>)

// 示例用法
"{name: .user.name, age: .user.age}"  // 对象构造
"[.name, .age, .email]"              // 数组构造
```

#### v1.5 - 内置函数

```rust
// 内置函数枚举
enum BuiltinFunction {
    Map, Select, Length, Keys, Add, Type,
    Split, Join, Contains, StartsWith, EndsWith,
    // ... 更多函数
}

PathExpression::FunctionCall {
    name: BuiltinFunction,
    args: Vec<PathExpression>,
}
```

#### v1.6 - 变量与赋值

```rust
// 变量系统
PathExpression::Variable(String)
PathExpression::Binding {
    expr: Box<PathExpression>,
    var_name: String,
    in_expr: Box<PathExpression>,
}

// 赋值操作
enum AssignmentOp { Assign, Update, Add, Subtract, /* ... */ }
PathExpression::Assignment {
    target: Box<PathExpression>,
    op: AssignmentOp,
    value: Box<PathExpression>,
}
```

### Phase 3: 高级特性 (v2.0+)

#### v2.0 - 函数定义与模块系统

```rust
// 用户自定义函数
struct FunctionDef {
    name: String,
    params: Vec<String>,
    body: PathExpression,
}

// 模块导入
PathExpression::Import {
    module: String,
    symbols: Vec<String>
}
```

#### v2.1 - 正则表达式与高级字符串

```rust
// 正则表达式支持
PathExpression::RegexMatch { pattern: String, flags: String }
PathExpression::RegexReplace {
    pattern: String,
    replacement: String,
    flags: String,
}
```

## 🏗️ 实现架构调整

### 1. 解析器重构

```rust
// 新的表达式抽象语法树
pub enum PathExpression {
    // 原有的段式路径
    Segments(Vec<PathSegment>),

    // 新增的复合表达式
    Pipe(Box<PathExpression>, Box<PathExpression>),
    Comma(Vec<PathExpression>),
    Conditional { condition: Box<PathExpression>, then_expr: Box<PathExpression>, else_expr: Option<Box<PathExpression>> },
    Comparison { left: Box<PathExpression>, op: ComparisonOp, right: Box<PathExpression> },
    FunctionCall { name: String, args: Vec<PathExpression> },
    Literal(serde_json::Value),

    // ... 其他表达式类型
}
```

### 2. 执行引擎升级

```rust
// 新的执行上下文
pub struct ExecutionContext {
    variables: HashMap<String, serde_json::Value>,
    functions: HashMap<String, FunctionDef>,
    current_value: serde_json::Value,
}

// 表达式求值器
pub trait ExpressionEvaluator {
    fn evaluate(&self, expr: &PathExpression, ctx: &mut ExecutionContext) -> Result<Vec<serde_json::Value>, EvalError>;
}
```

### 3. 类型系统增强

```rust
// 值类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum JqValue {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<JqValue>),
    Object(std::collections::HashMap<String, JqValue>),
}

// 类型检查与转换
pub trait TypeSystem {
    fn type_of(&self, value: &JqValue) -> JqType;
    fn is_truthy(&self, value: &JqValue) -> bool;
    fn compare(&self, left: &JqValue, right: &JqValue) -> std::cmp::Ordering;
}
```

## 📋 实现任务清单

### 🎯 短期任务 (1-2 个月)

- [ ] **解析器扩展**: 支持管道 `|` 操作符解析
- [ ] **逗号操作**: 实现多路输出 `,` 语法
- [ ] **数组切片**: 支持 `[start:end]` 语法解析与执行
- [ ] **可选操作**: 实现 `.field?` 错误抑制语法
- [ ] **比较操作**: 支持 `==`, `!=`, `<`, `>`, `<=`, `>=`
- [ ] **测试覆盖**: 为新功能编写完整测试用例

### 🚀 中期任务 (3-6 个月)

- [ ] **条件表达式**: `if-then-else` 完整实现
- [ ] **逻辑操作**: `and`, `or`, `not` 支持
- [ ] **数据构造**: `{}`, `[]` 对象/数组构造语法
- [ ] **内置函数**: `map`, `select`, `length`, `keys` 等核心函数
- [ ] **变量系统**: `as $var` 变量绑定机制
- [ ] **性能优化**: 针对复杂表达式的执行优化

### 🌟 长期规划 (6 个月+)

- [ ] **函数定义**: `def` 自定义函数语法
- [ ] **模块系统**: `import`/`include` 代码重用机制
- [ ] **正则支持**: `test()`, `match()`, 字符串处理函数
- [ ] **流式处理**: `reduce`, `foreach` 高级控制结构
- [ ] **错误处理**: `try-catch` 异常处理机制
- [ ] **REPL 模式**: 交互式 jq 表达式求值器

## 🎨 语法设计原则

### 1. 向后兼容性

- 现有 XQPath 语法保持 100%兼容
- 渐进式增强，不破坏现有用户代码

### 2. jq 语法对齐

- 优先实现 jq 的核心高频语法
- 语法行为与 jq 保持一致
- 适当简化复杂的边缘情况

### 3. Rust 生态集成

- 充分利用 Rust 类型系统优势
- 与 serde 生态深度集成
- 提供 zero-copy 优化路径

### 4. 性能优先

- 编译时优化表达式
- 避免不必要的数据拷贝
- 支持并行求值（适用场景）

## 🏆 成功指标

| 指标类型       | 目标值                     | 检验方法              |
| -------------- | -------------------------- | --------------------- |
| **语法覆盖率** | 覆盖 jq 核心语法的 80%     | jq 官方测试用例通过率 |
| **性能基准**   | 执行速度达到原生 jq 的 80% | benchmark 对比测试    |
| **兼容性**     | 100% 向后兼容              | 现有测试用例零失败    |
| **用户采用**   | GitHub Stars > 1000        | 社区反馈与采用情况    |

---

通过这个循序渐进的发展路线，XQPath 将从当前的简单路径提取工具，发展成为功能完整、性能优异的 jq 替代方案，同时保持 Rust 生态的原生优势。
