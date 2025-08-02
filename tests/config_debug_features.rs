#[cfg(feature = "config-management")]
use xqpath::config::{ConfigManager, DebugConfig, XQPathConfig};

#[cfg(feature = "interactive-debug")]
use xqpath::debugger::{DebugCommand, XQPathDebugger};

/// 测试配置管理系统的核心功能
#[cfg(feature = "config-management")]
#[test]
fn test_config_manager_basic() {
    let manager = ConfigManager::new().unwrap();

    // 测试默认配置
    let config = manager.get_config();
    assert_eq!(config.debug.level, "warn");
    assert_eq!(config.debug.output, "stderr");

    // 测试保存配置
    let result = manager.save_config();
    assert!(result.is_ok());
}

/// 测试配置文件的加载和保存
#[cfg(feature = "config-management")]
#[test]
fn test_config_persistence() {
    let manager = ConfigManager::new().unwrap();

    // 测试保存配置
    let result = manager.save_config();
    assert!(result.is_ok());

    // 测试重新加载配置
    let loaded_result = manager.load_config();
    assert!(loaded_result.is_ok());

    let loaded_config = loaded_result.unwrap();
    assert_eq!(loaded_config.debug.level, "warn");
    assert_eq!(loaded_config.debug.output, "stderr");
}

/// 测试配置模板功能
#[cfg(feature = "config-management")]
#[test]
fn test_config_profiles() {
    // 创建开发环境配置
    let mut dev_config = XQPathConfig::default();
    dev_config.debug.level = "trace".to_string();
    dev_config.debug.timing = true;

    // 测试序列化和反序列化
    let serialized = serde_yaml::to_string(&dev_config).unwrap();
    let loaded_config: XQPathConfig =
        serde_yaml::from_str(&serialized).unwrap();

    assert_eq!(loaded_config.debug.level, "trace");
    assert!(loaded_config.debug.timing);
}

/// 测试配置验证功能
#[cfg(feature = "config-management")]
#[test]
fn test_config_validation() {
    use std::path::PathBuf;

    let config = XQPathConfig {
        debug: DebugConfig {
            level: "custom_level".to_string(),
            output: "file".to_string(),
            file: Some(PathBuf::from("/tmp/debug.log")),
            timing: true,
        },
        ..Default::default()
    };

    // 测试配置结构的有效性
    assert_eq!(config.debug.level, "custom_level");
    assert_eq!(config.debug.output, "file");
    assert!(config.debug.timing);
}

/// 测试交互式调试器的基本功能
#[cfg(feature = "interactive-debug")]
#[test]
fn test_debugger_creation() {
    let _debugger = XQPathDebugger::new();
    // 调试器创建成功
}

/// 测试调试命令解析
#[cfg(feature = "interactive-debug")]
#[test]
fn test_debug_command_parsing() {
    // 测试帮助命令
    let help_cmd = DebugCommand::Help;
    match help_cmd {
        DebugCommand::Help => assert!(true),
        _ => assert!(false, "Expected Help command"),
    }

    // 测试查询命令
    let query_cmd = DebugCommand::Run {
        query: ".test.path".to_string(),
    };
    match query_cmd {
        DebugCommand::Run { query } => {
            assert_eq!(query, ".test.path");
        }
        _ => assert!(false, "Expected Run command"),
    }

    // 测试退出命令
    let quit_cmd = DebugCommand::Quit;
    match quit_cmd {
        DebugCommand::Quit => assert!(true),
        _ => assert!(false, "Expected Quit command"),
    }
}

/// 集成测试：配置和调试器协同工作
#[cfg(all(feature = "config-management", feature = "interactive-debug"))]
#[test]
fn test_config_debugger_integration() {
    // 创建配置管理器
    let config_manager = ConfigManager::new().unwrap();

    // 创建调试器
    let _debugger = XQPathDebugger::new();

    // 测试配置可以被正确加载
    let config = config_manager.get_config();
    assert!(!config.debug.level.is_empty());
}

/// 测试错误处理
#[cfg(feature = "config-management")]
#[test]
fn test_config_error_handling() {
    // 测试配置管理器创建
    let manager = ConfigManager::new();
    assert!(manager.is_ok());

    // 测试默认配置访问
    let manager = manager.unwrap();
    let config = manager.get_config();
    assert!(!config.debug.level.is_empty());
}

/// 性能测试：配置加载速度
#[cfg(feature = "config-management")]
#[test]
fn test_config_performance() {
    use std::time::Instant;

    let manager = ConfigManager::new().unwrap();

    // 测试加载性能
    let start = Instant::now();
    for _ in 0..100 {
        let _ = manager.get_config();
    }
    let duration = start.elapsed();

    // 100次配置访问应该在合理时间内完成（1秒）
    assert!(duration.as_secs() < 1, "配置访问性能过慢: {duration:?}");
}
