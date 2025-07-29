# 🚀 XQPath 调试维测能力实施快速指南

## 📋 立即开始 v1.4.1

### 第一步：环境准备 (15 分钟)

```bash
# 1. 更新项目依赖
cd /Users/cal/Downloads/datapath-template

# 2. 编辑 Cargo.toml，添加调试相关依赖
cat >> Cargo.toml << 'EOF'

# 调试和监控依赖
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json", "fmt"] }
tracing-appender = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
once_cell = "1.19"
chrono = { version = "0.4", features = ["serde"] }

# 性能监控 (为 v1.4.2 准备)
criterion = { version = "0.5", features = ["html_reports"], optional = true }
sysinfo = { version = "0.30", optional = true }

[features]
default = []
debug = ["tracing", "tracing-subscriber", "tracing-appender"]
performance = ["criterion", "sysinfo", "debug"]
full = ["debug", "performance"]
EOF

# 3. 更新项目
cargo update
cargo check --features debug
```

### 第二步：创建基础结构 (30 分钟)

```bash
# 创建新的模块目录
mkdir -p src/{logging,debug,config,monitoring}

# 创建日志模块
cat > src/logging/mod.rs << 'EOF'
//! 结构化日志模块
//!
//! 提供统一的日志记录接口，支持多种输出格式和目标

use tracing_subscriber::{EnvFilter, FmtSubscriber};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use std::path::Path;

pub mod config;
pub mod formatter;

pub use config::LogConfig;

/// 初始化日志系统
pub fn init_logging(config: &LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.level))?;

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_target(config.show_target)
        .with_line_number(config.show_line_number)
        .with_file(config.show_file);

    match &config.output_file {
        Some(file_path) => {
            let file_appender = RollingFileAppender::new(
                Rotation::daily(),
                Path::new(file_path).parent().unwrap_or(Path::new(".")),
                Path::new(file_path).file_name().unwrap_or("xqpath.log".as_ref())
            );
            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
            subscriber.with_writer(non_blocking).init();
        }
        None => {
            subscriber.init();
        }
    }

    Ok(())
}
EOF

# 创建日志配置
cat > src/logging/config.rs << 'EOF'
//! 日志配置模块

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// 日志级别: trace, debug, info, warn, error
    pub level: String,
    /// 是否显示目标模块
    pub show_target: bool,
    /// 是否显示行号
    pub show_line_number: bool,
    /// 是否显示文件名
    pub show_file: bool,
    /// 输出文件路径
    pub output_file: Option<String>,
    /// 是否启用彩色输出
    pub colored: bool,
    /// JSON格式输出
    pub json_format: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            show_target: false,
            show_line_number: true,
            show_file: false,
            output_file: None,
            colored: true,
            json_format: false,
        }
    }
}
EOF

# 创建调试模块
cat > src/debug/mod.rs << 'EOF'
//! 调试功能模块
//!
//! 提供详细的调试信息和错误诊断

use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

pub mod diagnostics;
pub mod timing;

pub use diagnostics::ErrorDiagnostics;
pub use timing::ExecutionTimer;

/// 调试上下文信息
#[derive(Debug, Clone)]
pub struct DebugContext {
    pub operation: String,
    pub input_size: usize,
    pub start_time: Instant,
    pub memory_usage: Option<usize>,
}

impl DebugContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            input_size: 0,
            start_time: Instant::now(),
            memory_usage: None,
        }
    }

    pub fn with_input_size(mut self, size: usize) -> Self {
        self.input_size = size;
        self
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn log_completion(&self) {
        info!(
            operation = %self.operation,
            input_size = self.input_size,
            duration_ms = self.elapsed().as_millis(),
            "Operation completed"
        );
    }
}
EOF

# 更新主模块
cat >> src/lib.rs << 'EOF'

#[cfg(feature = "debug")]
pub mod logging;
#[cfg(feature = "debug")]
pub mod debug;

pub use debug::DebugContext;
EOF
```

### 第三步：更新 CLI 接口 (20 分钟)

```bash
# 更新 src/cli.rs，添加调试选项
cat > temp_cli_update.rs << 'EOF'
// 添加到现有的 CLI 结构中

#[derive(Debug, Parser)]
pub struct DebugOptions {
    /// 启用调试模式
    #[arg(long, help = "Enable debug mode with detailed logging")]
    pub debug: bool,

    /// 设置日志级别
    #[arg(long, default_value = "info", help = "Set log level: trace, debug, info, warn, error")]
    pub log_level: String,

    /// 日志输出文件
    #[arg(long, help = "Output logs to file instead of stderr")]
    pub log_file: Option<PathBuf>,

    /// 显示执行时间
    #[arg(long, help = "Show execution timing information")]
    pub timing: bool,

    /// JSON格式输出
    #[arg(long, help = "Output logs in JSON format")]
    pub json_logs: bool,

    /// 显示内存使用情况
    #[arg(long, help = "Show memory usage statistics")]
    pub memory_stats: bool,
}

impl Default for DebugOptions {
    fn default() -> Self {
        Self {
            debug: false,
            log_level: "info".to_string(),
            log_file: None,
            timing: false,
            json_logs: false,
            memory_stats: false,
        }
    }
}
EOF

echo "✅ CLI 更新模板已创建，请手动集成到现有的 src/cli.rs 中"
```

### 第四步：基础测试和验证 (15 分钟)

```bash
# 1. 编译检查
cargo check --features debug

# 2. 运行现有测试
cargo test --features debug

# 3. 创建调试功能的基础测试
cat > tests/debug_tests.rs << 'EOF'
#[cfg(feature = "debug")]
mod debug_integration {
    use datapath::debug::DebugContext;
    use std::time::Duration;

    #[test]
    fn test_debug_context_creation() {
        let ctx = DebugContext::new("test_operation");
        assert_eq!(ctx.operation, "test_operation");
        assert_eq!(ctx.input_size, 0);
    }

    #[test]
    fn test_debug_context_with_input_size() {
        let ctx = DebugContext::new("test_operation").with_input_size(1024);
        assert_eq!(ctx.input_size, 1024);
    }

    #[test]
    fn test_debug_context_timing() {
        let ctx = DebugContext::new("test_operation");
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = ctx.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
    }
}
EOF

# 4. 运行新测试
cargo test debug_tests --features debug
```

## 📝 第一天完整工作清单

### 上午 (4 小时)

- [ ] ✅ 更新 Cargo.toml 依赖
- [ ] ✅ 创建 logging 模块结构
- [ ] ✅ 实现基础的日志配置
- [ ] ✅ 创建 debug 模块框架

### 下午 (4 小时)

- [ ] ⏳ 集成 CLI 调试选项
- [ ] ⏳ 实现 DebugContext 功能
- [ ] ⏳ 编写基础测试用例
- [ ] ⏳ 验证功能正常工作

## 🎯 第一周目标检查

### Day 1-2: 基础设施 ✅

- [x] 项目依赖更新
- [x] 日志系统框架
- [x] 调试模块创建
- [x] CLI 选项扩展

### Day 3-4: 核心功能

- [ ] 完整的错误诊断系统
- [ ] 执行时间统计功能
- [ ] 内存使用监控
- [ ] JSON 格式日志输出

### Day 5-7: 集成和测试

- [ ] 与现有代码集成
- [ ] 全面测试覆盖
- [ ] 性能影响评估
- [ ] 用户文档更新

## 🚨 常见问题解决

### Q1: 编译错误 - tracing 相关

```bash
# 解决方案：确保feature正确启用
cargo build --features debug
```

### Q2: 测试失败

```bash
# 解决方案：检查feature gate
cargo test --features debug --verbose
```

### Q3: 依赖冲突

```bash
# 解决方案：更新所有依赖
cargo update
cargo tree # 检查依赖树
```

## 📞 获取帮助

### 快速调试命令

```bash
# 检查项目状态
cargo check --features debug --verbose

# 查看日志系统工作
RUST_LOG=debug cargo run --features debug -- get '.test' -f test.json

# 运行特定测试
cargo test debug --features debug -- --nocapture
```

### 下一步计划

1. **完成第一天任务后**：继续实现错误诊断系统
2. **第一周完成后**：开始 v1.4.2 性能监控功能
3. **遇到问题时**：参考详细实施文档或创建 GitHub Issue

---

**记住**: 这是一个渐进式的改进过程，每一步都要确保现有功能不受影响。调试功能默认不启用，只有在明确需要时才开启。
