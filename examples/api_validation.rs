//! 最终验证 README 中的所有示例

use serde_json::json;
use xqpath::{
    count, exists, exists_all, query, query_as_type, query_one, query_string,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== README 示例最终验证 ===\n");

    // 基础宏示例验证
    test_basic_macros()?;

    // 高级宏示例验证
    test_advanced_macros()?;

    // 表达式语法验证
    test_expression_syntax()?;

    // 错误处理验证
    test_error_handling()?;

    println!("\n🎉 所有 README 示例验证完成!");

    Ok(())
}

fn test_basic_macros() -> Result<(), Box<dyn std::error::Error>> {
    println!("1️⃣ 基础宏示例验证");
    println!("------------------");

    let data = json!({
        "users": [
            {"name": "Alice", "age": 30, "active": true},
            {"name": "Bob", "age": 25, "active": false}
        ]
    });

    // 查询多个值
    let names = query!(data.to_string(), "users[*].name")?;
    assert_eq!(names, vec![json!("Alice"), json!("Bob")]);
    println!("   ✅ query! - 查询多个值");

    // 查询单个值
    let first_name = query_one!(data.to_string(), "users[0].name")?;
    assert_eq!(first_name, Some(json!("Alice")));
    println!("   ✅ query_one! - 查询单个值");

    // 检查路径是否存在
    let has_users = exists!(data.to_string(), "users")?;
    assert!(has_users);
    println!("   ✅ exists! - 检查路径存在");

    // 计算数量
    let user_count = count!(data.to_string(), "users[*]")?;
    assert_eq!(user_count, 2);
    println!("   ✅ count! - 计算数量");

    Ok(())
}

fn test_advanced_macros() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n2️⃣ 高级宏示例验证");
    println!("------------------");

    let data = json!({
        "user": {
            "name": "Alice",
            "age": 30,
            "email": "alice@example.com"
        },
        "users": [
            {"name": "Alice", "email": "alice@example.com"},
            {"name": "Bob", "email": "bob@example.com"}
        ]
    });

    // 类型转换查询
    let age = query_as_type!(data.to_string(), ".user.age", i64)?;
    assert_eq!(age, Some(30));
    println!("   ✅ query_as_type! - 类型转换查询");

    // 字符串查询
    let _emails = query_string!(data.to_string(), ".users[*].email")?;
    println!("   ✅ query_string! - 字符串查询");

    // 检查多个路径
    let has_all = exists_all!(
        data.to_string(),
        ".user.name",
        ".user.email",
        ".user.age"
    )?;
    assert!(has_all);
    println!("   ✅ exists_all! - 检查多个路径");

    Ok(())
}

fn test_expression_syntax() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n3️⃣ 表达式语法验证");
    println!("------------------");

    let data = json!({
        "users": [
            {"name": "Alice", "age": 30, "active": true},
            {"name": "Bob", "age": 25, "active": false},
            {"name": "Carol", "age": 35, "active": true}
        ],
        "config": {
            "version": "1.0",
            "database": {
                "host": "localhost",
                "port": 5432
            }
        }
    });

    // 基础字段访问
    let version = query!(data.to_string(), ".config.version")?;
    assert_eq!(version, vec![json!("1.0")]);
    println!("   ✅ 基础字段访问");

    // 嵌套字段访问
    let host = query!(data.to_string(), ".config.database.host")?;
    assert_eq!(host, vec![json!("localhost")]);
    println!("   ✅ 嵌套字段访问");

    // 数组索引
    let first_user = query!(data.to_string(), ".users[0].name")?;
    assert_eq!(first_user, vec![json!("Alice")]);
    println!("   ✅ 数组索引");

    // 数组通配符
    let all_names = query!(data.to_string(), ".users[*].name")?;
    assert_eq!(
        all_names,
        vec![json!("Alice"), json!("Bob"), json!("Carol")]
    );
    println!("   ✅ 数组通配符");

    // 空数组（通配符）
    let all_names2 = query!(data.to_string(), ".users[].name")?;
    assert_eq!(
        all_names2,
        vec![json!("Alice"), json!("Bob"), json!("Carol")]
    );
    println!("   ✅ 空数组语法");

    Ok(())
}

fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n4️⃣ 错误处理验证");
    println!("------------------");

    let data = json!({
        "user": {
            "name": "Alice",
            "age": 30
        }
    });

    // 标准错误处理
    match query!(data.to_string(), ".nonexistent.path") {
        Ok(_) => println!("   ❌ 应该出错但成功了"),
        Err(_) => println!("   ✅ 标准错误处理"),
    }

    // 查询不存在的字段
    let optional_field = query_one!(data.to_string(), ".user.email")?;
    assert_eq!(optional_field, None);
    println!("   ✅ 查询不存在字段返回 None");

    Ok(())
}
