use xqpath::parse_path_expression;

fn main() {
    // 测试简单的函数调用解析
    match parse_path_expression("map(.)") {
        Ok(expr) => println!("Parsed successfully: {:?}", expr),
        Err(e) => println!("Parse error: {:?}", e),
    }
    
    match parse_path_expression("map(. * 2)") {
        Ok(expr) => println!("Parsed successfully: {:?}", expr),
        Err(e) => println!("Parse error: {:?}", e),
    }
}
