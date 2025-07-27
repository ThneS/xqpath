//! XQPath v1.1 新表达式 API 的集成示例
//! 展示如何在现有的 extractor API 中集成新的表达式功能

use serde_json::{json, Value};
use xqpath::{
    evaluate_path_expression, extractor::extract, parse_path,
    parse_path_expression,
};

/// 扩展的提取器，支持新的表达式语法
pub struct ExtendedExtractor;

impl ExtendedExtractor {
    /// 使用新的表达式语法提取值
    pub fn extract_with_expression(
        expression: &str,
        value: &Value,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let parsed_expr = parse_path_expression(expression)?;
        let results = evaluate_path_expression(&parsed_expr, value)?;
        Ok(results)
    }

    /// 向后兼容的提取函数，自动检测语法类型
    pub fn extract_auto(
        path_or_expression: &str,
        value: &Value,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        // 如果包含管道或逗号操作符，使用新的表达式解析器
        if path_or_expression.contains('|') || path_or_expression.contains(',')
        {
            Self::extract_with_expression(path_or_expression, value)
        } else {
            // 使用现有的简单路径提取器
            let parsed_path = parse_path(path_or_expression)?;
            let results = extract(value, &parsed_path)?;
            // 转换 &Value 到 Value
            let owned_results: Vec<Value> =
                results.into_iter().cloned().collect();
            Ok(owned_results)
        }
    }

    /// 批量提取多个表达式
    pub fn extract_multiple(
        expressions: &[&str],
        value: &Value,
    ) -> Result<Vec<Vec<Value>>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        for expr in expressions {
            let result = Self::extract_auto(expr, value)?;
            results.push(result);
        }

        Ok(results)
    }

    /// 提取并合并所有结果
    pub fn extract_merged(
        expressions: &[&str],
        value: &Value,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let mut all_results = Vec::new();

        for expr in expressions {
            let results = Self::extract_auto(expr, value)?;
            all_results.extend(results);
        }

        Ok(all_results)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 测试数据
    let data = json!({
        "company": {
            "name": "TechCorp",
            "employees": [
                {
                    "id": 1,
                    "name": "Alice Johnson",
                    "department": "Engineering",
                    "salary": 75000,
                    "skills": ["Rust", "Python", "Docker"]
                },
                {
                    "id": 2,
                    "name": "Bob Smith",
                    "department": "Marketing",
                    "salary": 60000,
                    "skills": ["SEO", "Content", "Analytics"]
                },
                {
                    "id": 3,
                    "name": "Carol Davis",
                    "department": "Engineering",
                    "salary": 80000,
                    "skills": ["JavaScript", "React", "Node.js"]
                }
            ],
            "departments": {
                "Engineering": {"budget": 500000, "head": "Alice Johnson"},
                "Marketing": {"budget": 200000, "head": "Bob Smith"}
            }
        }
    });

    println!("=== XQPath v1.1 表达式功能演示 ===\n");

    // 演示 1: 向后兼容的简单路径
    println!("1. 向后兼容 - 公司名称:");
    let result1 = ExtendedExtractor::extract_auto(".company.name", &data)?;
    println!("   结果: {result1:?}\n");

    // 演示 2: 新的管道语法
    println!("2. 管道操作 - 员工姓名:");
    let result2 = ExtendedExtractor::extract_auto(
        ".company.employees | [*] | .name",
        &data,
    )?;
    println!("   结果: {result2:?}\n");

    // 演示 3: 逗号操作 - 多字段提取
    println!("3. 逗号操作 - 公司信息:");
    let result3 = ExtendedExtractor::extract_auto(
        ".company.name, .company.employees | length",
        &data,
    )?;
    println!("   结果: {result3:?}\n");

    // 演示 4: 复杂表达式 - 工程部门员工信息
    println!("4. 复杂表达式 - 所有员工薪资:");
    // 注意：select 功能需要进一步实现，这里用简化版本
    let result4 = ExtendedExtractor::extract_auto(
        ".company.employees | [*] | .salary",
        &data,
    )?;
    println!("   所有员工薪资: {result4:?}\n");

    // 演示 5: 批量提取
    println!("5. 批量提取多个表达式:");
    let expressions = vec![
        ".company.name",
        ".company.employees | [*] | .name",
        ".company.departments | * | .budget",
    ];
    let results5 = ExtendedExtractor::extract_multiple(&expressions, &data)?;
    for (i, result) in results5.iter().enumerate() {
        println!("   表达式 {}: {:?}", i + 1, result);
    }
    println!();

    // 演示 6: 合并提取结果
    println!("6. 合并提取结果:");
    let merged = ExtendedExtractor::extract_merged(&expressions, &data)?;
    println!("   合并结果: {merged:?}\n");

    // 演示 7: 错误处理
    println!("7. 错误处理演示:");
    match ExtendedExtractor::extract_auto(".nonexistent.field", &data) {
        Ok(result) => println!("   结果: {result:?}"),
        Err(e) => println!("   预期的错误: {e}"),
    }

    // 演示语法检测
    println!("\n8. 自动语法检测:");
    let test_expressions = vec![
        ".simple.path",               // 简单路径
        ".path | .field",             // 管道表达式
        ".field1, .field2",           // 逗号表达式
        "(.complex | .pipe), .field", // 复杂表达式
    ];

    for expr in test_expressions {
        let has_operators = expr.contains('|') || expr.contains(',');
        println!(
            "   '{}' -> {}",
            expr,
            if has_operators {
                "新表达式语法"
            } else {
                "传统路径语法"
            }
        );
    }

    println!("\n=== 演示完成 ===");
    Ok(())
}
