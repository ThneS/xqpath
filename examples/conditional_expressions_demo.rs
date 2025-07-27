use serde_json::json;
use xqpath::{evaluate_path_expression, parse_path_expression};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== XQPath v1.2 Phase 2: 条件表达式和比较操作符演示 ===\n");

    // 示例数据
    let data = json!({
        "users": [
            {
                "name": "Alice",
                "age": 30,
                "active": true,
                "score": 95,
                "department": "Engineering"
            },
            {
                "name": "Bob",
                "age": 17,
                "active": false,
                "score": 88,
                "department": "Marketing"
            },
            {
                "name": "Carol",
                "age": 35,
                "active": true,
                "score": 92,
                "department": "Engineering"
            }
        ],
        "config": {
            "min_age": 18,
            "min_score": 90,
            "departments": ["Engineering", "Marketing", "Sales"]
        }
    });

    println!("原始数据:");
    println!("{}\n", serde_json::to_string_pretty(&data)?);

    // 1. 比较操作符演示
    println!("1. 比较操作符演示:");

    // 数值比较
    let expr = parse_path_expression(".users[0].age > 25")?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   Alice 年龄 > 25: {:?}", result);

    let expr = parse_path_expression(".users[1].age < .config.min_age")?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   Bob 年龄 < 最小年龄: {:?}", result);

    // 字符串比较
    let expr = parse_path_expression(".users[0].name == \"Alice\"")?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   Alice 的名字是 'Alice': {:?}", result);

    // 分数比较
    let expr = parse_path_expression(".users[0].score >= .config.min_score")?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   Alice 分数 >= 最小分数: {:?}\n", result);

    // 2. 逻辑操作符演示
    println!("2. 逻辑操作符演示:");

    // and 操作
    let expr =
        parse_path_expression(".users[0].age > 18 and .users[0].active")?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   Alice 成年且活跃: {:?}", result);

    // or 操作
    let expr =
        parse_path_expression(".users[1].age < 18 or .users[1].score > 85")?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   Bob 未成年或分数 > 85: {:?}", result);

    // not 操作
    let expr = parse_path_expression("not .users[1].active")?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   Bob 不活跃: {:?}\n", result);

    // 3. 条件表达式演示
    println!("3. 条件表达式演示:");

    // 简单条件表达式
    let expr = parse_path_expression(
        "if .users[0].age >= 18 then \"adult\" else \"minor\" end",
    )?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   Alice 年龄分类: {:?}", result);

    let expr = parse_path_expression(
        "if .users[1].age >= 18 then \"adult\" else \"minor\" end",
    )?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   Bob 年龄分类: {:?}", result);

    // 复杂条件表达式
    let expr = parse_path_expression(
        "if .users[0].active and .users[0].score >= 90 then \"excellent\" else \"good\" end"
    )?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   Alice 表现评级: {:?}", result);

    // 嵌套条件
    let expr = parse_path_expression(
        "if .users[2].age > 30 then (if .users[2].score > 90 then \"senior_excellent\" else \"senior_good\" end) else \"junior\" end"
    )?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   Carol 综合评级: {:?}\n", result);

    // 4. 与内置函数结合使用
    println!("4. 与内置函数结合的复合表达式:");

    // 条件与函数调用结合
    let expr = parse_path_expression(
        "if (.users | length()) > 2 then \"large_team\" else \"small_team\" end"
    )?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   团队规模分类: {:?}", result);

    // 比较与函数调用结合
    let expr = parse_path_expression(
        "(.users[0].name | length()) > (.users[1].name | length())",
    )?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   Alice 名字比 Bob 长: {:?}", result);

    // 逻辑与函数调用结合
    let expr = parse_path_expression(
        "(.config | keys() | length()) > 2 and (.users | length()) >= 3",
    )?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   配置项多且用户充足: {:?}\n", result);

    // 5. 复杂查询示例
    println!("5. 复杂查询示例:");

    // 查找符合条件的用户（模拟查询）
    let expr = parse_path_expression(
        "if .users[0].age >= .config.min_age and .users[0].score >= .config.min_score then .users[0].name else null end"
    )?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   符合条件的用户1: {:?}", result);

    let expr = parse_path_expression(
        "if .users[1].age >= .config.min_age and .users[1].score >= .config.min_score then .users[1].name else null end"
    )?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   符合条件的用户2: {:?}", result);

    let expr = parse_path_expression(
        "if .users[2].age >= .config.min_age and .users[2].score >= .config.min_score then .users[2].name else null end"
    )?;
    let result = evaluate_path_expression(&expr, &data)?;
    println!("   符合条件的用户3: {:?}\n", result);

    println!("=== Phase 2 演示完成 ===");
    println!("✅ 实现了比较操作符: ==, !=, <, <=, >, >=");
    println!("✅ 实现了逻辑操作符: and, or, not");
    println!("✅ 实现了条件表达式: if-then-else-end");
    println!("✅ 支持复杂嵌套和函数组合");
    println!("🚀 接下来: Phase 3 将实现更多高级函数和数组操作！");

    Ok(())
}
