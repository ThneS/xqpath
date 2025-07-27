use serde_json::json;
use xqpath::{evaluate_path_expression, parse_path_expression};

fn main() {
    println!("XQPath v1.2 Phase 4: Error Handling Demo");

    let data = json!({"user": {"name": "Alice", "age": 30}});

    // Try-catch demo
    let expr = parse_path_expression("try .user.nonexistent catch \"default\"")
        .unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    println!("Try-catch result: {result:?}");

    // Optional operator demo
    let expr = parse_path_expression(".user.nonexistent?").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    println!("Optional result: {result:?}");
}
