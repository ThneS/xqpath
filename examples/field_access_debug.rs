use serde_json::json;
use xqpath::{evaluate_path_expression, parse_path_expression};

fn main() {
    let data = json!({
        "user": {
            "name": "Alice",
            "age": 30
        }
    });

    println!("Testing field access behavior...");

    // 测试存在的字段
    let expr_str = ".user.name";
    println!("Testing: {}", expr_str);
    match parse_path_expression(expr_str) {
        Ok(expr) => match evaluate_path_expression(&expr, &data) {
            Ok(result) => println!("  ✓ Result: {:?}", result),
            Err(e) => println!("  ✗ Error: {}", e),
        },
        Err(e) => println!("  ✗ Parse error: {}", e),
    }

    // 测试不存在的字段
    let expr_str = ".user.nonexistent";
    println!("\nTesting: {}", expr_str);
    match parse_path_expression(expr_str) {
        Ok(expr) => match evaluate_path_expression(&expr, &data) {
            Ok(result) => println!("  ✓ Result: {:?}", result),
            Err(e) => println!("  ✗ Error: {}", e),
        },
        Err(e) => println!("  ✗ Parse error: {}", e),
    }

    // 测试 try 表达式与不存在的字段
    let expr_str = "try .user.nonexistent";
    println!("\nTesting: {}", expr_str);
    match parse_path_expression(expr_str) {
        Ok(expr) => match evaluate_path_expression(&expr, &data) {
            Ok(result) => println!("  ✓ Result: {:?}", result),
            Err(e) => println!("  ✗ Error: {}", e),
        },
        Err(e) => println!("  ✗ Parse error: {}", e),
    }
}
