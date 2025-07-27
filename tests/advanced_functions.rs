use serde_json::json;
use xqpath::{evaluate_path_expression, parse_path_expression, PathExpression};

#[allow(clippy::uninlined_format_args)]
#[test]
fn test_map_function() {
    let data = json!([1, 2, 3, 4, 5]);

    // 测试简单的map操作 - 使用恒等变换
    let result = evaluate_path_expression(
        &parse_path_expression(". | map(.)").unwrap(),
        &data,
    )
    .unwrap();

    assert_eq!(result, vec![json!([1, 2, 3, 4, 5])]);

    // 测试map访问对象属性
    let users_data = json!([
        {"name": "Alice", "age": 30},
        {"name": "Bob", "age": 25}
    ]);

    let result = evaluate_path_expression(
        &parse_path_expression(". | map(.name)").unwrap(),
        &users_data,
    )
    .unwrap();

    assert_eq!(result, vec![json!(["Alice", "Bob"])]);
}

#[test]
fn test_select_function() {
    let data = json!([1, 2, 3, 4, 5]);

    // 选择大于3的元素
    let result = evaluate_path_expression(
        &parse_path_expression(". | select(. > 3)").unwrap(),
        &data,
    )
    .unwrap();

    assert_eq!(result, vec![json!([4, 5])]);

    // 测试对象数组的select
    let users_data = json!([
        {"name": "Alice", "age": 30},
        {"name": "Bob", "age": 25},
        {"name": "Charlie", "age": 35}
    ]);

    let result = evaluate_path_expression(
        &parse_path_expression(". | select(.age > 28)").unwrap(),
        &users_data,
    )
    .unwrap();

    assert_eq!(
        result,
        vec![json!([
            {"name": "Alice", "age": 30},
            {"name": "Charlie", "age": 35}
        ])]
    );
}

#[test]
fn test_sort_function() {
    let data = json!([3, 1, 4, 1, 5, 9, 2, 6]);

    let result = evaluate_path_expression(
        &parse_path_expression(". | sort()").unwrap(),
        &data,
    )
    .unwrap();

    assert_eq!(result, vec![json!([1, 1, 2, 3, 4, 5, 6, 9])]);

    // 测试字符串排序
    let string_data = json!(["banana", "apple", "cherry"]);
    let result = evaluate_path_expression(
        &parse_path_expression(". | sort()").unwrap(),
        &string_data,
    )
    .unwrap();

    assert_eq!(result, vec![json!(["apple", "banana", "cherry"])]);
}

#[test]
fn test_sort_by_function() {
    let users_data = json!([
        {"name": "Alice", "age": 30},
        {"name": "Bob", "age": 25},
        {"name": "Charlie", "age": 35}
    ]);

    // 按年龄排序
    let result = evaluate_path_expression(
        &parse_path_expression(". | sort_by(.age)").unwrap(),
        &users_data,
    )
    .unwrap();

    assert_eq!(
        result,
        vec![json!([
            {"name": "Bob", "age": 25},
            {"name": "Alice", "age": 30},
            {"name": "Charlie", "age": 35}
        ])]
    );

    // 按名字排序
    let result = evaluate_path_expression(
        &parse_path_expression(". | sort_by(.name)").unwrap(),
        &users_data,
    )
    .unwrap();

    assert_eq!(
        result,
        vec![json!([
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25},
            {"name": "Charlie", "age": 35}
        ])]
    );
}

#[test]
fn test_group_by_function() {
    let users_data = json!([
        {"name": "Alice", "department": "Engineering"},
        {"name": "Bob", "department": "Sales"},
        {"name": "Charlie", "department": "Engineering"},
        {"name": "David", "department": "Sales"}
    ]);

    let result = evaluate_path_expression(
        &parse_path_expression(". | group_by(.department)").unwrap(),
        &users_data,
    )
    .unwrap();

    // 结果应该是一个包含分组的数组
    assert_eq!(result.len(), 1);
    if let Some(groups) = result[0].as_array() {
        assert_eq!(groups.len(), 2); // Engineering 和 Sales 两个组

        // 检查每个组都包含正确的元素
        let mut engineering_count = 0;
        let mut sales_count = 0;

        for group in groups {
            if let Some(group_array) = group.as_array() {
                if let Some(first_person) = group_array.first() {
                    if let Some(dept) =
                        first_person.get("department").and_then(|d| d.as_str())
                    {
                        match dept {
                            "Engineering" => {
                                engineering_count = group_array.len();
                                assert_eq!(engineering_count, 2);
                            }
                            "Sales" => {
                                sales_count = group_array.len();
                                assert_eq!(sales_count, 2);
                            }
                            _ => panic!("Unexpected department: {dept}"),
                        }
                    }
                }
            }
        }

        assert_eq!(engineering_count, 2);
        assert_eq!(sales_count, 2);
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_unique_function() {
    let data = json!([1, 2, 2, 3, 3, 3, 4]);

    let result = evaluate_path_expression(
        &parse_path_expression(". | unique()").unwrap(),
        &data,
    )
    .unwrap();

    assert_eq!(result, vec![json!([1, 2, 3, 4])]);

    // 测试字符串去重
    let string_data = json!(["apple", "banana", "apple", "cherry", "banana"]);
    let result = evaluate_path_expression(
        &parse_path_expression(". | unique()").unwrap(),
        &string_data,
    )
    .unwrap();

    assert_eq!(result, vec![json!(["apple", "banana", "cherry"])]);
}

#[test]
fn test_unique_by_function() {
    let users_data = json!([
        {"name": "Alice", "age": 30},
        {"name": "Bob", "age": 25},
        {"name": "Charlie", "age": 30},
        {"name": "David", "age": 25}
    ]);

    // 按年龄去重
    let result = evaluate_path_expression(
        &parse_path_expression(". | unique_by(.age)").unwrap(),
        &users_data,
    )
    .unwrap();

    assert_eq!(
        result,
        vec![json!([
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ])]
    );
}

#[test]
fn test_reverse_function() {
    let data = json!([1, 2, 3, 4, 5]);

    let result = evaluate_path_expression(
        &parse_path_expression(". | reverse()").unwrap(),
        &data,
    )
    .unwrap();

    assert_eq!(result, vec![json!([5, 4, 3, 2, 1])]);

    // 测试字符串数组反转
    let string_data = json!(["apple", "banana", "cherry"]);
    let result = evaluate_path_expression(
        &parse_path_expression(". | reverse()").unwrap(),
        &string_data,
    )
    .unwrap();

    assert_eq!(result, vec![json!(["cherry", "banana", "apple"])]);
}

#[test]
fn test_advanced_function_combinations() {
    let users_data = json!([
        {"name": "Alice", "age": 30, "salary": 70000},
        {"name": "Bob", "age": 25, "salary": 50000},
        {"name": "Charlie", "age": 35, "salary": 80000},
        {"name": "David", "age": 28, "salary": 60000}
    ]);

    // 复杂组合：筛选、排序、映射
    let result = evaluate_path_expression(
        &parse_path_expression(
            ". | select(.age > 25) | sort_by(.salary) | map(.name)",
        )
        .unwrap(),
        &users_data,
    )
    .unwrap();

    assert_eq!(result, vec![json!(["David", "Alice", "Charlie"])]);

    // 组合：映射后去重
    let ages_data = json!([
        {"age": 30}, {"age": 25}, {"age": 30}, {"age": 28}
    ]);

    let result = evaluate_path_expression(
        &parse_path_expression(". | map(.age) | unique() | sort()").unwrap(),
        &ages_data,
    )
    .unwrap();

    assert_eq!(result, vec![json!([25, 28, 30])]);
}

#[test]
fn test_advanced_function_error_handling() {
    let data = json!([1, 2, 3]);

    // map 需要表达式参数
    let result = evaluate_path_expression(
        &parse_path_expression(". | map()").unwrap(),
        &data,
    );
    assert!(result.is_err());

    // select 需要表达式参数
    let result = evaluate_path_expression(
        &parse_path_expression(". | select()").unwrap(),
        &data,
    );
    assert!(result.is_err());

    // sort 不需要参数
    let result = evaluate_path_expression(
        &parse_path_expression(". | sort(invalid_arg)").unwrap(),
        &data,
    );
    assert!(result.is_err());

    // 对非数组使用数组函数
    let scalar_data = json!(42);
    let result = evaluate_path_expression(
        &parse_path_expression(". | map(.)").unwrap(),
        &scalar_data,
    );
    assert!(result.is_err());
}

#[test]
fn test_function_call_parsing() {
    // 测试高级函数调用解析
    let expr = parse_path_expression("map(.name)").unwrap();
    if let PathExpression::FunctionCall { name, args } = expr {
        assert_eq!(name, "map");
        assert_eq!(args.len(), 1);
    } else {
        panic!("Expected function call");
    }

    // 测试无参函数调用
    let expr = parse_path_expression("sort()").unwrap();
    if let PathExpression::FunctionCall { name, args } = expr {
        assert_eq!(name, "sort");
        assert_eq!(args.len(), 0);
    } else {
        panic!("Expected function call");
    }

    // 测试简单的管道中的高级函数调用
    let expr = parse_path_expression(".users | map(.name)").unwrap();
    if let PathExpression::Pipe { left: _, right } = expr {
        if let PathExpression::FunctionCall { name, args } = *right {
            assert_eq!(name, "map");
            assert_eq!(args.len(), 1);
        } else {
            panic!("Expected map function call");
        }
    } else {
        panic!("Expected pipe expression");
    }
}
