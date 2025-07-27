use serde_json::json;
use xqpath::{evaluate_path_expression, parse_path_expression};

#[test]
fn test_try_catch_basic() {
    let data = json!({
        "user": {
            "name": "Alice",
            "age": 30
        }
    });

    // 成功的 try 表达式
    let expr = parse_path_expression("try .user.name").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!("Alice")]);

    // 失败的 try 表达式（没有 catch）- 字段不存在返回空数组，not null
    let expr = parse_path_expression("try .user.nonexistent").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    let expected: Vec<serde_json::Value> = vec![];
    assert_eq!(result, expected); // 空数组，不是 null
}

#[test]
fn test_try_catch_with_fallback() {
    let data = json!({
        "user": {
            "name": "Alice",
            "age": 30
        }
    });

    // try-catch 表达式，使用 fallback - 需要创建真正的错误来触发 catch
    // 使用除零或其他会导致错误的操作，但目前 XQPath 不支持算术运算
    // 让我们使用函数调用错误
    let expr =
        parse_path_expression("try unknown_function() catch \"default\"")
            .unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!("default")]);

    // try-catch 表达式，字段不存在不会触发错误，所以返回空数组
    let expr =
        parse_path_expression("try .user.email catch .user.name").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    let expected: Vec<serde_json::Value> = vec![]; // 空数组而不是 fallback
    assert_eq!(result, expected);
}

#[test]
fn test_optional_operator_basic() {
    let data = json!({
        "user": {
            "name": "Alice",
            "age": 30
        }
    });

    // 成功的可选操作符
    let expr = parse_path_expression(".user.name?").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!("Alice")]);

    // 失败的可选操作符
    let expr = parse_path_expression(".user.nonexistent?").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!(null)]);
}

#[test]
fn test_optional_operator_with_functions() {
    let data = json!({
        "numbers": [1, 2, 3, 4, 5]
    });

    // 使用可选操作符与函数调用 - 需要使用 length() 而不是 length
    let expr = parse_path_expression(".numbers | length()?").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!(5)]);

    // 在不存在的字段上使用可选操作符 - 空数组输入到函数会返回什么取决于函数实现
    let expr = parse_path_expression(".nonexistent | length()?").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    let expected: Vec<serde_json::Value> = vec![]; // 空数组进入管道，函数不会被调用
    assert_eq!(result, expected);
}

#[test]
fn test_nested_try_catch() {
    let data = json!({
        "config": {
            "fallback": "backup_value"
        }
    });

    // 嵌套的 try-catch 表达式 - 使用不存在的字段来触发错误处理
    let expr = parse_path_expression("try (try unknown_function() catch .config.fallback) catch \"ultimate_fallback\"").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!("backup_value")]);
}

#[test]
fn test_optional_with_pipe() {
    let data = json!({
        "users": [
            {"name": "Alice", "email": "alice@example.com"},
            {"name": "Bob"}
        ]
    });

    // 可选操作符与索引访问
    let expr = parse_path_expression(".users[1]?").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!({"name": "Bob"})]);

    // 简化的可选操作符测试
    let expr = parse_path_expression(".users[10]?").unwrap(); // 超出索引范围
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!(null)]); // 可选操作符将空结果转换为 null
}

#[test]
fn test_try_catch_with_conditions() {
    let data = json!({
        "user": {
            "age": 25
        }
    });

    // try-catch 与条件表达式结合
    let expr = parse_path_expression("try (if .user.age > 30 then \"senior\" else \"junior\" end) catch \"unknown\"").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!("junior")]);

    // 条件表达式中的 try-catch
    let expr = parse_path_expression("if (try .user.email catch null) != null then \"has_email\" else \"no_email\" end").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!("no_email")]);
}

#[test]
fn test_error_handling_combinations() {
    let data = json!({
        "data": [1, 2, 3]
    });

    // 结合多种错误处理方式 - 使用函数调用语法
    let expr =
        parse_path_expression("try (.data | length()?) catch 0").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!(3)]);

    // 简化的组合表达式
    let expr =
        parse_path_expression("try .nonexistent? catch (.data | length())")
            .unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!(null)]); // try 成功但返回 null
}

#[test]
fn test_expression_display() {
    // 测试新表达式的字符串表示
    let expr =
        parse_path_expression("try .user.name catch \"default\"").unwrap();
    assert_eq!(expr.as_string(), "try .user.name catch \"default\"");

    let expr = parse_path_expression(".user.name?").unwrap();
    assert_eq!(expr.as_string(), ".user.name?");

    let expr = parse_path_expression("try .field").unwrap();
    assert_eq!(expr.as_string(), "try .field");
}
