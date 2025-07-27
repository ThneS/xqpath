use serde_json::json;
use xqpath::{datapath_count, datapath_exists, datapath_get, datapath_type};

#[cfg(feature = "update")]
use xqpath::datapath_set;

/// 测试基本的路径提取功能
#[test]
fn test_basic_field_extraction() {
    let json_data = r#"
    {
        "user": {
            "name": "Alice",
            "age": 30,
            "email": "alice@example.com"
        }
    }
    "#;

    let result = datapath_get!(json_data, "user.name").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], json!("Alice"));
}

/// 测试数组索引访问
#[test]
fn test_array_index_access() {
    let json_data = r#"
    {
        "users": [
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25},
            {"name": "Charlie", "age": 35}
        ]
    }
    "#;

    let result = datapath_get!(json_data, "users[1].name").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], json!("Bob"));
}

/// 测试通配符功能
#[test]
fn test_wildcard_matching() {
    let json_data = r#"
    {
        "users": [
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ]
    }
    "#;

    let result = datapath_get!(json_data, "users[*].name").unwrap();
    assert_eq!(result.len(), 2);
    assert!(result.contains(&json!("Alice")));
    assert!(result.contains(&json!("Bob")));
}

/// 测试 YAML 格式支持
#[test]
fn test_yaml_support() {
    let yaml_data = r#"
user:
  name: Alice
  age: 30
  addresses:
    - street: 123 Main St
      city: Anytown
    - street: 456 Oak Ave
      city: Another City
"#;

    let result = datapath_get!(yaml_data, "user.name").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], json!("Alice"));

    let addresses = datapath_get!(yaml_data, "user.addresses[*].city").unwrap();
    assert_eq!(addresses.len(), 2);
    assert!(addresses.contains(&json!("Anytown")));
    assert!(addresses.contains(&json!("Another City")));
}

/// 测试路径存在性检查
#[test]
fn test_path_existence() {
    let json_data = r#"
    {
        "user": {
            "name": "Alice",
            "age": 30
        }
    }
    "#;

    let exists = datapath_exists!(json_data, "user.name").unwrap();
    assert!(exists);

    let missing = datapath_exists!(json_data, "user.email").unwrap();
    assert!(!missing);
}

/// 测试值计数功能
#[test]
fn test_value_counting() {
    let json_data = r#"
    {
        "items": [1, 2, 3, 4, 5]
    }
    "#;

    let count = datapath_count!(json_data, "items[*]").unwrap();
    assert_eq!(count, 5);
}

/// 测试类型检测
#[test]
fn test_type_detection() {
    let json_data = r#"
    {
        "name": "Alice",
        "age": 30,
        "active": true,
        "scores": [85, 92, 78]
    }
    "#;

    let name_type = datapath_type!(json_data, "name").unwrap();
    assert_eq!(name_type, vec!["string"]);

    let age_type = datapath_type!(json_data, "age").unwrap();
    assert_eq!(age_type, vec!["number"]);

    let active_type = datapath_type!(json_data, "active").unwrap();
    assert_eq!(active_type, vec!["boolean"]);

    let scores_type = datapath_type!(json_data, "scores").unwrap();
    assert_eq!(scores_type, vec!["array"]);
}

/// 测试复杂路径表达式
#[test]
fn test_complex_path_expressions() {
    let json_data = r#"
    {
        "projects": [
            {
                "name": "Project A",
                "team": {
                    "members": [
                        {"name": "Alice", "role": "lead"},
                        {"name": "Bob", "role": "dev"}
                    ]
                }
            },
            {
                "name": "Project B",
                "team": {
                    "members": [
                        {"name": "Charlie", "role": "lead"},
                        {"name": "David", "role": "dev"}
                    ]
                }
            }
        ]
    }
    "#;

    // 获取所有项目名称
    let project_names = datapath_get!(json_data, "projects[*].name").unwrap();
    assert_eq!(project_names.len(), 2);
    assert!(project_names.contains(&json!("Project A")));
    assert!(project_names.contains(&json!("Project B")));

    // 获取所有团队成员名称
    let member_names =
        datapath_get!(json_data, "projects[*].team.members[*].name").unwrap();
    assert_eq!(member_names.len(), 4);
    assert!(member_names.contains(&json!("Alice")));
    assert!(member_names.contains(&json!("Bob")));
    assert!(member_names.contains(&json!("Charlie")));
    assert!(member_names.contains(&json!("David")));
}

/// 测试错误处理
#[test]
fn test_error_handling() {
    let json_data = r#"
    {
        "user": {
            "name": "Alice"
        }
    }
    "#;

    // 测试不存在的路径
    let result = datapath_get!(json_data, "user.nonexistent");
    assert!(result.is_ok()); // 应该返回空结果而不是错误
    assert_eq!(result.unwrap().len(), 0);
}

/// 测试更新功能（仅在启用 update feature 时）
#[cfg(feature = "update")]
#[test]
fn test_update_functionality() {
    let json_data = r#"
    {
        "user": {
            "name": "Alice",
            "age": 30
        }
    }
    "#;

    // 更新用户年龄
    let updated = datapath_set!(json_data, "user.age", json!(31)).unwrap();

    // 验证更新是否成功
    let age_result = datapath_get!(&updated, "user.age").unwrap();
    assert_eq!(age_result[0], json!(31));

    // 验证其他字段未受影响
    let name_result = datapath_get!(&updated, "user.name").unwrap();
    assert_eq!(name_result[0], json!("Alice"));
}

/// 测试创建新路径
#[cfg(feature = "update")]
#[test]
fn test_create_new_paths() {
    let json_data = r#"
    {
        "user": {
            "name": "Alice"
        }
    }
    "#;

    // 添加新字段
    let updated =
        datapath_set!(json_data, "user.email", json!("alice@example.com"))
            .unwrap();

    // 验证新字段是否添加
    let email_result = datapath_get!(&updated, "user.email").unwrap();
    assert_eq!(email_result[0], json!("alice@example.com"));

    // 创建嵌套路径
    let updated2 =
        datapath_set!(&updated, "user.profile.bio", json!("Software Engineer"))
            .unwrap();
    let bio_result = datapath_get!(&updated2, "user.profile.bio").unwrap();
    assert_eq!(bio_result[0], json!("Software Engineer"));
}

/// 测试批量更新（通配符）
#[cfg(feature = "update")]
#[test]
fn test_wildcard_updates() {
    let json_data = r#"
    {
        "users": [
            {"name": "Alice", "active": false},
            {"name": "Bob", "active": false}
        ]
    }
    "#;

    // 批量激活所有用户
    let updated =
        datapath_set!(json_data, "users[*].active", json!(true)).unwrap();

    // 验证所有用户都被激活
    let active_statuses = datapath_get!(&updated, "users[*].active").unwrap();
    assert_eq!(active_statuses.len(), 2);
    assert!(active_statuses.iter().all(|v| v == &json!(true)));
}

/// 测试混合格式处理
#[test]
fn test_mixed_format_handling() {
    // 测试 JSON 输入
    let json_input = r#"{"name": "Alice", "age": 30}"#;
    let json_result = datapath_get!(json_input, "name").unwrap();
    assert_eq!(json_result[0], json!("Alice"));

    // 测试 YAML 输入
    let yaml_input = r#"
name: Bob
age: 25
"#;
    let yaml_result = datapath_get!(yaml_input, "name").unwrap();
    assert_eq!(yaml_result[0], json!("Bob"));
}

/// 性能基准测试（简单）
#[test]
fn test_performance_basic() {
    let large_json = json!({
        "users": (0..1000).map(|i| {
            json!({
                "id": i,
                "name": format!("User{}", i),
                "active": i % 2 == 0
            })
        }).collect::<Vec<_>>()
    });

    let json_str = serde_json::to_string(&large_json).unwrap();

    // 测试提取大量数据的性能
    let start = std::time::Instant::now();
    let result = datapath_get!(&json_str, "users[*].name").unwrap();
    let duration = start.elapsed();

    assert_eq!(result.len(), 1000);
    println!("Performance test completed in {duration:?}");

    // 基本性能断言（应该在合理时间内完成）
    assert!(
        duration.as_millis() < 1000,
        "Performance test took too long: {duration:?}"
    );
}
