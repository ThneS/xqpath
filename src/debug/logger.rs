//! 结构化日志系统

#[cfg(feature = "debug")]
use tracing::{debug, error, info, trace, warn};

#[cfg(feature = "debug")]
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
    EnvFilter, Layer,
};

#[cfg(feature = "debug")]
use tracing_appender::rolling::{RollingFileAppender, Rotation};

use super::{DebugConfig, LogLevel};

/// 日志器配置
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    pub level: LogLevel,
    pub file_path: Option<String>,
    pub console_enabled: bool,
    pub json_format: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            file_path: None,
            console_enabled: true,
            json_format: false,
        }
    }
}

/// 日志管理器
pub struct Logger {
    config: LoggerConfig,
    #[cfg(feature = "debug")]
    _guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}

impl Logger {
    pub fn new(config: LoggerConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "debug")]
            _guard: None,
        }
    }

    /// 初始化日志系统
    #[cfg(feature = "debug")]
    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let level_filter = match self.config.level {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        };

        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(level_filter));

        let mut layers = Vec::new();

        // 控制台输出层
        if self.config.console_enabled {
            let console_layer = tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_span_events(FmtSpan::CLOSE);

            layers.push(console_layer.boxed());
        }

        // 文件输出层
        if let Some(ref file_path) = self.config.file_path {
            let file_appender =
                RollingFileAppender::new(Rotation::DAILY, "logs", file_path);
            let (non_blocking, guard) =
                tracing_appender::non_blocking(file_appender);
            self._guard = Some(guard);

            let file_layer = tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false);

            layers.push(file_layer.boxed());
        }

        tracing_subscriber::registry()
            .with(env_filter)
            .with(layers)
            .init();

        Ok(())
    }

    #[cfg(not(feature = "debug"))]
    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 无操作实现，当没有 tracing feature 时
        Ok(())
    }

    /// 记录调试信息
    pub fn log_debug(&self, message: &str) {
        #[cfg(feature = "debug")]
        debug!("{}", message);

        #[cfg(not(feature = "debug"))]
        if self.config.console_enabled {
            eprintln!("[DEBUG] {}", message);
        }
    }

    /// 记录信息
    pub fn log_info(&self, message: &str) {
        #[cfg(feature = "debug")]
        info!("{}", message);

        #[cfg(not(feature = "debug"))]
        if self.config.console_enabled {
            println!("[INFO] {}", message);
        }
    }

    /// 记录警告
    pub fn log_warn(&self, message: &str) {
        #[cfg(feature = "debug")]
        warn!("{}", message);

        #[cfg(not(feature = "debug"))]
        if self.config.console_enabled {
            eprintln!("[WARN] {}", message);
        }
    }

    /// 记录错误
    pub fn log_error(&self, message: &str) {
        #[cfg(feature = "debug")]
        error!("{}", message);

        #[cfg(not(feature = "debug"))]
        if self.config.console_enabled {
            eprintln!("[ERROR] {}", message);
        }
    }

    /// 记录执行跟踪
    pub fn log_trace(
        &self,
        path: &str,
        operation: &str,
        duration: std::time::Duration,
    ) {
        let message = format!(
            "Path: {path} | Operation: {operation} | Duration: {duration:?}"
        );

        #[cfg(feature = "debug")]
        trace!("{}", message);

        #[cfg(not(feature = "debug"))]
        if self.config.console_enabled
            && matches!(self.config.level, LogLevel::Trace)
        {
            eprintln!("[TRACE] {}", message);
        }
    }
}

impl From<DebugConfig> for LoggerConfig {
    fn from(config: DebugConfig) -> Self {
        Self {
            level: config.log_level,
            file_path: None,
            console_enabled: true,
            json_format: false,
        }
    }
}

// 从调试日志级别转换为标准日志级别
impl From<LogLevel> for LoggerConfig {
    fn from(level: LogLevel) -> Self {
        Self {
            level,
            file_path: None,
            console_enabled: true,
            json_format: false,
        }
    }
}
