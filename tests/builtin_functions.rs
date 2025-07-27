use xqpath::{parse_path_expression, evaluate_path_expression, PathExpression};
use serde_json::json;

#[test]
fn test_builtin_functions() {
    // 测试 length 函数
    let data = json!({
        "users": [
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ],
        "name": "Test Company"
    });

    // 测试数组长度
    let expr = parse_path_expression(".users | length()").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!(2)]);

    // 测试字符串长度
    let expr = parse_path_expression(".name | length()").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!(12)]);

    // 测试 type 函数
    let expr = parse_path_expression(".users | type()").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!("array")]);

    let expr = parse_path_expression(".name | type()").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!("string")]);

    // 测试 keys 函数
    let expr = parse_path_expression(". | keys()").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!(["name", "users"])]);

    // 测试 values 函数
    let simple_obj = json!({"a": 1, "b": 2});
    let expr = parse_path_expression(". | values()").unwrap();
    let result = evaluate_path_expression(&expr, &simple_obj).unwrap();
    // values 的结果可能不是排序的，所以检查长度和内容
    let values_array = result[0].as_array().unwrap();
    assert_eq!(values_array.len(), 2);
    assert!(values_array.contains(&json!(1)));
    assert!(values_array.contains(&json!(2)));
}

#[test]
fn test_function_call_parsing() {
    // 测试无参函数调用
    let expr = parse_path_expression("length()").unwrap();
    if let PathExpression::FunctionCall { name, args } = expr {
        assert_eq!(name, "length");
        assert_eq!(args.len(), 0);
    } else {
        panic!("Expected function call");
    }

    // 测试带参函数调用 - 暂时跳过，因为 has 函数还未实现
    // let expr = parse_path_expression("has(\"name\")").unwrap();
    // if let PathExpression::FunctionCall { name, args } = expr {
    //     assert_eq!(name, "has");
    //     assert_eq!(args.len(), 1);
    // } else {
    //     panic!("Expected function call");
    // }

    // 测试管道中的函数调用
    let expr = parse_path_expression(".users | length()").unwrap();
    if let PathExpression::Pipe { left, right } = expr {
        if let PathExpression::FunctionCall { name, args } = *right {
            assert_eq!(name, "length");
            assert_eq!(args.len(), 0);
        } else {
            panic!("Expected function call in pipe");
        }
    } else {
        panic!("Expected pipe expression");
    }
}

#[test]
fn test_function_error_handling() {
    let data = json!({"name": "test"});

    // 测试未知函数
    let expr = parse_path_expression("unknown_function()").unwrap();
    let result = evaluate_path_expression(&expr, &data);
    assert!(result.is_err());

    // 测试参数错误
    let expr = parse_path_expression("length(\"arg\")").unwrap();
    let result = evaluate_path_expression(&expr, &data);
    assert!(result.is_err());

    // 测试类型错误
    let expr = parse_path_expression("42 | length()").unwrap();
    let result = evaluate_path_expression(&expr, &data);
    assert!(result.is_err());
}
