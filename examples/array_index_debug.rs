use serde_json::json;
use xqpath::{evaluate_path_expression, parse_path_expression};

fn main() {
    let data = json!({
        "users": [
            {"name": "Alice", "email": "alice@example.com"},
            {"name": "Bob"}
        ]
    });

    println!("Testing array index behavior...");

    // 测试正常索引
    let expr_str = ".users[1]";
    println!("Testing: {}", expr_str);
    match parse_path_expression(expr_str) {
        Ok(expr) => match evaluate_path_expression(&expr, &data) {
            Ok(result) => println!("  ✓ Result: {:?}", result),
            Err(e) => println!("  ✗ Error: {}", e),
        },
        Err(e) => println!("  ✗ Parse error: {}", e),
    }

    // 测试越界索引
    let expr_str = ".users[10]";
    println!("\nTesting: {}", expr_str);
    match parse_path_expression(expr_str) {
        Ok(expr) => match evaluate_path_expression(&expr, &data) {
            Ok(result) => println!("  ✓ Result: {:?}", result),
            Err(e) => println!("  ✗ Error: {}", e),
        },
        Err(e) => println!("  ✗ Parse error: {}", e),
    }

    // 测试越界索引与可选操作符
    let expr_str = ".users[10]?";
    println!("\nTesting: {}", expr_str);
    match parse_path_expression(expr_str) {
        Ok(expr) => match evaluate_path_expression(&expr, &data) {
            Ok(result) => println!("  ✓ Result: {:?}", result),
            Err(e) => println!("  ✗ Error: {}", e),
        },
        Err(e) => println!("  ✗ Parse error: {}", e),
    }
}
