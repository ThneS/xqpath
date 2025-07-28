/// 比较操作符
#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOp {
    /// 等于 ==
    Equal,
    /// 不等于 !=
    NotEqual,
    /// 小于 <
    LessThan,
    /// 小于等于 <=
    LessThanOrEqual,
    /// 大于 >
    GreaterThan,
    /// 大于等于 >=
    GreaterThanOrEqual,
}

/// 逻辑操作符
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOp {
    /// 逻辑与 and / &&
    And,
    /// 逻辑或 or / ||
    Or,
    /// 逻辑非 not
    Not,
}
