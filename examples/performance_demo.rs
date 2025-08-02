//! # XQPath v1.4.2 性能分析功能演示
//!
//! 本示例展示了 v1.4.2 版本的性能监控分析功能

use xqpath::{query, query_one};

#[cfg(feature = "profiling")]
use xqpath::{
    profile_complete, query_memory, query_with_profile, PerformanceMonitor,
};

#[cfg(feature = "benchmark")]
use xqpath::{
    benchmark_query, BenchmarkConfig, BenchmarkOutputFormat, BenchmarkSuite,
};

use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 XQPath v1.4.2 性能分析功能演示\n");

    // 准备测试数据
    let test_data = json!({
        "users": [
            {"name": "Alice", "age": 30, "scores": [85, 92, 78, 96]},
            {"name": "Bob", "age": 25, "scores": [88, 76, 89, 94]},
            {"name": "Carol", "age": 35, "scores": [91, 95, 87, 92]},
            {"name": "David", "age": 28, "scores": [79, 83, 90, 85]}
        ],
        "metadata": {
            "version": "1.0",
            "created": "2024-01-01",
            "total_users": 4
        }
    });

    let data_str = serde_json::to_string(&test_data)?;

    // 基础查询演示
    println!("0️⃣  基础查询功能（总是可用）");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let names = query!(data_str, ".users[*].name")?;
    println!("用户名称: {names:?}");

    let version = query_one!(data_str, ".metadata.version")?;
    println!("版本: {version:?}");
    println!();

    // 1. 基础性能分析
    demo_basic_profiling(&data_str)?;

    // 2. 内存使用分析
    demo_memory_analysis(&data_str)?;

    // 3. 基准测试
    demo_benchmarking(&data_str)?;

    // 4. 完整性能分析
    demo_complete_performance(&data_str)?;

    // 5. 基准测试套件演示
    demo_benchmark_suite(&data_str)?;

    // 6. 性能监控器演示
    demo_performance_monitor(&data_str)?;

    println!("\n✨ v1.4.2 性能分析功能演示完成！");

    println!("\n💡 要完整体验性能分析功能，请运行:");
    println!("   cargo run --features=\"profiling,benchmark\" --example performance_demo");

    Ok(())
}

fn demo_basic_profiling(
    _data_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("1️⃣  基础性能分析 (需要 profiling feature)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    #[cfg(feature = "profiling")]
    {
        let (result, profile) =
            query_with_profile!(data_str, ".users[*].name")?;
        println!("查询结果: {result:?}");
        println!("性能报告: {}", profile.summary());

        if !profile.optimization_hints.is_empty() {
            println!("优化建议:");
            for hint in &profile.optimization_hints {
                println!("  💡 {hint}");
            }
        }
    }

    #[cfg(not(feature = "profiling"))]
    {
        println!("⚠️  profiling feature 未启用，跳过此演示");
    }

    println!();
    Ok(())
}

fn demo_memory_analysis(
    _data_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("2️⃣  内存使用分析 (需要 profiling feature)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    #[cfg(feature = "profiling")]
    {
        let (result, memory_report) =
            query_memory!(data_str, ".users[*].scores[*]")?;
        println!("查询结果数量: {}", result.len());
        println!(
            "峰值内存: {:.2} MB",
            memory_report.peak_memory_bytes as f64 / 1024.0 / 1024.0
        );
        println!(
            "当前内存: {:.2} MB",
            memory_report.current_memory_bytes as f64 / 1024.0 / 1024.0
        );

        if let Some(efficiency) = memory_report.metrics.get("memory_efficiency")
        {
            println!("内存效率: {efficiency:.1}%");
        }
    }

    #[cfg(not(feature = "profiling"))]
    {
        println!("⚠️  profiling feature 未启用，跳过此演示");
    }

    println!();
    Ok(())
}

fn demo_benchmarking(
    _data_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("3️⃣  基准测试 (需要 benchmark feature)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    #[cfg(feature = "benchmark")]
    {
        let data_owned = data_str.to_string();
        let (result, benchmark_result) =
            benchmark_query!(data_owned, ".users[*].name", 50)?;
        println!("查询结果: {result:?}");
        println!("基准测试结果: {}", benchmark_result.summary());
    }

    #[cfg(not(feature = "benchmark"))]
    {
        println!("⚠️  benchmark feature 未启用，跳过此演示");
    }

    println!();
    Ok(())
}

fn demo_complete_performance(
    _data_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("4️⃣  完整性能分析 (需要 profiling feature)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    #[cfg(feature = "profiling")]
    {
        let (result, profile) = profile_complete!(data_str, ".users[*].name")?;
        println!("查询结果: {result:?}");
        println!("详细性能报告:");
        println!("  ⏱️  执行时间: {:?}", profile.execution_time);
        println!(
            "  💾 峰值内存: {:.2} MB",
            profile.peak_memory_bytes as f64 / 1024.0 / 1024.0
        );
        println!("  🖥️  CPU使用: {:.1}%", profile.cpu_usage_percent);

        println!("  📊 详细指标:");
        for (name, value) in &profile.metrics {
            println!("    • {name}: {value:.3}");
        }
    }

    #[cfg(not(feature = "profiling"))]
    {
        println!("⚠️  profiling feature 未启用，跳过此演示");
    }

    println!();
    Ok(())
}

fn demo_benchmark_suite(
    _data_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("5️⃣  基准测试套件演示 (需要 benchmark feature)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    #[cfg(feature = "benchmark")]
    {
        use std::time::Duration;

        let config = BenchmarkConfig {
            warmup_iterations: 5,
            test_iterations: 30,
            min_test_time: Duration::from_millis(50),
            max_test_time: Duration::from_secs(5),
        };

        let mut suite = BenchmarkSuite::with_config(config);
        let data_clone = data_str.to_string();

        // 添加不同复杂度的测试
        suite.add_test("simple_path", {
            let data = data_clone.clone();
            move || {
                let _result = query!(data, ".metadata.version")?;
                Ok(())
            }
        });

        suite.add_test("array_access", {
            let data = data_clone.clone();
            move || {
                let _result = query!(data, ".users[*].name")?;
                Ok(())
            }
        });

        suite.add_test("nested_array", {
            let data = data_clone.clone();
            move || {
                let _result = query!(data, ".users[*].scores[*]")?;
                Ok(())
            }
        });

        let results = suite.run()?;

        println!("基准测试结果:");
        for result in &results {
            println!("  {}", result.summary());
        }

        // 保存报告
        if let Err(e) = BenchmarkSuite::save_results_to_file(
            &results,
            "benchmark_report.html",
            BenchmarkOutputFormat::Html,
        ) {
            println!("⚠️  保存HTML报告失败: {e}");
        } else {
            println!("\n📄 HTML报告已保存到: benchmark_report.html");
        }
    }

    #[cfg(not(feature = "benchmark"))]
    {
        println!("⚠️  benchmark feature 未启用，跳过此演示");
    }

    println!();
    Ok(())
}

fn demo_performance_monitor(
    _data_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("6️⃣  性能监控器演示 (需要 profiling feature)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    #[cfg(feature = "profiling")]
    {
        let mut monitor = PerformanceMonitor::new();
        monitor.start();

        // 模拟一些工作负载
        for i in 0..3 {
            let path = match i {
                0 => ".users[*].name",
                1 => ".users[*].age",
                _ => ".users[*].scores[*]",
            };

            let _result = query!(data_str, path)?;

            // 获取当前指标
            let current_metrics = monitor.get_current_metrics();
            println!("步骤 {}: 当前指标", i + 1);
            for (name, value) in current_metrics {
                println!("  • {name}: {value:.2}");
            }
        }

        let final_report = monitor.stop();

        println!("\n最终性能报告:");
        println!("  {}", final_report.summary());

        // 保存HTML报告
        if let Err(e) =
            std::fs::write("performance_report.html", final_report.to_html())
        {
            println!("⚠️  保存性能报告失败: {e}");
        } else {
            println!("📄 HTML性能报告已保存到: performance_report.html");
        }
    }

    #[cfg(not(feature = "profiling"))]
    {
        println!("⚠️  profiling feature 未启用，跳过此演示");
    }

    println!();
    Ok(())
}
