//! 演示 XQPath 表达式功能的示例

use serde_json::json;
use xqpath::{parse_path_expression, evaluate_path_expression};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 示例数据
    let data = json!({
        "users": [
            {"name": "Alice", "age": 30, "active": true},
            {"name": "Bob", "age": 25, "active": false},
            {"name": "Charlie", "age": 35, "active": true}
        ],
        "metadata": {
            "total": 3,
            "created": "2024-01-15"
        }
    });

    println!("原始数据:");
    println!("{}", serde_json::to_string_pretty(&data)?);
    println!();

    // 示例 1: 简单路径（向后兼容）
    println!("示例 1: 简单路径 `.users[0].name`");
    let expr1 = parse_path_expression(".users[0].name")?;
    let result1 = evaluate_path_expression(&expr1, &data)?;
    println!("结果: {:?}", result1);
    println!();

    // 示例 2: 恒等表达式
    println!("示例 2: 恒等表达式 `.`");
    let expr2 = parse_path_expression(".")?;
    let result2 = evaluate_path_expression(&expr2, &data)?;
    println!("结果类型: {}", result2[0].as_object().unwrap().len());
    println!();

    // 示例 3: 管道操作
    println!("示例 3: 管道操作 `.users | [0] | .name`");
    let expr3 = parse_path_expression(".users | [0] | .name")?;
    let result3 = evaluate_path_expression(&expr3, &data)?;
    println!("结果: {:?}", result3);
    println!();

    // 示例 4: 逗号操作（多选择）
    println!("示例 4: 逗号操作 `.users[*].name, .metadata.total`");
    let expr4 = parse_path_expression(".users[*].name, .metadata.total")?;
    let result4 = evaluate_path_expression(&expr4, &data)?;
    println!("结果: {:?}", result4);
    println!();

    // 示例 5: 复杂表达式（嵌套管道和逗号）
    println!("示例 5: 复杂表达式 `(.users | [*] | .name), (.metadata | .total)`");
    let expr5 = parse_path_expression("(.users | [*] | .name), (.metadata | .total)")?;
    let result5 = evaluate_path_expression(&expr5, &data)?;
    println!("结果: {:?}", result5);
    println!();

    // 示例 6: 字面量与表达式混合
    println!("示例 6: 字面量 `.users[*].name, \"summary\", 42`");
    let expr6 = parse_path_expression(".users[*].name, \"summary\", 42")?;
    let result6 = evaluate_path_expression(&expr6, &data)?;
    println!("结果: {:?}", result6);
    println!();

    // 示例 7: 通配符与管道
    println!("示例 7: 通配符与管道 `.users[*] | .age`");
    let expr7 = parse_path_expression(".users[*] | .age")?;
    let result7 = evaluate_path_expression(&expr7, &data)?;
    println!("结果: {:?}", result7);
    println!();

    Ok(())
}
