//! 执行路径跟踪器

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// 跟踪事件
#[derive(Debug, Clone)]
pub struct TraceEvent {
    pub timestamp: Instant,
    pub path: String,
    pub operation: String,
    pub duration: Option<Duration>,
    pub result: TraceResult,
}

/// 跟踪结果
#[derive(Debug, Clone)]
pub enum TraceResult {
    Success(usize), // 返回结果数量
    Error(String),
}

/// 执行路径跟踪器
pub struct Tracer {
    events: VecDeque<TraceEvent>,
    max_events: usize,
    enabled: bool,
}

impl Default for Tracer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tracer {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            max_events: 1000,
            enabled: false,
        }
    }

    pub fn with_max_events(mut self, max: usize) -> Self {
        self.max_events = max;
        self
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 开始跟踪一个操作
    pub fn start_trace(&mut self, path: &str, operation: &str) -> TraceHandle {
        if !self.enabled {
            return TraceHandle::disabled();
        }

        let start_time = Instant::now();
        TraceHandle {
            tracer: self as *mut Tracer,
            path: path.to_string(),
            operation: operation.to_string(),
            start_time,
            enabled: true,
        }
    }

    /// 记录跟踪事件
    pub fn record_event(&mut self, event: TraceEvent) {
        if !self.enabled {
            return;
        }

        // 限制事件数量，避免内存无限增长
        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }

        self.events.push_back(event);
    }

    /// 获取所有跟踪事件
    pub fn get_events(&self) -> &VecDeque<TraceEvent> {
        &self.events
    }

    /// 获取最近的跟踪事件
    pub fn get_recent_events(&self, count: usize) -> Vec<&TraceEvent> {
        self.events.iter().rev().take(count).collect()
    }

    /// 清除所有跟踪事件
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// 生成执行路径摘要
    pub fn get_execution_summary(&self) -> ExecutionSummary {
        let total_operations = self.events.len();
        let successful_operations = self
            .events
            .iter()
            .filter(|e| matches!(e.result, TraceResult::Success(_)))
            .count();

        let total_duration =
            self.events.iter().filter_map(|e| e.duration).sum();

        let average_duration = if total_operations > 0 {
            total_duration / total_operations as u32
        } else {
            Duration::from_nanos(0)
        };

        ExecutionSummary {
            total_operations,
            successful_operations,
            failed_operations: total_operations - successful_operations,
            total_duration,
            average_duration,
        }
    }
}

/// 跟踪句柄
pub struct TraceHandle {
    tracer: *mut Tracer,
    path: String,
    operation: String,
    start_time: Instant,
    enabled: bool,
}

impl TraceHandle {
    fn disabled() -> Self {
        Self {
            tracer: std::ptr::null_mut(),
            path: String::new(),
            operation: String::new(),
            start_time: Instant::now(),
            enabled: false,
        }
    }

    /// 完成跟踪，记录成功结果
    pub fn finish_success(self, result_count: usize) {
        if !self.enabled {
            return;
        }

        let duration = self.start_time.elapsed();
        let event = TraceEvent {
            timestamp: self.start_time,
            path: self.path,
            operation: self.operation,
            duration: Some(duration),
            result: TraceResult::Success(result_count),
        };

        unsafe {
            if !self.tracer.is_null() {
                (*self.tracer).record_event(event);
            }
        }
    }

    /// 完成跟踪，记录错误结果
    pub fn finish_error(self, error: String) {
        if !self.enabled {
            return;
        }

        let duration = self.start_time.elapsed();
        let event = TraceEvent {
            timestamp: self.start_time,
            path: self.path,
            operation: self.operation,
            duration: Some(duration),
            result: TraceResult::Error(error),
        };

        unsafe {
            if !self.tracer.is_null() {
                (*self.tracer).record_event(event);
            }
        }
    }
}

/// 执行摘要
#[derive(Debug, Clone)]
pub struct ExecutionSummary {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
}

impl std::fmt::Display for ExecutionSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "Execution Summary:\n  Total Operations: {}\n  Successful: {}\n  Failed: {}\n  Total Duration: {:?}\n  Average Duration: {:?}",
            self.total_operations,
            self.successful_operations,
            self.failed_operations,
            self.total_duration,
            self.average_duration
        )
    }
}
