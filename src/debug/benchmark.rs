//! 基准测试套件模块 - v1.4.2
//!
//! 提供性能基准测试和比较分析功能

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// 基准测试结果
#[cfg(feature = "benchmark")]
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// 测试名称
    pub name: String,
    /// 平均执行时间
    pub mean_time: Duration,
    /// 最小执行时间
    pub min_time: Duration,
    /// 最大执行时间
    pub max_time: Duration,
    /// 标准差
    pub std_dev: Duration,
    /// 执行次数
    pub iterations: usize,
    /// 每秒操作数
    pub ops_per_sec: f64,
}

#[cfg(feature = "benchmark")]
impl BenchmarkResult {
    /// 生成结果摘要
    pub fn summary(&self) -> String {
        format!(
            "{}: {:?} (±{:?}) {} ops/sec",
            self.name, self.mean_time, self.std_dev, self.ops_per_sec as u64
        )
    }

    /// 与基线比较
    pub fn compare_with(&self, baseline: &BenchmarkResult) -> String {
        let ratio = self.mean_time.as_nanos() as f64
            / baseline.mean_time.as_nanos() as f64;

        if ratio > 1.1 {
            format!("⚠️  {} 比基线慢 {:.1}%", self.name, (ratio - 1.0) * 100.0)
        } else if ratio < 0.9 {
            format!("✅ {} 比基线快 {:.1}%", self.name, (1.0 - ratio) * 100.0)
        } else {
            format!(
                "➖ {} 与基线相近 ({:.1}%)",
                self.name,
                (ratio - 1.0) * 100.0
            )
        }
    }
}

/// 基准测试函数类型
#[cfg(feature = "benchmark")]
type BenchmarkTestFn = Box<dyn Fn() -> Result<(), Box<dyn std::error::Error>>>;

/// 基准测试套件
#[cfg(feature = "benchmark")]
pub struct BenchmarkSuite {
    tests: Vec<(String, BenchmarkTestFn)>,
    config: BenchmarkConfig,
}

/// 基准测试配置
#[cfg(feature = "benchmark")]
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// 预热次数
    pub warmup_iterations: usize,
    /// 测试次数
    pub test_iterations: usize,
    /// 最小测试时间
    pub min_test_time: Duration,
    /// 最大测试时间
    pub max_test_time: Duration,
}

#[cfg(feature = "benchmark")]
impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            warmup_iterations: 10,
            test_iterations: 100,
            min_test_time: Duration::from_millis(100),
            max_test_time: Duration::from_secs(10),
        }
    }
}

#[cfg(feature = "benchmark")]
impl BenchmarkSuite {
    /// 创建新的基准测试套件
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            config: BenchmarkConfig::default(),
        }
    }

    /// 使用自定义配置创建基准测试套件
    pub fn with_config(config: BenchmarkConfig) -> Self {
        Self {
            tests: Vec::new(),
            config,
        }
    }

    /// 添加测试用例
    pub fn add_test<F>(&mut self, name: impl Into<String>, test_fn: F)
    where
        F: Fn() -> Result<(), Box<dyn std::error::Error>> + 'static,
    {
        self.tests.push((name.into(), Box::new(test_fn)));
    }

    /// 运行所有基准测试
    pub fn run(
        &self,
    ) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        for (name, test_fn) in &self.tests {
            println!("运行基准测试: {name}");
            let result = self.run_single_test(name, test_fn)?;
            println!("  {}", result.summary());
            results.push(result);
        }

        Ok(results)
    }

    /// 运行单个测试
    fn run_single_test(
        &self,
        name: &str,
        test_fn: &dyn Fn() -> Result<(), Box<dyn std::error::Error>>,
    ) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        // 预热
        for _ in 0..self.config.warmup_iterations {
            test_fn()?;
        }

        let mut times = Vec::new();
        let start_time = Instant::now();

        // 执行测试
        for i in 0..self.config.test_iterations {
            // 检查是否超时
            if start_time.elapsed() > self.config.max_test_time {
                break;
            }

            // 如果还没达到最小时间，继续测试
            if i >= self.config.test_iterations
                && start_time.elapsed() < self.config.min_test_time
            {
                continue;
            }

            let test_start = Instant::now();
            test_fn()?;
            let test_time = test_start.elapsed();
            times.push(test_time);
        }

        if times.is_empty() {
            return Err("没有成功的测试迭代".into());
        }

        // 计算统计数据
        let total_time: Duration = times.iter().sum();
        let mean_time = total_time / times.len() as u32;
        let min_time = *times.iter().min().unwrap();
        let max_time = *times.iter().max().unwrap();

        // 计算标准差
        let variance: f64 = times
            .iter()
            .map(|&time| {
                let diff = time.as_nanos() as f64 - mean_time.as_nanos() as f64;
                diff * diff
            })
            .sum::<f64>()
            / times.len() as f64;
        let std_dev = Duration::from_nanos(variance.sqrt() as u64);

        // 计算每秒操作数
        let ops_per_sec = 1_000_000_000.0 / mean_time.as_nanos() as f64;

        Ok(BenchmarkResult {
            name: name.to_string(),
            mean_time,
            min_time,
            max_time,
            std_dev,
            iterations: times.len(),
            ops_per_sec,
        })
    }

    /// 与基线结果比较
    pub fn compare_with_baseline(
        results: &[BenchmarkResult],
        baseline: &[BenchmarkResult],
    ) -> Vec<String> {
        let baseline_map: HashMap<_, _> =
            baseline.iter().map(|r| (r.name.clone(), r)).collect();

        results
            .iter()
            .filter_map(|result| {
                baseline_map
                    .get(&result.name)
                    .map(|baseline| result.compare_with(baseline))
            })
            .collect()
    }

    /// 生成HTML格式的基准测试报告
    pub fn generate_html_report(results: &[BenchmarkResult]) -> String {
        let mut html = String::new();
        html.push_str(
            "<!DOCTYPE html><html><head><title>XQPath 基准测试报告</title>",
        );
        html.push_str("<style>");
        html.push_str("body{font-family:Arial,sans-serif;margin:20px;}");
        html.push_str("table{border-collapse:collapse;width:100%;}");
        html.push_str(
            "th,td{border:1px solid #ddd;padding:8px;text-align:left;}",
        );
        html.push_str("th{background-color:#f2f2f2;}");
        html.push_str("tr:nth-child(even){background-color:#f9f9f9;}");
        html.push_str(".fast{color:#4CAF50;font-weight:bold;}");
        html.push_str(".slow{color:#f44336;font-weight:bold;}");
        html.push_str("</style></head><body>");

        html.push_str("<h1>XQPath 基准测试报告</h1>");
        html.push_str("<table>");
        html.push_str("<tr><th>测试名称</th><th>平均时间</th><th>最小时间</th><th>最大时间</th><th>标准差</th><th>操作/秒</th></tr>");

        for result in results {
            let class = if result.ops_per_sec > 1000.0 {
                "fast"
            } else if result.ops_per_sec < 100.0 {
                "slow"
            } else {
                ""
            };
            html.push_str(&format!("<tr class='{class}'>"));
            html.push_str(&format!("<td>{}</td>", result.name));
            html.push_str(&format!("<td>{:?}</td>", result.mean_time));
            html.push_str(&format!("<td>{:?}</td>", result.min_time));
            html.push_str(&format!("<td>{:?}</td>", result.max_time));
            html.push_str(&format!("<td>{:?}</td>", result.std_dev));
            html.push_str(&format!("<td>{:.0}</td>", result.ops_per_sec));
            html.push_str("</tr>");
        }

        html.push_str("</table>");
        html.push_str("</body></html>");
        html
    }

    /// 保存基准测试结果到文件
    pub fn save_results_to_file(
        results: &[BenchmarkResult],
        filename: &str,
        format: BenchmarkOutputFormat,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;

        let content = match format {
            BenchmarkOutputFormat::Json => {
                serde_json::to_string_pretty(results)?
            }
            BenchmarkOutputFormat::Html => Self::generate_html_report(results),
            BenchmarkOutputFormat::Csv => {
                let mut csv = String::from("name,mean_time_ns,min_time_ns,max_time_ns,std_dev_ns,ops_per_sec\n");
                for result in results {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{}\n",
                        result.name,
                        result.mean_time.as_nanos(),
                        result.min_time.as_nanos(),
                        result.max_time.as_nanos(),
                        result.std_dev.as_nanos(),
                        result.ops_per_sec
                    ));
                }
                csv
            }
        };

        fs::write(filename, content)?;
        Ok(())
    }
}

/// 基准测试输出格式
#[cfg(feature = "benchmark")]
#[derive(Debug, Clone)]
pub enum BenchmarkOutputFormat {
    Json,
    Html,
    Csv,
}

#[cfg(feature = "benchmark")]
impl Default for BenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}

// Serde 支持
#[cfg(feature = "benchmark")]
#[derive(Serialize, Deserialize)]
struct SerializableBenchmarkResult {
    name: String,
    mean_time_ns: u64,
    min_time_ns: u64,
    max_time_ns: u64,
    std_dev_ns: u64,
    iterations: usize,
    ops_per_sec: f64,
}

#[cfg(feature = "benchmark")]
impl From<&BenchmarkResult> for SerializableBenchmarkResult {
    fn from(result: &BenchmarkResult) -> Self {
        Self {
            name: result.name.clone(),
            mean_time_ns: result.mean_time.as_nanos() as u64,
            min_time_ns: result.min_time.as_nanos() as u64,
            max_time_ns: result.max_time.as_nanos() as u64,
            std_dev_ns: result.std_dev.as_nanos() as u64,
            iterations: result.iterations,
            ops_per_sec: result.ops_per_sec,
        }
    }
}

#[cfg(feature = "benchmark")]
impl Serialize for BenchmarkResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let serializable: SerializableBenchmarkResult = self.into();
        serializable.serialize(serializer)
    }
}

// 当 benchmark feature 未启用时的空实现
#[cfg(not(feature = "benchmark"))]
pub struct BenchmarkSuite;

#[cfg(not(feature = "benchmark"))]
pub struct BenchmarkResult;

#[cfg(not(feature = "benchmark"))]
pub struct BenchmarkConfig;

#[cfg(not(feature = "benchmark"))]
impl BenchmarkSuite {
    pub fn new() -> Self {
        Self
    }
    pub fn add_test<F>(&mut self, _name: impl Into<String>, _test_fn: F)
    where
        F: Fn() -> Result<(), Box<dyn std::error::Error>> + 'static,
    {
    }
    pub fn run(
        &self,
    ) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
        Err("Benchmark feature not enabled".into())
    }
}

#[cfg(not(feature = "benchmark"))]
impl Default for BenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "benchmark"))]
impl BenchmarkResult {
    pub fn summary(&self) -> String {
        "Benchmark feature not enabled".to_string()
    }
}
