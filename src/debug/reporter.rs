//! 错误诊断和报告

use std::collections::HashMap;

/// 错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    PathError,
    ParsingError,
    TypeMismatchError,
    IndexOutOfBoundsError,
    FieldNotFoundError,
    InvalidFormatError,
    UnknownError,
}

/// 错误诊断信息
#[derive(Debug, Clone)]
pub struct DiagnosticInfo {
    pub error_type: ErrorType,
    pub message: String,
    pub path: String,
    pub position: Option<usize>,
    pub suggestions: Vec<String>,
    pub fix_suggestions: Vec<FixSuggestion>,
}

/// 修复建议
#[derive(Debug, Clone)]
pub struct FixSuggestion {
    pub description: String,
    pub fix_code: String,
    pub confidence: f32, // 0.0 到 1.0
}

/// 增强的错误类型
#[derive(Debug, Clone)]
pub struct EnhancedError {
    pub original_error: String,
    pub diagnostic: DiagnosticInfo,
}

impl EnhancedError {
    pub fn new(original_error: String, diagnostic: DiagnosticInfo) -> Self {
        Self {
            original_error,
            diagnostic,
        }
    }

    pub fn is_path_error(&self) -> bool {
        self.diagnostic.error_type == ErrorType::PathError
    }

    pub fn get_path_suggestion(&self) -> String {
        if let Some(suggestion) = self.diagnostic.suggestions.first() {
            suggestion.clone()
        } else {
            "No suggestions available".to_string()
        }
    }

    pub fn get_fix_suggestions(&self) -> &Vec<FixSuggestion> {
        &self.diagnostic.fix_suggestions
    }
}

impl std::fmt::Display for EnhancedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error: {}", self.original_error)?;
        writeln!(f, "Path: {}", self.diagnostic.path)?;

        if !self.diagnostic.suggestions.is_empty() {
            writeln!(f, "Suggestions:")?;
            for suggestion in &self.diagnostic.suggestions {
                writeln!(f, "  - {suggestion}")?;
            }
        }

        if !self.diagnostic.fix_suggestions.is_empty() {
            writeln!(f, "Possible fixes:")?;
            for fix in &self.diagnostic.fix_suggestions {
                writeln!(
                    f,
                    "  - {} (confidence: {:.1}%)",
                    fix.description,
                    fix.confidence * 100.0
                )?;
                writeln!(f, "    Fix: {}", fix.fix_code)?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for EnhancedError {}

/// 错误报告器
pub struct ErrorReporter {
    error_patterns: HashMap<String, ErrorPattern>,
}

/// 错误模式
struct ErrorPattern {
    error_type: ErrorType,
    pattern: String,
    suggestion_generator: fn(&str, &str) -> Vec<String>,
    fix_generator: fn(&str, &str) -> Vec<FixSuggestion>,
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorReporter {
    pub fn new() -> Self {
        let mut reporter = Self {
            error_patterns: HashMap::new(),
        };

        reporter.register_default_patterns();
        reporter
    }

    fn register_default_patterns(&mut self) {
        // 路径错误模式
        self.error_patterns.insert(
            "path_not_found".to_string(),
            ErrorPattern {
                error_type: ErrorType::PathError,
                pattern: "path not found".to_string(),
                suggestion_generator: generate_path_suggestions,
                fix_generator: generate_path_fixes,
            },
        );

        // 字段不存在模式
        self.error_patterns.insert(
            "field_not_found".to_string(),
            ErrorPattern {
                error_type: ErrorType::FieldNotFoundError,
                pattern: "field not found".to_string(),
                suggestion_generator: generate_field_suggestions,
                fix_generator: generate_field_fixes,
            },
        );

        // 索引越界模式
        self.error_patterns.insert(
            "index_out_of_bounds".to_string(),
            ErrorPattern {
                error_type: ErrorType::IndexOutOfBoundsError,
                pattern: "index out of bounds".to_string(),
                suggestion_generator: generate_index_suggestions,
                fix_generator: generate_index_fixes,
            },
        );

        // 类型不匹配模式
        self.error_patterns.insert(
            "type_mismatch".to_string(),
            ErrorPattern {
                error_type: ErrorType::TypeMismatchError,
                pattern: "type mismatch".to_string(),
                suggestion_generator: generate_type_suggestions,
                fix_generator: generate_type_fixes,
            },
        );
    }

    /// 增强错误信息
    pub fn enhance_error(
        &self,
        error_message: &str,
        path: &str,
    ) -> EnhancedError {
        let diagnostic = self.analyze_error(error_message, path);
        EnhancedError::new(error_message.to_string(), diagnostic)
    }

    fn analyze_error(&self, error_message: &str, path: &str) -> DiagnosticInfo {
        // 尝试匹配已知的错误模式
        for pattern in self.error_patterns.values() {
            if error_message.to_lowercase().contains(&pattern.pattern) {
                let suggestions =
                    (pattern.suggestion_generator)(error_message, path);
                let fix_suggestions =
                    (pattern.fix_generator)(error_message, path);

                return DiagnosticInfo {
                    error_type: pattern.error_type.clone(),
                    message: error_message.to_string(),
                    path: path.to_string(),
                    position: None,
                    suggestions,
                    fix_suggestions,
                };
            }
        }

        // 默认诊断信息
        DiagnosticInfo {
            error_type: ErrorType::UnknownError,
            message: error_message.to_string(),
            path: path.to_string(),
            position: None,
            suggestions: vec![
                "Check the path syntax and data structure".to_string()
            ],
            fix_suggestions: vec![],
        }
    }
}

// 建议生成函数

fn generate_path_suggestions(_error_message: &str, path: &str) -> Vec<String> {
    let mut suggestions = Vec::new();

    suggestions.push(format!("Check if path '{path}' exists in the data"));
    suggestions.push("Use '.[]' for array iteration".to_string());
    suggestions.push("Use '.*' for object field iteration".to_string());

    // 检查常见的路径错误
    if path.contains("..") {
        suggestions.push(
            "Use single '.' for field access instead of '..'".to_string(),
        );
    }

    if path.starts_with('[') && !path.starts_with(".[") {
        suggestions.push(
            "Array access should be prefixed with '.' (e.g., '.[0]')"
                .to_string(),
        );
    }

    suggestions
}

fn generate_path_fixes(_error_message: &str, path: &str) -> Vec<FixSuggestion> {
    let mut fixes = Vec::new();

    // 常见路径修复
    if path.contains("..") {
        fixes.push(FixSuggestion {
            description: "Replace '..' with single '.'".to_string(),
            fix_code: path.replace("..", "."),
            confidence: 0.8,
        });
    }

    if path.starts_with('[') && !path.starts_with(".[") {
        fixes.push(FixSuggestion {
            description: "Add '.' prefix for array access".to_string(),
            fix_code: format!(".{path}"),
            confidence: 0.9,
        });
    }

    fixes
}

fn generate_field_suggestions(_error_message: &str, path: &str) -> Vec<String> {
    vec![
        format!("Field in path '{}' does not exist", path),
        "Check the field name spelling".to_string(),
        "Use '.*' to list all available fields".to_string(),
    ]
}

fn generate_field_fixes(
    _error_message: &str,
    _path: &str,
) -> Vec<FixSuggestion> {
    vec![FixSuggestion {
        description: "Use exists() to check field existence first".to_string(),
        fix_code: "exists(data, path)".to_string(),
        confidence: 0.7,
    }]
}

fn generate_index_suggestions(_error_message: &str, path: &str) -> Vec<String> {
    vec![
        format!("Array index in path '{}' is out of bounds", path),
        "Check array length first".to_string(),
        "Use '.[]' to iterate over all array elements".to_string(),
    ]
}

fn generate_index_fixes(
    _error_message: &str,
    _path: &str,
) -> Vec<FixSuggestion> {
    vec![FixSuggestion {
        description: "Use count() to get array length first".to_string(),
        fix_code: "count(data, path)".to_string(),
        confidence: 0.8,
    }]
}

fn generate_type_suggestions(_error_message: &str, path: &str) -> Vec<String> {
    vec![
        format!("Type mismatch at path '{}'", path),
        "Check the expected data type".to_string(),
        "Use type filters like '.[]?' for safe access".to_string(),
    ]
}

fn generate_type_fixes(
    _error_message: &str,
    _path: &str,
) -> Vec<FixSuggestion> {
    vec![FixSuggestion {
        description: "Add type check before access".to_string(),
        fix_code: "if value.is_array() { /* access */ }".to_string(),
        confidence: 0.6,
    }]
}
