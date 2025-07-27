use serde_json::{json, Value};
use std::time::Instant;
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
                "email": format!("user{}@example.com", i)
            })
        })
        .collect();

    json!({
        "users": users,
        "metadata": {
            "total": user_count,
            "version": "1.0"
        }
    })
}

// 简单性能测试函数
fn benchmark_expression(
    name: &str,
    expression: &str,
    data: &Value,
    iterations: usize,
) {
    println!("=== {name} ===");

    // 测试解析时间
    let parse_start = Instant::now();
    let expr = match parse_path_expression(expression) {
        Ok(expr) => expr,
        Err(e) => {
            println!("解析错误: {e}");
            return;
        }
    };
    let parse_time = parse_start.elapsed();

    // 测试执行时间
    let mut total_eval_time = std::time::Duration::new(0, 0);
    let mut success_count = 0;

    for _ in 0..iterations {
        let eval_start = Instant::now();
        match evaluate_path_expression(&expr, data) {
            Ok(_result) => {
                total_eval_time += eval_start.elapsed();
                success_count += 1;
            }
            Err(e) => {
                println!("执行错误: {e}");
                return;
            }
        }
    }

    let avg_eval_time = total_eval_time / success_count as u32;

    println!("表达式: {expression}");
    println!("解析时间: {parse_time:?}");
    println!(
        "平均执行时间: {avg_eval_time:?} (基于{success_count}次执行)"
    );
    println!("总执行时间: {total_eval_time:?}");
    println!();
}

fn main() {
    println!("XQPath 性能测试演示");
    println!("==================");

    // 生成不同大小的测试数据
    let small_data = generate_test_data(100);
    let medium_data = generate_test_data(1000);
    let large_data = generate_test_data(5000);

    println!("数据集大小:");
    println!("- 小数据集: 100 用户");
    println!("- 中数据集: 1000 用户");
    println!("- 大数据集: 5000 用户");
    println!();

    // 测试简单路径提取
    println!("📊 小数据集 (100 用户) 测试结果:");
    benchmark_expression("简单路径提取", ".users[0].name", &small_data, 1000);
    benchmark_expression("数组长度", ".users | length()", &small_data, 1000);
    benchmark_expression("映射操作", ".users | map(.name)", &small_data, 100);
    benchmark_expression(
        "条件过滤",
        ".users | select(.active)",
        &small_data,
        100,
    );

    println!("📊 中数据集 (1000 用户) 测试结果:");
    benchmark_expression("简单路径提取", ".users[0].name", &medium_data, 1000);
    benchmark_expression("数组长度", ".users | length()", &medium_data, 1000);
    benchmark_expression("映射操作", ".users | map(.name)", &medium_data, 50);
    benchmark_expression(
        "条件过滤",
        ".users | select(.active)",
        &medium_data,
        50,
    );

    println!("📊 大数据集 (5000 用户) 测试结果:");
    benchmark_expression("简单路径提取", ".users[0].name", &large_data, 1000);
    benchmark_expression("数组长度", ".users | length()", &large_data, 1000);
    benchmark_expression("映射操作", ".users | map(.name)", &large_data, 10);
    benchmark_expression(
        "条件过滤",
        ".users | select(.active)",
        &large_data,
        10,
    );

    println!("📊 复杂表达式测试 (中数据集):");
    benchmark_expression(
        "复合查询",
        ".users | select(.active and .age > 30) | map(.name)",
        &medium_data,
        20,
    );

    benchmark_expression(
        "条件表达式",
        ".users | map(if .age >= 30 then \"senior\" else \"junior\" end)",
        &medium_data,
        20,
    );

    println!("✅ 性能测试完成！");
    println!();
    println!("💡 提示:");
    println!("- 解析时间只需要执行一次，可以缓存表达式对象");
    println!("- 实际性能会因硬件配置和数据特征而变化");
    println!("- 使用 `cargo bench` 可以运行更精确的基准测试");
}
