// v1.2 Phase 1 示例：内置函数系统演示

use serde_json::json;
use xqpath::{evaluate_path_expression, parse_path_expression};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== XQPath v1.2 Phase 1: 内置函数系统演示 ===\n");

    // 示例数据
    let data = json!({
        "company": {
            "name": "TechCorp",
            "employees": [
                {"name": "Alice", "age": 30, "skills": ["Rust", "Python", "JavaScript"]},
                {"name": "Bob", "age": 25, "skills": ["Go", "Docker"]},
                {"name": "Carol", "age": 35, "skills": ["Java", "Kubernetes", "AWS", "Python"]}
            ],
            "founded": 2010,
            "active": true
        }
    });

    println!("原始数据:");
    println!("{}\n", serde_json::to_string_pretty(&data)?);

    // 1. length 函数演示
    println!("1. length 函数 - 获取长度:");

    // 获取员工数量
    let expr = parse_path_expression(".company.employees | length()")?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   员工数量: {:?}", result);

    // 获取公司名称长度
    let expr = parse_path_expression(".company.name | length()")?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   公司名称长度: {:?}", result);

    // 获取第一个员工的技能数量
    let expr =
        parse_path_expression(".company.employees[0].skills | length()")?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   Alice 的技能数量: {:?}\n", result);

    // 2. type 函数演示
    println!("2. type 函数 - 获取类型:");

    let expr = parse_path_expression(".company.name | type()")?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   公司名称类型: {:?}", result);

    let expr = parse_path_expression(".company.employees | type()")?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   员工列表类型: {:?}", result);

    let expr = parse_path_expression(".company.founded | type()")?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   创立年份类型: {:?}", result);

    let expr = parse_path_expression(".company.active | type()")?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   激活状态类型: {:?}\n", result);

    // 3. keys 函数演示
    println!("3. keys 函数 - 获取键名:");

    let expr = parse_path_expression(".company | keys()")?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   公司对象的键: {:?}", result);

    let expr = parse_path_expression(".company.employees[0] | keys()")?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   第一个员工的键: {:?}\n", result);

    // 4. values 函数演示
    println!("4. values 函数 - 获取所有值:");

    let simple_config =
        json!({"debug": true, "port": 8080, "host": "localhost"});
    let expr = parse_path_expression(". | values()")?;
    let result = evaluate_path_expression(&expr, &simple_config)?;
    println!("   配置对象的所有值: {:?}\n", result);

    // 5. 复杂表达式组合
    println!("5. 复杂表达式组合:");

    // 获取所有员工姓名的总长度
    let expr = parse_path_expression(".company.employees[*].name | length()")?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   所有员工姓名长度: {:?}", result);

    // 使用管道和函数的组合表达式
    let expr = parse_path_expression(
        "(.company | keys() | length()), (.company.employees | length())",
    )?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   公司字段数和员工数: {:?}\n", result);

    // 6. 错误处理演示
    println!("6. 错误处理演示:");

    // 未知函数
    let expr = parse_path_expression("unknown_function()")?;
    match evaluate_path_expression(&expr, &data) {
        Ok(_) => println!("   意外成功"),
        Err(e) => println!("   未知函数错误: {}", e),
    }

    // 参数错误
    let expr = parse_path_expression("length(\"extra_arg\")")?;
    match evaluate_path_expression(&expr, &data) {
        Ok(_) => println!("   意外成功"),
        Err(e) => println!("   参数错误: {}", e),
    }

    // 类型错误
    let expr = parse_path_expression("42 | keys()")?;
    match evaluate_path_expression(&expr, &data) {
        Ok(_) => println!("   意外成功"),
        Err(e) => println!("   类型错误: {}", e),
    }

    println!("\n=== Phase 1 演示完成 ===");
    println!("✅ 实现了 4 个基础内置函数: length, type, keys, values");
    println!("✅ 完整的函数系统架构和错误处理");
    println!("✅ 与现有管道和逗号操作符完美集成");
    println!("\n🚀 接下来: Phase 2 将实现条件表达式和比较操作符！");

    Ok(())
}
