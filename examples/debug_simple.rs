//! 简单的调试功能测试

use serde_json::json;

fn main() {
    println!("=== XQPath v1.4.1 调试功能演示 ===\n");

    // 测试数据
    let data = json!({
        "users": [
            {"name": "Alice", "age": 30, "active": true},
            {"name": "Bob", "age": 25, "active": false},
            {"name": "Carol", "age": 35, "active": true}
        ],
        "meta": {
            "total": 3,
            "version": "1.0"
        }
    });

    let data_str = data.to_string();

    // 1. 测试基础查询功能
    println!("1. 基础查询测试:");
    match xqpath::query!(data_str, ".users[*].name") {
        Ok(names) => {
            println!("   用户名列表: {names:?}");
        }
        Err(e) => {
            println!("   查询失败: {e}");
        }
    }

    // 2. 测试调试功能 (如果启用)
    #[cfg(feature = "debug")]
    {
        println!("\n2. 调试功能测试:");

        // 测试带调试信息的查询
        match xqpath::query_debug!(
            data_str,
            ".users[*].age",
            |debug_info: &xqpath::DebugInfo| {
                println!("   调试信息:");
                if let Some(parse_time) = debug_info.parse_duration {
                    println!("     解析耗时: {parse_time:?}");
                }
                if let Some(exec_time) = debug_info.execution_duration {
                    println!("     执行耗时: {exec_time:?}");
                }
                println!("     执行路径: {}", debug_info.execution_path);
                println!("     查询次数: {}", debug_info.queries_executed);
            }
        ) {
            Ok(ages) => {
                println!("   年龄列表: {ages:?}");
            }
            Err(e) => {
                println!("   调试查询失败: {e}");
            }
        }

        // 测试性能跟踪查询
        println!("\n3. 性能跟踪测试:");
        match xqpath::trace_query!(data_str, ".users[?(@.active)].name") {
            Ok((active_users, stats)) => {
                println!("   活跃用户: {active_users:?}");
                println!("   性能统计:");
                println!("     总耗时: {:?}", stats.duration);
                println!("     内存使用: {} bytes", stats.memory_used);
                println!("     峰值内存: {} bytes", stats.peak_memory);
            }
            Err(e) => {
                println!("   性能跟踪查询失败: {e}");
            }
        }

        // 测试调试上下文
        println!("\n4. 调试上下文测试:");
        let debug_ctx = xqpath::DebugContext::new()
            .with_timing(true)
            .with_memory_tracking(true)
            .with_path_tracing(true)
            .with_log_level(xqpath::LogLevel::Debug);

        println!("   调试上下文配置:");
        println!("     启用计时: {}", debug_ctx.get_config().timing_enabled);
        println!(
            "     启用内存跟踪: {}",
            debug_ctx.get_config().memory_tracking
        );
        println!("     启用路径跟踪: {}", debug_ctx.get_config().path_tracing);
        println!("     日志级别: {:?}", debug_ctx.get_config().log_level);
    }

    #[cfg(not(feature = "debug"))]
    {
        println!("\n2. 调试功能未启用");
        println!("   请使用 --features debug 编译以启用调试功能");
    }

    println!("\n=== 演示完成 ===");
}
