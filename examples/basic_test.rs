use xqpath::*;

fn main() {
    println!("测试基础功能...");

    let data = r#"{"name": "test", "value": 42}"#;

    // 基础查询
    let result = query!(data, ".name");
    println!("基础查询结果: {result:?}");

    // 检查是否有调试功能
    #[cfg(feature = "debug")]
    println!("调试功能已启用");

    #[cfg(not(feature = "debug"))]
    println!("调试功能未启用");

    println!("测试完成");
}
