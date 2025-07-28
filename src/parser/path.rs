use winnow::{
    ascii::{alpha1, digit1},
    combinator::{alt, delimited, repeat},
    token::take_while,
    PResult, Parser,
};

/// 路径段枚举，表示路径中的不同组件
#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment {
    /// 字段访问，如 .field
    Field(String),
    /// 数组索引访问，如 \[0\]
    Index(usize),
    /// 通配符，匹配任意字段名 *
    Wildcard,
    /// 递归通配符，递归匹配所有字段 **
    RecursiveWildcard,
    /// 类型过滤器，如 | string
    TypeFilter(String),
}

/// 解析结果类型
pub type ParseResult<T> = Result<T, ParseError>;

/// 解析错误类型
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parse error at position {}: {}",
            self.position, self.message
        )
    }
}

impl std::error::Error for ParseError {}

/// 解析字段名（标识符）
fn parse_identifier(input: &mut &str) -> PResult<String> {
    (
        alpha1,
        take_while(0.., |c: char| c.is_alphanumeric() || c == '_'),
    )
        .recognize()
        .map(|s: &str| s.to_string())
        .parse_next(input)
}

/// 解析数字
fn parse_number(input: &mut &str) -> PResult<usize> {
    digit1.try_map(|s: &str| s.parse()).parse_next(input)
}

/// 跳过空白字符
fn skip_whitespace(input: &mut &str) -> PResult<()> {
    take_while(0.., |c: char| c == ' ' || c == '\t')
        .void()
        .parse_next(input)
}

/// 解析字段访问 .field 或裸字段 field
fn parse_field(input: &mut &str) -> PResult<PathSegment> {
    alt((
        // 带点的字段访问 .field
        ('.', parse_identifier).map(|(_, name)| PathSegment::Field(name)),
        // 裸字段名（只在路径开始时或特定上下文中允许）
        parse_identifier.map(PathSegment::Field),
    ))
    .parse_next(input)
}

/// 解析数组索引 [index] 或通配符 [*] 或空数组 []
fn parse_index(input: &mut &str) -> PResult<PathSegment> {
    delimited(
        '[',
        alt((
            // 处理 [*] - 通配符
            '*'.value(PathSegment::Wildcard),
            // 处理具体索引
            parse_number.map(PathSegment::Index),
            // 处理空数组 [] - 也视为通配符
            winnow::combinator::empty.value(PathSegment::Wildcard),
        )),
        ']',
    )
    .parse_next(input)
}

/// 解析通配符 * (但不是 **)
fn parse_wildcard(input: &mut &str) -> PResult<PathSegment> {
    // 确保这不是 **
    if input.starts_with("**") {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ParserError::from_error_kind(
                input,
                winnow::error::ErrorKind::Verify,
            ),
        ));
    }
    '*'.value(PathSegment::Wildcard).parse_next(input)
}

/// 解析递归通配符 **
fn parse_recursive_wildcard(input: &mut &str) -> PResult<PathSegment> {
    "**".value(PathSegment::RecursiveWildcard).parse_next(input)
}

/// 解析类型过滤器 | type
fn parse_type_filter(input: &mut &str) -> PResult<PathSegment> {
    (skip_whitespace, '|', skip_whitespace, parse_identifier)
        .map(|(_, _, _, type_name)| PathSegment::TypeFilter(type_name))
        .parse_next(input)
}

/// 解析单个路径段
fn parse_segment(input: &mut &str) -> PResult<PathSegment> {
    alt((
        parse_recursive_wildcard, // 必须在 wildcard 之前，因为 ** 包含 *
        parse_type_filter,        // 类型过滤器需要较早解析
        parse_field,
        parse_index,
        parse_wildcard,
    ))
    .parse_next(input)
}

/// 解析完整路径表达式
fn parse_path_internal(input: &mut &str) -> PResult<Vec<PathSegment>> {
    // 跳过开头的空白字符
    skip_whitespace.parse_next(input)?;

    // 解析路径段序列
    let segments = repeat(0.., parse_segment).parse_next(input)?;

    // 跳过结尾的空白字符
    skip_whitespace.parse_next(input)?;

    Ok(segments)
}

/// 公共解析函数
pub fn parse_path(input: &str) -> ParseResult<Vec<PathSegment>> {
    let mut input_ref = input;
    match parse_path_internal.parse_next(&mut input_ref) {
        Ok(segments) => {
            if input_ref.is_empty() {
                Ok(segments)
            } else {
                Err(ParseError {
                    message: format!("Unexpected characters: '{input_ref}'"),
                    position: input.len() - input_ref.len(),
                })
            }
        }
        Err(e) => Err(ParseError {
            message: format!("Failed to parse path: {e:?}"),
            position: input.len() - input_ref.len(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_field() {
        let result = parse_path(".name").unwrap();
        assert_eq!(result, vec![PathSegment::Field("name".to_string())]);
    }

    #[test]
    fn test_parse_index() {
        let result = parse_path("[0]").unwrap();
        assert_eq!(result, vec![PathSegment::Index(0)]);
    }

    #[test]
    fn test_parse_wildcard() {
        let result = parse_path("*").unwrap();
        assert_eq!(result, vec![PathSegment::Wildcard]);
    }

    #[test]
    fn test_parse_recursive_wildcard() {
        let result = parse_path("**").unwrap();
        assert_eq!(result, vec![PathSegment::RecursiveWildcard]);
    }

    #[test]
    fn test_parse_complex_path() {
        let result = parse_path(".users[0].name").unwrap();
        assert_eq!(
            result,
            vec![
                PathSegment::Field("users".to_string()),
                PathSegment::Index(0),
                PathSegment::Field("name".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_with_wildcards() {
        let result = parse_path(".users[*].name").unwrap();
        assert_eq!(
            result,
            vec![
                PathSegment::Field("users".to_string()),
                PathSegment::Wildcard,
                PathSegment::Field("name".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_type_filter() {
        let result = parse_path(".users[*] | string").unwrap();
        assert_eq!(
            result,
            vec![
                PathSegment::Field("users".to_string()),
                PathSegment::Wildcard,
                PathSegment::TypeFilter("string".to_string()),
            ]
        );
    }
}
