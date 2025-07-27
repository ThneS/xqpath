use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::{json, Value};
use xqpath::{evaluate_path_expression, parse_path_expression};

// 生成测试数据
fn generate_test_data(user_count: usize) -> Value {
    let users: Vec<Value> = (0..user_count)
        .map(|i| {
            json!({
                "id": i,
                "name": format!("User{}", i),
                "age": 20 + (i % 50),
                "active": i % 3 == 0,
                "email": format!("user{}@example.com", i),
                "profile": {
                    "avatar": format!("avatar{}.png", i),
                    "bio": format!("Biography for user {}", i),
                    "preferences": {
                        "theme": if i % 2 == 0 { "dark" } else { "light" },
                        "notifications": i % 4 == 0
                    }
                },
                "scores": vec![i as f64 * 1.1, i as f64 * 2.2, i as f64 * 0.8]
            })
        })
        .collect();

    json!({
        "users": users,
        "metadata": {
            "total": user_count,
            "created": "2024-01-01T00:00:00Z",
            "version": "1.0"
        },
        "config": {
            "pagination": {
                "page_size": 50,
                "max_pages": 100
            },
            "features": {
                "search": true,
                "export": true,
                "analytics": false
            }
        }
    })
}

// 基准测试：简单路径提取
fn bench_simple_path(c: &mut Criterion) {
    let data = generate_test_data(1000);
    let expr = parse_path_expression(".users[0].name").unwrap();

    c.bench_function("simple_path_extraction", |b| {
        b.iter(|| evaluate_path_expression(black_box(&expr), black_box(&data)))
    });
}

// 基准测试：数组映射
fn bench_array_mapping(c: &mut Criterion) {
    let data = generate_test_data(1000);
    let expr = parse_path_expression(".users | map(.name)").unwrap();

    c.bench_function("array_mapping", |b| {
        b.iter(|| evaluate_path_expression(black_box(&expr), black_box(&data)))
    });
}

// 基准测试：条件过滤
fn bench_conditional_filtering(c: &mut Criterion) {
    let data = generate_test_data(1000);
    let expr = parse_path_expression(".users | select(.active)").unwrap();

    c.bench_function("conditional_filtering", |b| {
        b.iter(|| evaluate_path_expression(black_box(&expr), black_box(&data)))
    });
}

// 基准测试：复杂查询
fn bench_complex_query(c: &mut Criterion) {
    let data = generate_test_data(1000);
    let expr = parse_path_expression(
        ".users | select(.active and .age > 25) | map(.name) | sort()",
    )
    .unwrap();

    c.bench_function("complex_query", |b| {
        b.iter(|| evaluate_path_expression(black_box(&expr), black_box(&data)))
    });
}

// 基准测试：嵌套访问
fn bench_nested_access(c: &mut Criterion) {
    let data = generate_test_data(1000);
    let expr =
        parse_path_expression(".users | map(.profile.preferences.theme)")
            .unwrap();

    c.bench_function("nested_access", |b| {
        b.iter(|| evaluate_path_expression(black_box(&expr), black_box(&data)))
    });
}

// 基准测试：表达式解析
fn bench_expression_parsing(c: &mut Criterion) {
    c.bench_function("expression_parsing", |b| {
        b.iter(|| {
            parse_path_expression(black_box(
                ".users | select(.active and .age > 25) | map({name: .name, age: .age}) | sort_by(.age)"
            ))
        })
    });
}

// 基准测试组
criterion_group!(
    benches,
    bench_simple_path,
    bench_array_mapping,
    bench_conditional_filtering,
    bench_complex_query,
    bench_nested_access,
    bench_expression_parsing
);

criterion_main!(benches);
