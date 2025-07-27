use serde_json::json;
use xqpath::{
    evaluate_path_expression, parse_path_expression, ComparisonOp, LogicalOp,
    PathExpression,
};

#[test]
fn test_comparison_operations() {
    let data = json!({ "age": 25, "name": "Alice" });

    // 数值比较
    let expr = parse_path_expression(".age == 25").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!(true)]);

    let expr = parse_path_expression(".age != 25").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!(false)]);

    let expr = parse_path_expression(".age > 20").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!(true)]);

    let expr = parse_path_expression(".age < 30").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!(true)]);

    // 字符串比较
    let expr = parse_path_expression(".name == \"Alice\"").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!(true)]);
}

#[test]
fn test_logical_operations() {
    let data = json!({ "age": 25, "active": true, "name": "Alice" });

    // and 操作
    let expr = parse_path_expression(".age > 20 and .active").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!(true)]);

    let expr = parse_path_expression(".age < 20 and .active").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!(false)]);

    // or 操作
    let expr = parse_path_expression(".age < 20 or .active").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!(true)]);

    // not 操作
    let expr = parse_path_expression("not .active").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!(false)]);
}

#[test]
fn test_conditional_expressions() {
    let data = json!({ "age": 25, "name": "Alice" });

    // 简单条件表达式
    let expr =
        parse_path_expression("if .age > 18 then \"adult\" else \"minor\" end")
            .unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!("adult")]);

    let expr =
        parse_path_expression("if .age < 18 then \"minor\" else \"adult\" end")
            .unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!("adult")]);

    // 条件表达式没有else分支
    let expr =
        parse_path_expression("if .age > 30 then \"senior\" end").unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!(null)]);
}

#[test]
fn test_complex_expressions() {
    let data = json!({
        "users": [
            {"name": "Alice", "age": 25, "active": true},
            {"name": "Bob", "age": 17, "active": false},
            {"name": "Carol", "age": 30, "active": true}
        ]
    });

    // 组合条件和函数调用
    let expr = parse_path_expression(
        "if (.users | length()) > 2 then \"many\" else \"few\" end",
    )
    .unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!("many")]);

    // 复合逻辑条件
    let expr = parse_path_expression(".users[0].age > 18 and .users[0].active")
        .unwrap();
    let result = evaluate_path_expression(&expr, &data).unwrap();
    assert_eq!(result, vec![json!(true)]);
}

#[test]
fn test_ast_structure() {
    // 测试AST结构的正确性

    // 比较表达式AST
    let expr = parse_path_expression(".age > 18").unwrap();
    if let PathExpression::Comparison { left, op, right } = expr {
        assert!(matches!(*left, PathExpression::Segments(_)));
        assert_eq!(op, ComparisonOp::GreaterThan);
        assert!(matches!(*right, PathExpression::Literal(_)));
    } else {
        panic!("Expected comparison expression");
    }

    // 逻辑表达式AST
    let expr = parse_path_expression(".age > 18 and .active").unwrap();
    if let PathExpression::Logical { op, operands } = expr {
        assert_eq!(op, LogicalOp::And);
        assert_eq!(operands.len(), 2);
    } else {
        panic!("Expected logical expression");
    }

    // 条件表达式AST
    let expr =
        parse_path_expression("if .age > 18 then \"adult\" else \"minor\" end")
            .unwrap();
    if let PathExpression::Conditional {
        condition,
        then_expr,
        else_expr,
    } = expr
    {
        assert!(matches!(*condition, PathExpression::Comparison { .. }));
        assert!(matches!(*then_expr, PathExpression::Literal(_)));
        assert!(else_expr.is_some());
        assert!(matches!(*else_expr.unwrap(), PathExpression::Literal(_)));
    } else {
        panic!("Expected conditional expression");
    }
}
