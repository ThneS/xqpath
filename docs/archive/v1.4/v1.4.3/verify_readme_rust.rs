use xqpath::{exists, query, query_one};

fn main() {
    let data = r#"{
      "users": [
        {"name": "Alice", "age": 30},
        {"name": "Bob", "age": 25}
      ]
    }"#;

    // 验证README中的示例1: 查询多个值
    println!("验证 query! 宏:");
    let names = query!(data, "users[*].name").unwrap();
    println!("结果: {:?}", names);
    // 应该返回: [String("Alice"), String("Bob")]

    // 验证README中的示例2: 查询单个值
    println!("\n验证 query_one! 宏:");
    let first_name = query_one!(data, "users[0].name").unwrap();
    println!("结果: {:?}", first_name);
    // 应该返回: Some(String("Alice"))

    // 验证README中的示例3: 检查路径是否存在
    println!("\n验证 exists! 宏:");
    let has_users = exists!(data, "users").unwrap();
    println!("结果: {}", has_users);
    // 应该返回: true

    // 额外验证：不存在的路径
    let has_nonexistent = exists!(data, "nonexistent").unwrap();
    println!("不存在路径的结果: {}", has_nonexistent);
    // 应该返回: false

    println!("\n✅ 所有README Rust API示例验证完成！");
}
