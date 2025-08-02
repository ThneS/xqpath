use serde_json::json;
use xqpath::{exists, query, query_one};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 验证README中的Rust库示例");

    let data = r#"{
        "users": [
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ]
    }"#;

    // 查询多个值
    println!("\n1. 测试 query! 宏:");
    let names = query!(data, "users[*].name")?;
    println!("   结果: {:?}", names);
    assert_eq!(names.len(), 2);
    assert_eq!(names[0], json!("Alice"));
    assert_eq!(names[1], json!("Bob"));

    // 查询单个值
    println!("\n2. 测试 query_one! 宏:");
    let first_name = query_one!(data, "users[0].name")?;
    println!("   结果: {:?}", first_name);
    assert!(first_name.is_some());
    assert_eq!(first_name.unwrap(), json!("Alice"));

    // 检查路径是否存在
    println!("\n3. 测试 exists! 宏:");
    let has_users = exists!(data, "users")?;
    println!("   结果: {}", has_users);
    assert_eq!(has_users, true);

    let has_email = exists!(data, "users[0].email")?;
    println!("   users[0].email 存在: {}", has_email);
    assert_eq!(has_email, false);

    println!("\n✅ 所有库功能测试通过！");
    Ok(())
}
