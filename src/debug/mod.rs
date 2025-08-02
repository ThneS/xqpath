//! 调试模块 - XQPath 调试和维测能力
//!
//! 提供统一的调试接口，支持库模式和CLI模式

pub mod logger;
pub mod reporter;
pub mod tracer;

// 未来版本功能模块（预留）
// #[cfg(feature = "profiling")]
// pub mod profiler;

// #[cfg(feature = "monitoring")]
// pub mod config;

#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

/// 调试配置
#[derive(Debug, Clone)]
pub struct DebugConfig {
    pub timing_enabled: bool,
    pub memory_tracking: bool,
    pub path_tracing: bool,
    pub log_level: LogLevel,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            timing_enabled: false,
            memory_tracking: false,
            path_tracing: false,
            log_level: LogLevel::Info,
        }
    }
}

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// 调试信息
#[derive(Debug, Clone, Default)]
pub struct DebugInfo {
    pub parse_duration: Option<Duration>,
    pub execution_duration: Option<Duration>,
    pub execution_path: String,
    pub memory_used: Option<usize>,
    pub queries_executed: usize,
}

/// 性能统计信息
#[derive(Debug, Clone)]
pub struct TimingStats {
    pub duration: Duration,
    pub memory_used: usize,
    pub peak_memory: usize,
}

impl Default for TimingStats {
    fn default() -> Self {
        Self {
            duration: Duration::from_nanos(0),
            memory_used: 0,
            peak_memory: 0,
        }
    }
}

/// 调试上下文
pub struct DebugContext {
    config: DebugConfig,
    debug_info: DebugInfo,
    start_time: Option<Instant>,
}

impl DebugContext {
    pub fn new() -> Self {
        Self {
            config: DebugConfig::default(),
            debug_info: DebugInfo::default(),
            start_time: None,
        }
    }

    pub fn with_timing(mut self, enabled: bool) -> Self {
        self.config.timing_enabled = enabled;
        self
    }

    pub fn with_memory_tracking(mut self, enabled: bool) -> Self {
        self.config.memory_tracking = enabled;
        self
    }

    pub fn with_path_tracing(mut self, enabled: bool) -> Self {
        self.config.path_tracing = enabled;
        self
    }

    pub fn with_log_level(mut self, level: LogLevel) -> Self {
        self.config.log_level = level;
        self
    }

    pub fn start_timing(&mut self) {
        if self.config.timing_enabled {
            self.start_time = Some(Instant::now());
        }
    }

    pub fn stop_timing(&mut self) {
        if let Some(start) = self.start_time.take() {
            self.debug_info.execution_duration = Some(start.elapsed());
        }
    }

    pub fn get_debug_info(&self) -> &DebugInfo {
        &self.debug_info
    }

    pub fn get_config(&self) -> &DebugConfig {
        &self.config
    }
}

impl Default for DebugContext {
    fn default() -> Self {
        Self::new()
    }
}

/// 可调试的 trait
pub trait DebugCapable {
    fn enable_debug(&mut self, config: DebugConfig);
    fn get_debug_info(&self) -> DebugInfo;
}
