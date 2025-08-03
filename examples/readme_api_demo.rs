// README API 示例验证
// 将此文件添加到examples目录作为可执行示例

use xqpath::{exists, query, query_one};

fn main() {
    println!("🚀 XQPath README API 示例验证");
    println!("{}", "=".repeat(50));

    let data = r#"{
      "users": [
        {"name": "Alice", "age": 30},
        {"name": "Bob", "age": 25}
      ]
    }"#;

    // 示例1: 查询多个值
    println!("\n📊 示例1: 查询多个值");
    println!("代码: query!(data, \"users[*].name\")");
    match query!(data, "users[*].name") {
        Ok(names) => {
            println!("✅ 结果: {:?}", names);
            println!("📝 说明: 成功提取所有用户名称");
        }
        Err(e) => println!("❌ 错误: {}", e),
    }

    // 示例2: 查询单个值
    println!("\n📊 示例2: 查询单个值");
    println!("代码: query_one!(data, \"users[0].name\")");
    match query_one!(data, "users[0].name") {
        Ok(first_name) => {
            println!("✅ 结果: {:?}", first_name);
            println!("📝 说明: 成功提取第一个用户名称");
        }
        Err(e) => println!("❌ 错误: {}", e),
    }

    // 示例3: 检查路径是否存在
    println!("\n📊 示例3: 检查路径存在性");
    println!("代码: exists!(data, \"users\")");
    match exists!(data, "users") {
        Ok(has_users) => {
            println!("✅ 结果: {}", has_users);
            println!("📝 说明: 路径存在检查成功");
        }
        Err(e) => println!("❌ 错误: {}", e),
    }

    // 额外验证: 错误处理
    println!("\n📊 额外验证: 错误处理");
    println!("代码: query!(data, \".some.nonexistent\")");
    match query!(data, ".some.nonexistent") {
        Ok(result) => println!("结果: {:?}", result),
        Err(e) => {
            println!("✅ 错误处理正常: {}", e);
            println!("📝 说明: 优雅处理不存在的路径");
        }
    }

    println!("\n🎉 所有README API示例验证完成！");
}
