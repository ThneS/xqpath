# 🔧 XQPath 调试维测能力三版本实施计划总览

## 📋 版本规划概述

XQPath 调试和维测能力将分为三个递进版本实施，每个版本都在前一版本基础上构建更高级的功能。**重要**：所有功能将同时支持 Rust 库模式和命令行工具模式，确保双重形态的一致性体验。

## 🗓️ 版本时间线

```
v1.4.1 基础调试能力     v1.4.2 性能监控分析     v1.4.3 运维监控工具
   (2-3周)                 (3-4周)                 (3-4周)
      |                       |                       |
      ├─ 结构化日志系统        ├─ 性能指标收集          ├─ 健康检查系统
      ├─ CLI调试选项          ├─ 实时性能监控          ├─ 配置管理系统
      ├─ 库调试宏/API         ├─ 基准测试套件          ├─ 交互式调试器
      ├─ 错误诊断增强          ├─ 内存管理监控          ├─ 崩溃报告系统
      └─ 基础性能监控          ├─ 性能报告生成          └─ 监控数据导出
                             └─ 库性能Profile API     └─ 库调试Context API
```

## 🎯 各版本核心目标

### v1.4.1: 基础调试能力 🔍

**时间：2-3 周 | 重点：建立双模态调试基础设施**

#### 核心价值

- 🐛 **问题快速定位**：精确的错误信息和修复建议
- 📝 **完整执行记录**：结构化日志追踪程序执行
- ⏱️ **性能感知**：基础的执行时间统计
- 🔄 **双模态支持**：库和 CLI 模式统一调试体验

#### CLI 模式功能

```bash
# 新增调试相关CLI参数
xqpath get '.data' --debug                    # 开启调试模式
xqpath get '.data' --log-level info          # 设置日志级别
xqpath get '.data' --log-file debug.log      # 输出到文件
xqpath get '.data' --timing                  # 显示执行时间
xqpath get '.data' --trace-path              # 路径解析跟踪
xqpath get '.data' --memory-stats            # 内存使用统计

# 新增调试专用命令
xqpath debug '.complex.query' -f data.json   # 调试模式执行
xqpath trace '.path' -f data.json            # 执行路径跟踪
```

#### 库模式功能

```rust
use xqpath::{query_debug, trace_query, DebugContext, TimingStats};

// 调试宏 - 带详细执行信息
let result = query_debug!(data, ".users[*].name", |debug_info| {
    println!("解析耗时: {:?}", debug_info.parse_duration);
    println!("执行路径: {}", debug_info.execution_path);
})?;

// 性能跟踪宏
let (result, stats) = trace_query!(data, ".complex.nested.path")?;
println!("总耗时: {:?}, 内存分配: {} bytes", stats.duration, stats.memory_used);

// 调试上下文API
let mut ctx = DebugContext::new()
    .with_timing(true)
    .with_memory_tracking(true)
    .with_path_tracing(true);

let result = ctx.query(data, ".users[*].name")?;
println!("调试信息: {:#?}", ctx.get_debug_info());

// 错误诊断增强 - 兼容现有宏
match query_one!(data, ".invalid.path") {
    Err(e) if e.is_path_error() => {
        println!("路径错误: {}", e.get_path_suggestion());
        println!("可能的修复: {:?}", e.get_fix_suggestions());
    }
    _ => {}
}
```

#### 技术栈

- `tracing` - 结构化日志框架
- `tracing-subscriber` - 日志订阅和格式化
- `tracing-appender` - 文件日志输出
- feature gates: `debug`, `tracing`

### v1.4.2: 性能监控分析 📊

**时间：3-4 周 | 重点：全面性能可观测性**

#### 核心价值

- ⚡ **性能瓶颈识别**：精确定位程序热点
- 📈 **性能趋势分析**：历史数据对比和回归检测
- 💾 **资源使用优化**：内存和 CPU 使用监控
- 🔍 **库内嵌性能分析**：零配置的性能监控

#### CLI 模式功能

```bash
# 性能分析相关功能
xqpath get '.data' --profile                 # 启用性能分析
xqpath profile '.complex.query' -f large.json # 专用性能分析命令
xqpath get '.data' --memory-limit 100MB      # 设置资源限制
xqpath benchmark -f test-data/                # 基准测试套件

# 性能报告
xqpath profile '.data' -f big.json --report html --output profile.html
xqpath profile '.data' -f big.json --compare baseline.json

# 监控模式
xqpath monitor '.data' -f streaming.json --interval 1s
```

#### 库模式功能

```rust
use xqpath::{ProfiledQuery, BenchmarkSuite, MemoryProfiler, query_with_profile};

// 性能分析宏
let (result, profile) = query_with_profile!(data, ".complex.path")?;
println!("执行时间: {:?}", profile.execution_time);
println!("内存峰值: {} KB", profile.peak_memory_kb);
println!("路径优化建议: {:?}", profile.optimization_hints);

// 基准测试API
let mut bench = BenchmarkSuite::new();
bench.add_test("simple_path", || query!(data, ".simple"));
bench.add_test("complex_path", || query!(data, ".complex[*].nested"));
let results = bench.run()?;

// 内存分析器
let mut profiler = MemoryProfiler::new();
profiler.start();
let result = query!(data, ".users[*].name")?;
let report = profiler.stop();
println!("内存使用报告: {:#?}", report);

// 性能监控器 - 持续监控
let mut monitor = PerformanceMonitor::new()
    .with_memory_tracking(true)
    .with_cpu_profiling(true)
    .with_gc_monitoring(true);

monitor.start();
// 执行查询操作...
let metrics = monitor.get_current_metrics();
```

#### 技术栈

- `criterion` - 基准测试框架
- `sysinfo` - 系统信息收集
- `pprof` - CPU 性能分析
- HTML 性能报告生成
- feature gates: `profiling`, `benchmark`

### v1.4.3: 运维监控工具 🛠️

**时间：3-4 周 | 重点：生产环境支持**

#### 核心价值

- 🔧 **运维自动化**：健康检查和自动诊断
- 🎯 **交互式调试**：专业的调试环境
- 📊 **监控集成**：支持主流监控系统
- 🔐 **生产就绪**：企业级监控和报警能力

#### CLI 模式功能

```bash
# 运维和监控功能
xqpath health                                 # 系统健康检查
xqpath config set debug.level warn           # 配置管理
xqpath debug -f data.json                    # 交互式调试器
xqpath metrics export --format prometheus    # 监控数据导出
xqpath doctor                                # 自动诊断工具
xqpath watch '.path' -f streaming.json       # 实时监控

# 崩溃恢复和报告
xqpath crash-report --since "1 hour ago"     # 崩溃报告
xqpath recover --backup-file backup.json     # 数据恢复
```

#### 库模式功能

```rust
use xqpath::{HealthChecker, ConfigManager, CrashReporter, DebugSession};

// 健康检查系统
let mut health = HealthChecker::new();
health.add_check("memory_usage", |ctx| {
    ctx.memory_usage() < 1024 * 1024 * 100 // < 100MB
});
health.add_check("query_performance", |ctx| {
    ctx.avg_query_time() < Duration::from_millis(100)
});

let status = health.run_checks();
if !status.is_healthy() {
    println!("系统异常: {:?}", status.failed_checks());
}

// 配置管理
let mut config = ConfigManager::load_from_file("xqpath.toml")?;
config.set("debug.trace_enabled", true)?;
config.set("performance.max_memory_mb", 256)?;
config.save()?;

// 崩溃报告系统
let reporter = CrashReporter::new()
    .with_stacktrace(true)
    .with_system_info(true)
    .with_auto_upload(false);

// 在出错时自动生成报告
if let Err(e) = query!(data, ".complex.path") {
    let report = reporter.generate_report(&e);
    report.save_to_file("crash_report.json")?;
}

// 交互式调试会话
let mut debug_session = DebugSession::new(data);
debug_session.set_breakpoint(".users[*]");
debug_session.run_interactive(); // 启动交互式调试器

// 监控数据导出
let exporter = MetricsExporter::new()
    .with_format(ExportFormat::Prometheus)
    .with_endpoint("http://localhost:9090/metrics");

exporter.export_metrics()?;
```

#### 技术栈

- `serde_yaml` - 配置文件管理
- `crossterm` - 交互式命令行界面
- `tokio` - 异步运行时（用于监控）
- `prometheus` - 监控指标格式
- 崩溃报告和恢复机制
- feature gates: `monitoring`, `interactive-debug`

## 📊 预期收益对比

| 能力维度     | v1.4.1 基础版 | v1.4.2 增强版 | v1.4.3 完整版 |
| ------------ | ------------- | ------------- | ------------- |
| 问题定位速度 | 提升 50%      | 提升 70%      | 提升 85%      |
| 性能优化能力 | 基础统计      | 专业分析      | 全面监控      |
| 运维效率     | 手动调试      | 半自动        | 全自动化      |
| 用户体验     | 改善 30%      | 改善 60%      | 改善 80%      |
| 生产就绪度   | 开发阶段      | 测试阶段      | 生产就绪      |

## � 实施架构设计

### 双模态架构兼容性

#### 1. 共享核心模块 (src/debug/)

```rust
// src/debug/mod.rs - 核心调试基础设施
pub mod logger;      // 结构化日志系统
pub mod profiler;    // 性能分析器
pub mod tracer;      // 执行路径跟踪
pub mod reporter;    // 错误诊断报告
pub mod config;      // 配置管理

// 提供统一的调试接口
pub trait DebugCapable {
    fn enable_debug(&mut self, config: DebugConfig);
    fn get_debug_info(&self) -> DebugInfo;
}
```

#### 2. CLI 模式实现 (src/cli.rs)

```rust
// 扩展现有 CLI 结构
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    // 全局调试选项
    #[arg(long, global = true)]
    pub debug: bool,

    #[arg(long, global = true)]
    pub log_level: Option<LogLevel>,

    #[arg(long, global = true)]
    pub timing: bool,

    #[arg(long, global = true)]
    pub profile: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    // 现有命令保持不变...
    Get(GetCommand),
    Set(SetCommand),
    // ... 其他现有命令

    // 新增调试命令
    Debug(DebugCommand),
    Trace(TraceCommand),
    Profile(ProfileCommand),
    Benchmark(BenchmarkCommand),
    Health(HealthCommand),
    Doctor(DoctorCommand),
}
```

#### 3. 库模式实现 (src/lib.rs)

```rust
// 扩展现有宏系统
pub use crate::macros::{
    // 现有宏保持不变
    query, query_one, exists, count,

    // 新增调试宏
    query_debug, trace_query, query_with_profile,

    // 兼容性包装宏
    query_with_debug, profile_query,
};

// 新增调试 API
pub use crate::debug::{
    DebugContext, TimingStats, ProfileReport,
    HealthChecker, ConfigManager, CrashReporter,
};
```

### 功能特性控制

#### Cargo.toml 配置

```toml
[features]
default = ["json", "yaml"]

# 核心功能
json = ["serde_json"]
yaml = ["serde_yaml"]
update = ["serde_json/preserve_order"]

# 调试功能 (新增)
debug = ["tracing", "tracing-subscriber"]
profiling = ["debug", "criterion", "sysinfo", "pprof"]
monitoring = ["profiling", "tokio", "prometheus"]
interactive-debug = ["monitoring", "crossterm", "rustyline"]

# CLI 功能
cli = ["clap", "colored", "anyhow"]
cli-debug = ["cli", "debug"]
cli-full = ["cli-debug", "interactive-debug"]
```

## �🚀 实施建议

### 实施策略

1. **渐进式开发**：每个版本都保持向后兼容
2. **双模态同步**：每个功能同时实现 CLI 和库接口
3. **持续测试**：每个功能都有对应的测试用例
4. **文档同步**：功能开发与文档更新同步进行
5. **用户反馈**：每个版本发布后收集用户反馈

### 兼容性保证

#### 现有 API 兼容性

```rust
// 现有宏继续工作，无需修改用户代码
let result = query!(data, ".users[*].name")?;  // ✅ 保持不变
let user = query_one!(data, ".users[0]")?;     // ✅ 保持不变
let exists = exists!(data, ".users")?;         // ✅ 保持不变

// 新增功能通过新的 API 提供
let (result, debug_info) = query_debug!(data, ".users[*].name")?;  // 🆕 新增
```

#### CLI 向后兼容

```bash
# 现有命令保持不变
xqpath get '.users[*].name' -f data.json      # ✅ 继续工作
xqpath set '.version' '"2.0"' -f config.json  # ✅ 继续工作

# 新增选项是可选的
xqpath get '.users[*].name' -f data.json --debug  # 🆕 可选调试
```

### 风险控制

- ⚠️ **性能影响**：调试功能默认关闭，通过 feature gates 控制
- 🔒 **数据安全**：日志和监控数据不包含敏感信息
- 📦 **依赖管理**：新增依赖通过 feature gates 控制，保持核心轻量
- 🔄 **向后兼容**：现有 API 和 CLI 接口保持不变

### 成功指标

- ✅ **功能完整性**：所有计划功能按时交付，支持双模态
- ✅ **性能基准**：调试功能开启时性能损失<10%
- ✅ **兼容性**：现有代码无需修改即可使用
- ✅ **用户满意度**：通过 GitHub Issue 和用户反馈评估
- ✅ **代码质量**：所有测试通过，代码覆盖率>90%

## 📚 相关文档

- [v1.4.1 详细实施计划](./debug-plan-v1.4.1.md)
- [v1.4.2 详细实施计划](./debug-plan-v1.4.2.md)
- [v1.4.3 详细实施计划](./debug-plan-v1.4.3.md)

---

这个计划将把 XQPath 从一个功能性工具升级为一个具备企业级调试和监控能力的专业工具。
