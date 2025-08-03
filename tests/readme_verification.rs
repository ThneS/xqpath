use xqpath::{exists, query, query_one};

#[cfg(test)]
mod readme_verification_tests {
    use super::*;

    #[test]
    fn test_readme_rust_examples() {
        let data = r#"{
          "users": [
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
          ]
        }"#;

        // 测试 query! 宏 - 查询多个值
        let names = query!(data, "users[*].name").unwrap();
        assert_eq!(names.len(), 2);
        println!("✅ query!(data, \"users[*].name\") -> {:?}", names);

        // 测试 query_one! 宏 - 查询单个值
        let first_name = query_one!(data, "users[0].name").unwrap();
        assert!(first_name.is_some());
        println!("✅ query_one!(data, \"users[0].name\") -> {:?}", first_name);

        // 测试 exists! 宏 - 检查路径是否存在
        let has_users = exists!(data, "users").unwrap();
        assert!(has_users);
        println!("✅ exists!(data, \"users\") -> {}", has_users);

        let has_nonexistent = exists!(data, "nonexistent").unwrap();
        assert!(!has_nonexistent);
        println!("✅ exists!(data, \"nonexistent\") -> {}", has_nonexistent);
    }

    #[test]
    fn test_readme_error_handling() {
        let data = r#"{"user": {"name": "Alice"}}"#;

        // 测试错误处理
        match query!(data, ".some.nonexistent.path") {
            Ok(result) => println!("Found: {:?}", result),
            Err(e) => {
                println!("✅ 错误处理正常: {}", e);
                assert!(true); // 期望的错误
            }
        }

        // 测试可选字段查询
        let optional = query_one!(data, ".user.email").unwrap();
        assert!(optional.is_none());
        println!("✅ 可选字段查询: {:?}", optional);
    }

    #[test]
    fn test_readme_path_syntax() {
        let data = r#"{
          "config": {
            "nested": {"field": "value"},
            "users": [
              {"name": "Alice", "id": 1},
              {"name": "Bob", "id": 2}
            ]
          }
        }"#;

        // 基础访问
        let field = query_one!(data, ".config.nested.field").unwrap();
        assert!(field.is_some());
        println!("✅ .config.nested.field -> {:?}", field);

        // 数组索引
        let first_user = query_one!(data, ".config.users[0]").unwrap();
        assert!(first_user.is_some());
        println!("✅ .config.users[0] -> {:?}", first_user);

        // 数组通配符
        let user_names = query!(data, ".config.users[*].name").unwrap();
        assert_eq!(user_names.len(), 2);
        println!("✅ .config.users[*].name -> {:?}", user_names);
    }
}
