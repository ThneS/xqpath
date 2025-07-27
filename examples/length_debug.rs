use serde_json::json;
use xqpath::{evaluate_path_expression, parse_path_expression};

fn main() {
    let data = json!({
        "data": [1, 2, 3]
    });

    println!("Testing length function...");

    // 测试基本的 length 函数
    let expr_str = ".data | length";
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

    // 测试 length 函数与可选操作符
    let expr_str = ".data | length?";
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

    // 测试不存在的字段与 length
    let expr_str = ".nonexistent | length";
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
