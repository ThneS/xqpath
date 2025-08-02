//! XQPath v1.4.1 调试功能验证示例

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 XQPath v1.4.1 调试功能验证");
    println!("================================\n");

    let data = r#"{
        "users": [
            {
                "id": 1,
                "name": "Alice",
                "profile": {
                    "age": 30,
                    "email": "alice@example.com"
                }
            },
            {
                "id": 2,
                "name": "Bob",
                "profile": {
                    "age": 25,
                    "email": "bob@example.com"
                }
            }
        ],
        "metadata": {
            "total": 2,
            "timestamp": "2024-01-15T10:30:00Z"
        }
    }"#;

    // 1. 测试基础查询功能（确认基础功能工作）
    println!("1️⃣ 基础查询功能测试");
    println!("-----------------");

    match xqpath::query!(data, ".users[0].name") {
        Ok(result) => println!("   ✅ 基础查询成功: {result:?}"),
        Err(e) => println!("   ❌ 基础查询失败: {e}"),
    }

    // 2. 测试调试模块是否可用
    println!("\n2️⃣ 调试模块可用性测试");
    println!("--------------------");

    #[cfg(feature = "debug")]
    {
        use xqpath::{DebugContext, Logger, LoggerConfig};

        let _debug_ctx = DebugContext::new();
        println!("   ✅ DebugContext 创建成功");

        let config = LoggerConfig::default();
        let _logger = Logger::new(config);
        println!("   ✅ Logger 创建成功");

        println!("   � 调试功能已启用并可正常使用");
    }

    #[cfg(not(feature = "debug"))]
    {
        println!("   ⚠️  调试功能未启用（需要 --features debug）");
    }

    println!("\n🎉 XQPath v1.4.1 调试功能验证完成！");
    println!("核心调试基础设施已就绪。");

    Ok(())
}
