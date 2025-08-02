//! 性能分析器模块 - v1.4.2
//!
//! 提供全面的性能监控和分析功能

#[cfg(feature = "profiling")]
use std::collections::HashMap;
#[cfg(feature = "profiling")]
use std::time::{Duration, Instant};

/// 性能分析报告
#[cfg(feature = "profiling")]
#[derive(Debug, Clone)]
pub struct ProfileReport {
    /// 执行时间
    pub execution_time: Duration,
    /// 峰值内存使用 (字节)
    pub peak_memory_bytes: usize,
    /// 当前内存使用 (字节)
    pub current_memory_bytes: usize,
    /// CPU 使用率 (百分比)
    pub cpu_usage_percent: f64,
    /// 性能优化建议
    pub optimization_hints: Vec<String>,
    /// 详细性能指标
    pub metrics: HashMap<String, f64>,
}

#[cfg(feature = "profiling")]
impl Default for ProfileReport {
    fn default() -> Self {
        Self {
            execution_time: Duration::from_nanos(0),
            peak_memory_bytes: 0,
            current_memory_bytes: 0,
            cpu_usage_percent: 0.0,
            optimization_hints: Vec::new(),
            metrics: HashMap::new(),
        }
    }
}

#[cfg(feature = "profiling")]
impl ProfileReport {
    /// 添加性能指标
    pub fn add_metric(&mut self, name: impl Into<String>, value: f64) {
        self.metrics.insert(name.into(), value);
    }

    /// 添加优化建议
    pub fn add_hint(&mut self, hint: impl Into<String>) {
        self.optimization_hints.push(hint.into());
    }

    /// 生成性能报告摘要
    pub fn summary(&self) -> String {
        format!(
            "执行时间: {:?}, 峰值内存: {:.2} MB, CPU使用: {:.1}%",
            self.execution_time,
            self.peak_memory_bytes as f64 / 1024.0 / 1024.0,
            self.cpu_usage_percent
        )
    }

    /// 生成HTML格式报告
    pub fn to_html(&self) -> String {
        let mut html = String::new();
        html.push_str(
            "<!DOCTYPE html><html><head><title>XQPath 性能报告</title>",
        );
        html.push_str("<style>body{font-family:Arial,sans-serif;margin:20px;}");
        html.push_str(".metric{background:#f5f5f5;padding:10px;margin:5px 0;border-radius:5px;}");
        html.push_str(".hint{background:#e8f4fd;padding:8px;margin:3px 0;border-left:4px solid #2196F3;}");
        html.push_str("</style></head><body>");

        html.push_str("<h1>XQPath 性能分析报告</h1>");
        html.push_str(&format!(
            "<div class='metric'><strong>执行时间:</strong> {:?}</div>",
            self.execution_time
        ));
        html.push_str(&format!(
            "<div class='metric'><strong>峰值内存:</strong> {:.2} MB</div>",
            self.peak_memory_bytes as f64 / 1024.0 / 1024.0
        ));
        html.push_str(&format!(
            "<div class='metric'><strong>CPU使用率:</strong> {:.1}%</div>",
            self.cpu_usage_percent
        ));

        if !self.metrics.is_empty() {
            html.push_str("<h2>详细指标</h2>");
            for (name, value) in &self.metrics {
                html.push_str(&format!(
                    "<div class='metric'><strong>{name}:</strong> {value:.3}</div>",
                ));
            }
        }

        if !self.optimization_hints.is_empty() {
            html.push_str("<h2>优化建议</h2>");
            for hint in &self.optimization_hints {
                html.push_str(&format!("<div class='hint'>{hint}</div>"));
            }
        }

        html.push_str("</body></html>");
        html
    }
}

/// 性能监控器
#[cfg(feature = "profiling")]
pub struct PerformanceMonitor {
    start_time: Option<Instant>,
    memory_tracker: MemoryTracker,
    cpu_tracker: CpuTracker,
    enabled: bool,
}

#[cfg(feature = "profiling")]
impl PerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new() -> Self {
        Self {
            start_time: None,
            memory_tracker: MemoryTracker::new(),
            cpu_tracker: CpuTracker::new(),
            enabled: true,
        }
    }

    /// 启用/禁用监控
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 开始监控
    pub fn start(&mut self) {
        if !self.enabled {
            return;
        }

        self.start_time = Some(Instant::now());
        self.memory_tracker.start();
        self.cpu_tracker.start();
    }

    /// 停止监控并生成报告
    pub fn stop(&mut self) -> ProfileReport {
        if !self.enabled {
            return ProfileReport::default();
        }

        let execution_time = self
            .start_time
            .map(|start| start.elapsed())
            .unwrap_or_default();

        let memory_stats = self.memory_tracker.stop();
        let cpu_stats = self.cpu_tracker.stop();

        let mut report = ProfileReport {
            execution_time,
            peak_memory_bytes: memory_stats.peak_memory,
            current_memory_bytes: memory_stats.current_memory,
            cpu_usage_percent: cpu_stats.average_usage,
            optimization_hints: Vec::new(),
            metrics: HashMap::new(),
        };

        // 添加性能指标
        report.add_metric("memory_efficiency", memory_stats.efficiency_score());
        report.add_metric("cpu_efficiency", cpu_stats.efficiency_score());

        // 生成优化建议
        self.generate_optimization_hints(&mut report);

        report
    }

    /// 生成优化建议
    fn generate_optimization_hints(&self, report: &mut ProfileReport) {
        // 内存使用建议
        if report.peak_memory_bytes > 100 * 1024 * 1024 {
            // > 100MB
            report.add_hint("考虑使用流式处理减少内存占用");
        }

        // CPU 使用建议
        if report.cpu_usage_percent > 80.0 {
            report.add_hint("查询复杂度较高，考虑简化路径表达式");
        }

        // 执行时间建议
        if report.execution_time > Duration::from_millis(100) {
            report.add_hint("执行时间较长，考虑缓存中间结果");
        }
    }

    /// 获取当前性能指标
    pub fn get_current_metrics(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();

        if let Some(start) = self.start_time {
            metrics.insert(
                "elapsed_ms".to_string(),
                start.elapsed().as_millis() as f64,
            );
        }

        let memory_stats = self.memory_tracker.get_current();
        metrics.insert(
            "current_memory_mb".to_string(),
            memory_stats.current_memory as f64 / 1024.0 / 1024.0,
        );

        let cpu_stats = self.cpu_tracker.get_current();
        metrics
            .insert("cpu_usage_percent".to_string(), cpu_stats.current_usage);

        metrics
    }
}

#[cfg(feature = "profiling")]
impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 内存跟踪器
#[cfg(feature = "profiling")]
struct MemoryTracker {
    start_memory: usize,
    peak_memory: usize,
    current_memory: usize,
}

#[cfg(feature = "profiling")]
impl MemoryTracker {
    fn new() -> Self {
        Self {
            start_memory: 0,
            peak_memory: 0,
            current_memory: 0,
        }
    }

    fn start(&mut self) {
        let current = self.get_memory_usage();
        self.start_memory = current;
        self.current_memory = current;
        self.peak_memory = current;
    }

    fn stop(&mut self) -> MemoryStats {
        self.current_memory = self.get_memory_usage();
        MemoryStats {
            start_memory: self.start_memory,
            peak_memory: self.peak_memory,
            current_memory: self.current_memory,
        }
    }

    fn get_current(&self) -> MemoryStats {
        MemoryStats {
            start_memory: self.start_memory,
            peak_memory: self.peak_memory,
            current_memory: self.get_memory_usage(),
        }
    }

    #[cfg(feature = "profiling")]
    fn get_memory_usage(&self) -> usize {
        // 使用 sysinfo 获取当前进程内存使用
        use sysinfo::{Pid, System};

        let mut sys = System::new();
        let pid = Pid::from(std::process::id() as usize);
        sys.refresh_process(pid);

        if let Some(process) = sys.process(pid) {
            process.memory() as usize * 1024 // 转换为字节
        } else {
            0
        }
    }
}

/// 内存统计信息
#[cfg(feature = "profiling")]
struct MemoryStats {
    start_memory: usize,
    peak_memory: usize,
    current_memory: usize,
}

#[cfg(feature = "profiling")]
impl MemoryStats {
    fn efficiency_score(&self) -> f64 {
        if self.peak_memory == 0 {
            return 100.0;
        }

        let memory_growth =
            self.current_memory.saturating_sub(self.start_memory);

        // 内存效率：使用越少越好
        100.0 - (memory_growth as f64 / (1024.0 * 1024.0)).min(100.0)
    }
}

/// CPU 跟踪器
#[cfg(feature = "profiling")]
struct CpuTracker {
    start_time: Option<Instant>,
    samples: Vec<f64>,
}

#[cfg(feature = "profiling")]
impl CpuTracker {
    fn new() -> Self {
        Self {
            start_time: None,
            samples: Vec::new(),
        }
    }

    fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.samples.clear();
    }

    fn stop(&mut self) -> CpuStats {
        let average_usage = if self.samples.is_empty() {
            0.0
        } else {
            self.samples.iter().sum::<f64>() / self.samples.len() as f64
        };

        CpuStats {
            average_usage,
            peak_usage: self.samples.iter().fold(0.0, |a, &b| a.max(b)),
            current_usage: self.samples.last().copied().unwrap_or(0.0),
        }
    }

    fn get_current(&self) -> CpuStats {
        let current_usage = self.get_cpu_usage();
        CpuStats {
            average_usage: if self.samples.is_empty() {
                0.0
            } else {
                self.samples.iter().sum::<f64>() / self.samples.len() as f64
            },
            peak_usage: self.samples.iter().fold(0.0, |a, &b| a.max(b)),
            current_usage,
        }
    }

    #[cfg(feature = "profiling")]
    fn get_cpu_usage(&self) -> f64 {
        // 简化的CPU使用率计算
        // 在实际实现中，这里会使用更精确的CPU监控
        use sysinfo::{Pid, System};

        let mut sys = System::new();
        let pid = Pid::from(std::process::id() as usize);
        sys.refresh_process(pid);

        if let Some(process) = sys.process(pid) {
            process.cpu_usage() as f64
        } else {
            0.0
        }
    }
}

/// CPU 统计信息
#[cfg(feature = "profiling")]
struct CpuStats {
    average_usage: f64,
    #[allow(dead_code)]
    peak_usage: f64,
    current_usage: f64,
}

#[cfg(feature = "profiling")]
impl CpuStats {
    fn efficiency_score(&self) -> f64 {
        // CPU 效率：使用率适中最好
        let optimal_usage = 50.0;
        100.0 - (self.average_usage - optimal_usage).abs().min(100.0)
    }
}

/// 内存分析器 - 简化版本，专注于查询操作的内存使用
#[cfg(feature = "profiling")]
pub struct MemoryProfiler {
    monitor: PerformanceMonitor,
}

#[cfg(feature = "profiling")]
impl MemoryProfiler {
    /// 创建新的内存分析器
    pub fn new() -> Self {
        Self {
            monitor: PerformanceMonitor::new(),
        }
    }

    /// 开始内存分析
    pub fn start(&mut self) {
        self.monitor.start();
    }

    /// 停止内存分析并返回报告
    pub fn stop(&mut self) -> ProfileReport {
        self.monitor.stop()
    }
}

#[cfg(feature = "profiling")]
impl Default for MemoryProfiler {
    fn default() -> Self {
        Self::new()
    }
}

// 当 profiling feature 未启用时的空实现
#[cfg(not(feature = "profiling"))]
pub struct ProfileReport;

#[cfg(not(feature = "profiling"))]
impl ProfileReport {
    pub fn summary(&self) -> String {
        "Performance profiling not enabled".to_string()
    }

    pub fn to_html(&self) -> String {
        "<html><body><p>Performance profiling not enabled. Enable the 'profiling' feature to use this functionality.</p></body></html>".to_string()
    }
}

#[cfg(not(feature = "profiling"))]
pub struct PerformanceMonitor;

#[cfg(not(feature = "profiling"))]
impl PerformanceMonitor {
    pub fn new() -> Self {
        Self
    }
    pub fn start(&mut self) {}
    pub fn stop(&mut self) -> ProfileReport {
        ProfileReport
    }
    pub fn get_current_metrics(
        &self,
    ) -> std::collections::HashMap<String, f64> {
        std::collections::HashMap::new()
    }
}

#[cfg(not(feature = "profiling"))]
impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "profiling"))]
pub struct MemoryProfiler;

#[cfg(not(feature = "profiling"))]
impl MemoryProfiler {
    pub fn new() -> Self {
        Self
    }
    pub fn start(&mut self) {}
    pub fn stop(&mut self) -> ProfileReport {
        ProfileReport
    }
}

#[cfg(not(feature = "profiling"))]
impl Default for MemoryProfiler {
    fn default() -> Self {
        Self::new()
    }
}
