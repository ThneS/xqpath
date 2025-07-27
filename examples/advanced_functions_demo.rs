use serde_json::json;
use xqpath::{evaluate_path_expression, parse_path_expression};

#[allow(clippy::uninlined_format_args)]
fn main() {
    println!("=== XQPath v1.2 Phase 3: 高级函数系统演示 ===\n");

    // 创建测试数据
    let employees_data = json!([
        {"name": "Alice", "age": 30, "department": "Engineering", "salary": 75000},
        {"name": "Bob", "age": 25, "department": "Sales", "salary": 55000},
        {"name": "Charlie", "age": 35, "department": "Engineering", "salary": 85000},
        {"name": "David", "age": 28, "department": "Sales", "salary": 60000},
        {"name": "Eve", "age": 32, "department": "Marketing", "salary": 68000},
        {"name": "Frank", "age": 27, "department": "Engineering", "salary": 72000}
    ]);

    println!("原始数据:");
    println!(
        "{}\n",
        serde_json::to_string_pretty(&employees_data).unwrap()
    );

    // 1. map 函数 - 数据变换
    println!("1. map 函数 - 数据变换:");

    // 提取所有员工姓名
    let result = evaluate_path_expression(
        &parse_path_expression(". | map(.name)").unwrap(),
        &employees_data,
    )
    .unwrap();
    println!("   所有员工姓名: {:?}", result);

    // 提取年龄
    let result = evaluate_path_expression(
        &parse_path_expression(". | map(.age)").unwrap(),
        &employees_data,
    )
    .unwrap();
    println!("   所有员工年龄: {:?}\n", result);

    // 2. select 函数 - 条件过滤
    println!("2. select 函数 - 条件过滤:");

    // 筛选年龄大于30的员工
    let result = evaluate_path_expression(
        &parse_path_expression(". | select(.age > 30)").unwrap(),
        &employees_data,
    )
    .unwrap();
    println!(
        "   年龄大于30的员工数量: {}",
        result[0].as_array().unwrap().len()
    );

    // 筛选工程部门的员工
    let result = evaluate_path_expression(
        &parse_path_expression(". | select(.department == \"Engineering\")")
            .unwrap(),
        &employees_data,
    )
    .unwrap();
    println!(
        "   工程部门员工数量: {}\n",
        result[0].as_array().unwrap().len()
    );

    // 3. sort 和 sort_by 函数 - 排序
    println!("3. sort 和 sort_by 函数 - 排序:");

    // 按年龄排序
    let result = evaluate_path_expression(
        &parse_path_expression(". | sort_by(.age) | map(.name)").unwrap(),
        &employees_data,
    )
    .unwrap();
    println!("   按年龄排序的员工姓名: {:?}", result);

    // 按薪资排序
    let result = evaluate_path_expression(
        &parse_path_expression(". | sort_by(.salary) | map(.name)").unwrap(),
        &employees_data,
    )
    .unwrap();
    println!("   按薪资排序的员工姓名: {:?}\n", result);

    // 4. group_by 函数 - 分组
    println!("4. group_by 函数 - 分组:");

    let result = evaluate_path_expression(
        &parse_path_expression(". | group_by(.department)").unwrap(),
        &employees_data,
    )
    .unwrap();

    if let Some(groups) = result[0].as_array() {
        println!("   按部门分组结果: {} 个组", groups.len());
        for (i, group) in groups.iter().enumerate() {
            if let Some(group_array) = group.as_array() {
                if let Some(first_person) = group_array.first() {
                    let dept = first_person
                        .get("department")
                        .and_then(|d| d.as_str())
                        .unwrap_or("Unknown");
                    println!(
                        "     组 {}: {} 部门 ({} 人)",
                        i + 1,
                        dept,
                        group_array.len()
                    );
                }
            }
        }
    }
    println!();

    // 5. unique 和 unique_by 函数 - 去重
    println!("5. unique 和 unique_by 函数 - 去重:");

    // 提取所有部门并去重
    let result = evaluate_path_expression(
        &parse_path_expression(". | map(.department) | unique()").unwrap(),
        &employees_data,
    )
    .unwrap();
    println!("   所有部门: {:?}", result);

    // 按年龄去重（保留每个年龄的第一个员工）
    let result = evaluate_path_expression(
        &parse_path_expression(". | unique_by(.age) | map(.name)").unwrap(),
        &employees_data,
    )
    .unwrap();
    println!("   每个年龄的代表员工: {:?}\n", result);

    // 6. reverse 函数 - 反转
    println!("6. reverse 函数 - 反转:");

    let result = evaluate_path_expression(
        &parse_path_expression(". | map(.name) | reverse()").unwrap(),
        &employees_data,
    )
    .unwrap();
    println!("   员工姓名反序: {:?}\n", result);

    // 7. 复杂组合操作
    println!("7. 复杂组合操作演示:");

    // 场景：找出工程部门中薪资最高的员工
    let result = evaluate_path_expression(
        &parse_path_expression(". | select(.department == \"Engineering\") | sort_by(.salary) | reverse() | map(.name)").unwrap(),
        &employees_data,
    ).unwrap();
    println!("   工程部门按薪资排序（高到低）: {:?}", result);

    // 场景：统计各部门平均年龄以上的员工
    let result = evaluate_path_expression(
        &parse_path_expression(". | select(.age > 28) | group_by(.department)")
            .unwrap(),
        &employees_data,
    )
    .unwrap();

    if let Some(groups) = result[0].as_array() {
        println!("   年龄大于28的员工按部门分组: {} 个组", groups.len());
    }

    // 场景：获取每个部门薪资最高的员工
    println!("   各部门薪资排序:");
    let departments = ["Engineering", "Sales", "Marketing"];
    for dept in departments {
        let result = evaluate_path_expression(
            &parse_path_expression(&format!(". | select(.department == \"{}\") | sort_by(.salary) | reverse() | map(.name)", dept)).unwrap(),
            &employees_data,
        ).unwrap();
        if let Some(names) = result[0].as_array() {
            if let Some(top_employee) = names.first() {
                println!(
                    "     {} 部门薪资最高: {}",
                    dept,
                    top_employee.as_str().unwrap_or("Unknown")
                );
            }
        }
    }

    println!("\n=== Phase 3 演示完成 ===");
    println!("✅ 实现了 8 个高级内置函数:");
    println!("   • map(expr) - 数据变换映射");
    println!("   • select(condition) - 条件过滤");
    println!("   • sort() - 简单排序");
    println!("   • sort_by(expr) - 按表达式排序");
    println!("   • group_by(expr) - 按表达式分组");
    println!("   • unique() - 去重");
    println!("   • unique_by(expr) - 按表达式去重");
    println!("   • reverse() - 反转");
    println!("✅ 完整的表达式参数支持");
    println!("✅ 与现有管道、条件表达式完美集成");
    println!("✅ 支持复杂的函数组合操作");
    println!("🚀 XQPath v1.2 核心功能已完成！");
}
