/// XQPath v1.2 Phase 4: 错误处理和优化 - 演示程序
///
/// 本演示展示了 try-catch 表达式和可选操作符的功能
use serde_json::json;
use xqpath::{evaluate_path_expression, parse_path_expression};

fn main() {
    println!("🔧 XQPath v1.2 Phase 4: 错误处理和优化演示");
    println!("===============================================\n");

    let data = json!({
        "user": {
            "name": "Alice",
            "age": 30,
            "profile": {
                "email": "alice@example.com"
            }
        },
        "posts": [
            {"title": "Hello World", "views": 150},
            {"title": "XQPath Tutorial", "views": 300},
            {"title": "Advanced Queries", "views": 200}
        ],
        "config": {
            "theme": "dark",
            "notifications": true
        }
    });

    println!("📊 测试数据:");
    println!("{}\n", serde_json::to_string_pretty(&data).unwrap());

    // 1. try-catch 表达式演示
    println!("1️⃣ try-catch 表达式演示");
    println!("========================");

    demo_expression("try .user.name", &data, "获取存在的字段");
    demo_expression(
        "try .user.nonexistent",
        &data,
        "获取不存在的字段（无 catch）",
    );
    demo_expression(
        "try unknown_function() catch \"fallback\"",
        &data,
        "函数调用失败的错误处理",
    );
    demo_expression(
        "try .posts | length() catch 0",
        &data,
        "复杂表达式的错误处理",
    );

    println!();

    // 2. 可选操作符演示
    println!("2️⃣ 可选操作符演示");
    println!("==================");

    demo_expression(".user.name?", &data, "获取存在的字段（可选）");
    demo_expression(".user.nonexistent?", &data, "获取不存在的字段（可选）");
    demo_expression(".posts[0]?", &data, "数组索引访问（可选）");
    demo_expression(".posts[10]?", &data, "数组越界访问（可选）");
    demo_expression(".posts | length()?", &data, "函数调用（可选）");

    println!();

    // 3. 组合使用演示
    println!("3️⃣ 组合使用演示");
    println!("================");

    demo_expression(
        "try .user.profile.email catch .user.email?",
        &data,
        "try-catch 与可选操作符组合",
    );
    demo_expression(
        "try (.posts | map(select(.views > 250))?) catch []",
        &data,
        "复杂表达式组合",
    );
    demo_expression(
        "if (.config.notifications?) then \"启用\" else \"禁用\" end",
        &data,
        "条件表达式中的可选操作符",
    );

    println!();

    // 4. 实际应用场景
    println!("4️⃣ 实际应用场景");
    println!("================");

    let api_response = json!({
        "status": "success",
        "data": {
            "users": [
                {"id": 1, "name": "Alice", "email": "alice@example.com"},
                {"id": 2, "name": "Bob"},  // 缺少 email
                {"id": 3, "name": "Charlie", "email": "charlie@example.com"}
            ]
        }
    });

    println!("📡 API 响应数据:");
    println!("{}\n", serde_json::to_string_pretty(&api_response).unwrap());

    demo_expression(
        "try .data.users catch []",
        &api_response,
        "安全获取用户列表",
    );
    demo_expression(
        ".data.users[1].email?",
        &api_response,
        "安全获取可能不存在的邮箱",
    );
    demo_expression(
        "try .data.users | map(.name) catch []",
        &api_response,
        "提取所有用户名",
    );
    demo_expression(
        "try .data.users | map(.email?) catch []",
        &api_response,
        "提取所有邮箱（包括缺失的）",
    );

    println!();

    // 5. 错误处理最佳实践
    println!("5️⃣ 错误处理最佳实践");
    println!("====================");

    println!("✅ 推荐做法:");
    println!("  • 使用 try-catch 处理可能失败的函数调用");
    println!("  • 使用可选操作符 (?) 处理可能不存在的字段");
    println!("  • 在复杂查询中提供合理的默认值");
    println!("  • 结合条件表达式进行更精细的控制");

    println!("\n⚠️  注意事项:");
    println!("  • 字段不存在返回空数组，不会触发 try-catch");
    println!("  • 可选操作符会将空结果转换为 null");
    println!("  • try-catch 主要用于函数调用和真正的错误情况");

    println!("\n🎯 错误处理功能总结:");
    println!("  • ✅ try-catch 表达式");
    println!("  • ✅ 可选操作符 (?)");
    println!("  • ✅ 友好的错误信息");
    println!("  • ✅ 与现有功能无缝集成");
    println!("  • ✅ 数组和对象字面量支持");
}

fn demo_expression(
    expr_str: &str,
    data: &serde_json::Value,
    description: &str,
) {
    println!("🔍 {}", description);
    println!("   表达式: {}", expr_str);

    match parse_path_expression(expr_str) {
        Ok(expr) => match evaluate_path_expression(&expr, data) {
            Ok(result) => {
                if result.is_empty() {
                    println!("   结果: [] (空结果)");
                } else {
                    println!("   结果: {:?}", result);
                }
            }
            Err(e) => println!("   ❌ 求值错误: {}", e),
        },
        Err(e) => println!("   ❌ 解析错误: {}", e),
    }
    println!();
}
