use serde_json::json;
use xqpath::{evaluate_path_expression, parse_path_expression};

fn main() {
    let data = json!({
        "data": [1, 2, 3]
    });

    println!("Testing error handling expressions...");

    // 测试简单的表达式
    let expr_str = "try (.data | length?) catch 0";
    println!("Testing: {}", expr_str);

    match parse_path_expression(expr_str) {
        Ok(expr) => {
            println!("  ✓ Parsed: {}", expr.as_string());
            match evaluate_path_expression(&expr, &data) {
                Ok(result) => println!("  ✓ Result: {:?}", result),
                Err(e) => println!("  ✗ Eval error: {}", e),
            }
        }
        Err(e) => println!("  ✗ Parse error: {}", e),
    }

    // 测试更复杂的表达式
    let expr_str = "try .data.nonexistent?.length? catch (.data | length)";
    println!("\nTesting: {}", expr_str);

    match parse_path_expression(expr_str) {
        Ok(expr) => {
            println!("  ✓ Parsed: {}", expr.as_string());
            match evaluate_path_expression(&expr, &data) {
                Ok(result) => println!("  ✓ Result: {:?}", result),
                Err(e) => println!("  ✗ Eval error: {}", e),
            }
        }
        Err(e) => println!("  ✗ Parse error: {}", e),
    }

    // 测试基础的可选操作符
    let expr_str = ".data.nonexistent?";
    println!("\nTesting: {}", expr_str);

    match parse_path_expression(expr_str) {
        Ok(expr) => {
            println!("  ✓ Parsed: {}", expr.as_string());
            match evaluate_path_expression(&expr, &data) {
                Ok(result) => println!("  ✓ Result: {:?}", result),
                Err(e) => println!("  ✗ Eval error: {}", e),
            }
        }
        Err(e) => println!("  ✗ Parse error: {}", e),
    }
}
