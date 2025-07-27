use xqpath::parse_path_expression;

fn main() {
    println!("Testing try-catch parsing...");

    // 测试基本的 try-catch 功能
    match parse_path_expression("try .user.name") {
        Ok(expr) => println!("✓ 'try .user.name' parsed: {}", expr.as_string()),
        Err(e) => println!("✗ 'try .user.name' failed: {}", e),
    }

    match parse_path_expression("try .user.name catch \"default\"") {
        Ok(expr) => println!(
            "✓ 'try .user.name catch \"default\"' parsed: {}",
            expr.as_string()
        ),
        Err(e) => {
            println!("✗ 'try .user.name catch \"default\"' failed: {}", e)
        }
    }

    // 测试可选操作符
    match parse_path_expression(".user.name?") {
        Ok(expr) => println!("✓ '.user.name?' parsed: {}", expr.as_string()),
        Err(e) => println!("✗ '.user.name?' failed: {}", e),
    }

    // 测试数组字面量
    match parse_path_expression("[]") {
        Ok(expr) => println!("✓ '[]' parsed: {}", expr.as_string()),
        Err(e) => println!("✗ '[]' failed: {}", e),
    }

    // 测试简单的乘法表达式
    match parse_path_expression(". * 2") {
        Ok(expr) => println!("✓ '. * 2' parsed: {}", expr.as_string()),
        Err(e) => println!("✗ '. * 2' failed: {}", e),
    }

    // 测试简单的 map 函数调用
    match parse_path_expression("map(.)") {
        Ok(expr) => println!("✓ 'map(.)' parsed: {}", expr.as_string()),
        Err(e) => println!("✗ 'map(.)' failed: {}", e),
    }

    match parse_path_expression("try .data catch []") {
        Ok(expr) => {
            println!("✓ 'try .data catch []' parsed: {}", expr.as_string())
        }
        Err(e) => println!("✗ 'try .data catch []' failed: {}", e),
    }
}
