//! 高级基准测试套件 - v1.4.2
//!
//! 集成了 debug/benchmark.rs 中的高级基准测试功能

use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use serde_json::{json, Value};
use xqpath::{query, query_one};

#[cfg(feature = "profiling")]
use xqpath::{
    profile_complete, query_memory, query_with_profile, PerformanceMonitor,
};

#[cfg(feature = "benchmark")]
use xqpath::benchmark_query;

// 生成不同复杂度的测试数据
fn generate_complex_test_data(user_count: usize, nested_depth: usize) -> Value {
    let users: Vec<Value> = (0..user_count)
        .map(|i| {
            let mut user = json!({
                "id": i,
                "name": format!("User{}", i),
                "age": 20 + (i % 50),
                "active": i % 3 == 0,
                "email": format!("user{}@example.com", i),
                "scores": vec![i as f64 * 1.1, i as f64 * 2.2, i as f64 * 0.8],
                "tags": vec![
                    format!("tag{}", i % 5),
                    format!("category{}", i % 3),
                    format!("level{}", i % 4)
                ]
            });

            // 添加嵌套结构
            if nested_depth > 0 {
                let mut nested = json!({});
                for depth in 0..nested_depth {
                    nested = json!({
                        format!("level{}", depth): nested,
                        "value": i + depth,
                        "data": format!("nested_data_{}_{}", depth, i)
                    });
                }
                user.as_object_mut()
                    .unwrap()
                    .insert("nested".to_string(), nested);
            }

            user
        })
        .collect();

    json!({
        "users": users,
        "metadata": {
            "total": user_count,
            "created": "2024-01-01T00:00:00Z",
            "version": "1.4.2",
            "nested_depth": nested_depth
        },
        "stats": {
            "active_users": users.iter().filter(|u| u["active"].as_bool().unwrap_or(false)).count(),
            "avg_age": users.iter().map(|u| u["age"].as_u64().unwrap_or(0)).sum::<u64>() / user_count as u64,
            "total_scores": users.len() * 3
        }
    })
}

// 基准测试：基础查询性能
fn bench_basic_queries(c: &mut Criterion) {
    let data = generate_complex_test_data(1000, 0);
    let data_str = serde_json::to_string(&data).unwrap();

    let mut group = c.benchmark_group("basic_queries");

    group.bench_function("single_field", |b| {
        b.iter(|| {
            let _result =
                query!(black_box(&data_str), ".metadata.version").unwrap();
        })
    });

    group.bench_function("array_access", |b| {
        b.iter(|| {
            let _result =
                query!(black_box(&data_str), ".users[0].name").unwrap();
        })
    });

    group.bench_function("wildcard_array", |b| {
        b.iter(|| {
            let _result =
                query!(black_box(&data_str), ".users[*].name").unwrap();
        })
    });

    group.bench_function("nested_array", |b| {
        b.iter(|| {
            let _result =
                query!(black_box(&data_str), ".users[*].scores[*]").unwrap();
        })
    });

    group.finish();
}

// 基准测试：数据集大小对性能的影响
fn bench_scaling_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling_performance");

    for size in [10, 100, 1000, 10000].iter() {
        let data = generate_complex_test_data(*size, 0);
        let data_str = serde_json::to_string(&data).unwrap();

        group.bench_with_input(
            BenchmarkId::new("extract_all_names", size),
            size,
            |b, _| {
                b.iter(|| {
                    let _result =
                        query!(black_box(&data_str), ".users[*].name").unwrap();
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("extract_nested_scores", size),
            size,
            |b, _| {
                b.iter(|| {
                    let _result =
                        query!(black_box(&data_str), ".users[*].scores[*]")
                            .unwrap();
                })
            },
        );
    }

    group.finish();
}

// 基准测试：嵌套深度对性能的影响
fn bench_nesting_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("nesting_performance");

    for depth in [1, 3, 5, 10].iter() {
        let data = generate_complex_test_data(100, *depth);
        let data_str = serde_json::to_string(&data).unwrap();

        group.bench_with_input(
            BenchmarkId::new("deep_nested_access", depth),
            depth,
            |b, depth| {
                let path = (0..*depth)
                    .map(|i| format!(".level{i}"))
                    .collect::<Vec<_>>()
                    .join("");
                let full_path = format!(".users[*].nested{path}");

                b.iter(|| {
                    let _result =
                        query!(black_box(&data_str), &full_path).unwrap();
                })
            },
        );
    }

    group.finish();
}

// 基准测试：性能分析功能本身的开销
#[cfg(feature = "profiling")]
fn bench_profiling_overhead(c: &mut Criterion) {
    let data = generate_complex_test_data(1000, 0);
    let data_str = serde_json::to_string(&data).unwrap();

    let mut group = c.benchmark_group("profiling_overhead");

    group.bench_function("normal_query", |b| {
        b.iter(|| {
            let _result =
                query!(black_box(&data_str), ".users[*].name").unwrap();
        })
    });

    group.bench_function("profiled_query", |b| {
        b.iter(|| {
            let (_result, _profile) =
                query_with_profile!(black_box(&data_str), ".users[*].name")
                    .unwrap();
        })
    });

    group.bench_function("memory_query", |b| {
        b.iter(|| {
            let (_result, _memory_report) =
                query_memory!(black_box(&data_str), ".users[*].name").unwrap();
        })
    });

    group.bench_function("complete_profile", |b| {
        b.iter(|| {
            let (_result, _profile) =
                profile_complete!(black_box(&data_str), ".users[*].name")
                    .unwrap();
        })
    });

    group.finish();
}

// 基准测试：使用自定义基准测试套件
#[cfg(feature = "benchmark")]
fn bench_custom_benchmark_suite(c: &mut Criterion) {
    let data = generate_complex_test_data(500, 2);
    let data_str = serde_json::to_string(&data).unwrap();

    c.bench_function("benchmark_suite_overhead", |b| {
        b.iter(|| {
            let (_result, _benchmark_result) = benchmark_query!(
                black_box(&data_str),
                ".users[*].name",
                black_box(10)
            )
            .unwrap();
        })
    });
}

// 基准测试：复杂查询模式
fn bench_complex_patterns(c: &mut Criterion) {
    let data = generate_complex_test_data(1000, 3);
    let data_str = serde_json::to_string(&data).unwrap();

    let mut group = c.benchmark_group("complex_patterns");

    group.bench_function("multiple_arrays", |b| {
        b.iter(|| {
            let _result =
                query!(black_box(&data_str), ".users[*].tags[*]").unwrap();
        })
    });

    group.bench_function("query_one_optimization", |b| {
        b.iter(|| {
            let _result =
                query_one!(black_box(&data_str), ".metadata.version").unwrap();
        })
    });

    group.bench_function("deep_nested_path", |b| {
        b.iter(|| {
            let _result = query!(
                black_box(&data_str),
                ".users[*].nested.level0.level1.level2.value"
            )
            .unwrap();
        })
    });

    group.finish();
}

// 基准测试：内存使用模式
#[cfg(feature = "profiling")]
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    for size in [100, 1000, 5000].iter() {
        let data = generate_complex_test_data(*size, 0);
        let data_str = serde_json::to_string(&data).unwrap();

        group.bench_with_input(
            BenchmarkId::new("memory_tracking", size),
            size,
            |b, _| {
                b.iter(|| {
                    let mut monitor = PerformanceMonitor::new();
                    monitor.start();
                    let _result =
                        query!(black_box(&data_str), ".users[*].scores[*]")
                            .unwrap();
                    let _report = monitor.stop();
                })
            },
        );
    }

    group.finish();
}

// 创建基准测试组
criterion_group!(
    basic_benches,
    bench_basic_queries,
    bench_scaling_performance,
    bench_nesting_performance,
    bench_complex_patterns
);

#[cfg(feature = "profiling")]
criterion_group!(
    profiling_benches,
    bench_profiling_overhead,
    bench_memory_patterns
);

#[cfg(feature = "benchmark")]
criterion_group!(benchmark_benches, bench_custom_benchmark_suite);

// 根据可用功能选择要运行的基准测试
#[cfg(all(feature = "profiling", feature = "benchmark"))]
criterion_main!(basic_benches, profiling_benches, benchmark_benches);

#[cfg(all(feature = "profiling", not(feature = "benchmark")))]
criterion_main!(basic_benches, profiling_benches);

#[cfg(all(not(feature = "profiling"), feature = "benchmark"))]
criterion_main!(basic_benches, benchmark_benches);

#[cfg(all(not(feature = "profiling"), not(feature = "benchmark")))]
criterion_main!(basic_benches);
