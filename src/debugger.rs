//! # 交互式调试器
//!
//! 提供交互式的XQPath查询调试环境，支持断点、监视点、单步执行等功能。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(feature = "interactive-debug")]
use rustyline::{history::DefaultHistory, Editor};

/// 交互式调试器主结构
#[cfg(feature = "interactive-debug")]
#[derive(Debug)]
pub struct XQPathDebugger {
    data_inspector: DataInspector,
    command_history: CommandHistory,
    session: DebugSession,
}

/// 调试会话，包含断点、监视点等调试状态
#[derive(Debug, Clone)]
pub struct DebugSession {
    pub breakpoints: Vec<Breakpoint>,
    pub watch_points: Vec<WatchPoint>,
    pub call_stack: CallStack,
    pub variables: VariableScope,
    pub current_data: Option<Value>,
    pub execution_state: ExecutionState,
}

/// 查询求值器
#[derive(Debug, Clone)]
pub struct QueryEvaluator {
    pub current_query: Option<String>,
    pub last_result: Option<Value>,
    pub evaluation_context: EvaluationContext,
}

/// 数据检查器
#[derive(Debug, Clone, Default)]
pub struct DataInspector {
    pub inspect_target: Option<Value>,
    pub inspect_path: Option<String>,
    pub type_info: Option<TypeInfo>,
}

/// 命令历史管理
#[derive(Debug, Clone)]
pub struct CommandHistory {
    commands: Vec<DebugCommand>,
    current_index: usize,
}

/// 断点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub id: u32,
    pub path: String,
    pub condition: Option<String>,
    pub enabled: bool,
}

/// 监视点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchPoint {
    pub id: u32,
    pub expression: String,
    pub condition: Option<String>,
    pub enabled: bool,
}

/// 调用栈
#[derive(Debug, Clone)]
pub struct CallStack {
    pub frames: Vec<StackFrame>,
    pub current_frame: usize,
}

/// 栈帧
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_name: String,
    pub query: String,
    pub variables: HashMap<String, Value>,
    pub line: u32,
}

/// 变量作用域
#[derive(Debug, Clone)]
pub struct VariableScope {
    pub global_vars: HashMap<String, Value>,
    pub local_vars: HashMap<String, Value>,
    pub current: Option<Value>,
}

/// 执行状态
#[derive(Debug, Clone)]
pub enum ExecutionState {
    Running,
    Paused,
    Stepping,
    Stopped,
}

/// 评估上下文
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    pub data: Option<Value>,
    pub path: Option<String>,
    pub filters: Vec<String>,
}

/// 类型信息
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub type_name: String,
    pub size: Option<usize>,
    pub properties: Vec<PropertyInfo>,
}

/// 属性信息
#[derive(Debug, Clone)]
pub struct PropertyInfo {
    pub name: String,
    pub type_name: String,
    pub value: Option<Value>,
}

/// 调试命令
#[derive(Debug, Clone)]
pub enum DebugCommand {
    Help,
    Quit,
    Load {
        file: PathBuf,
    },
    Save {
        file: PathBuf,
    },
    Inspect {
        path: String,
    },
    SetBreakpoint {
        path: String,
        condition: Option<String>,
    },
    RemoveBreakpoint {
        id: u32,
    },
    ListBreakpoints,
    SetWatchPoint {
        expression: String,
        condition: Option<String>,
    },
    RemoveWatchPoint {
        id: u32,
    },
    ListWatchPoints,
    Continue,
    Step,
    StepInto,
    StepOut,
    Run {
        query: String,
    },
    Evaluate {
        expression: String,
    },
    ListVariables,
    ShowCallStack,
    Reset,
}

/// 调试错误
#[derive(Debug)]
pub enum DebugError {
    InvalidCommand(String),
    FileNotFound(PathBuf),
    ParseError(String),
    EvaluationError(String),
    IOError(std::io::Error),
}

/// 调试结果
pub type DebugResult<T> = Result<T, DebugError>;

impl std::fmt::Display for DebugError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DebugError::InvalidCommand(cmd) => {
                write!(f, "Invalid command: {}", cmd)
            }
            DebugError::FileNotFound(path) => {
                write!(f, "File not found: {:?}", path)
            }
            DebugError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            DebugError::EvaluationError(msg) => {
                write!(f, "Evaluation error: {}", msg)
            }
            DebugError::IOError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for DebugError {}

impl From<std::io::Error> for DebugError {
    fn from(err: std::io::Error) -> Self {
        DebugError::IOError(err)
    }
}

#[cfg(feature = "interactive-debug")]
impl XQPathDebugger {
    /// 创建新的调试器实例
    pub fn new() -> Self {
        Self {
            data_inspector: DataInspector::default(),
            command_history: CommandHistory::new(),
            session: DebugSession::new(),
        }
    }

    /// 启动交互式调试会话
    pub fn run(&mut self) -> DebugResult<()> {
        println!("🔍 XQPath Interactive Debugger");
        println!("Type ':help' for available commands, ':quit' to exit\n");

        let mut rl = Editor::<(), DefaultHistory>::new().map_err(|e| {
            DebugError::IOError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e,
            ))
        })?;

        loop {
            match rl.readline("xqpath> ") {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    let _ = rl.add_history_entry(line);

                    match self.execute_command(line) {
                        Ok(should_continue) => {
                            if !should_continue {
                                break;
                            }
                        }
                        Err(err) => {
                            eprintln!("Error: {}", err);
                        }
                    }
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(rustyline::error::ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }

        println!("Goodbye!");
        Ok(())
    }

    /// 执行调试命令
    fn execute_command(&mut self, input: &str) -> DebugResult<bool> {
        let command = DebugCommand::parse(input)?;
        self.command_history.add_command(command.clone());

        match command {
            DebugCommand::Help => {
                self.show_help();
                Ok(true)
            }
            DebugCommand::Quit => Ok(false),
            DebugCommand::Load { file: _file } => {
                println!(
                    "⚠️  Load command will be implemented in future version"
                );
                Ok(true)
            }
            DebugCommand::Save { file: _file } => {
                println!(
                    "⚠️  Save command will be implemented in future version"
                );
                Ok(true)
            }
            DebugCommand::Inspect { path: _path } => {
                println!(
                    "⚠️  Inspect command will be implemented in future version"
                );
                Ok(true)
            }
            DebugCommand::Run { query: _query } => {
                println!(
                    "⚠️  Run command will be implemented in future version"
                );
                Ok(true)
            }
            DebugCommand::Evaluate { expression: _expr } => {
                println!("⚠️  Evaluate command will be implemented in future version");
                Ok(true)
            }
            _ => {
                println!(
                    "⚠️  This command will be implemented in future version"
                );
                Ok(true)
            }
        }
    }

    /// 显示帮助信息
    fn show_help(&self) {
        println!("Available commands:");
        println!("  :help                    - Show this help message");
        println!("  :quit                    - Exit the debugger");
        println!("  :load <file>             - Load data from file");
        println!("  :save <file>             - Save current data to file");
        println!("  :inspect <path>          - Inspect data at path");
        println!("  :run <query>             - Run a query");
        println!("  :eval <expression>       - Evaluate an expression");
        println!();
        println!("⚠️  Most commands will be implemented in future versions");
    }
}

impl DebugSession {
    /// 创建新的调试会话
    pub fn new() -> Self {
        Self {
            breakpoints: Vec::new(),
            watch_points: Vec::new(),
            call_stack: CallStack::new(),
            variables: VariableScope::new(),
            current_data: None,
            execution_state: ExecutionState::Stopped,
        }
    }
}

impl CallStack {
    /// 创建新的调用栈
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            current_frame: 0,
        }
    }
}

impl VariableScope {
    /// 创建新的变量作用域
    pub fn new() -> Self {
        Self {
            global_vars: HashMap::new(),
            local_vars: HashMap::new(),
            current: None,
        }
    }
}

impl CommandHistory {
    /// 创建新的命令历史
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            current_index: 0,
        }
    }

    /// 添加命令到历史
    pub fn add_command(&mut self, command: DebugCommand) {
        self.commands.push(command);
        self.current_index = self.commands.len();
    }
}

impl DebugCommand {
    /// 解析命令字符串
    pub fn parse(input: &str) -> DebugResult<Self> {
        let input = input.trim();

        if let Some(stripped) = input.strip_prefix(':') {
            // 调试器命令
            let parts: Vec<&str> = stripped.split_whitespace().collect();

            match parts.first() {
                Some(&"help") => Ok(DebugCommand::Help),
                Some(&"quit") => Ok(DebugCommand::Quit),
                Some(&"load") => {
                    if let Some(&file) = parts.get(1) {
                        Ok(DebugCommand::Load {
                            file: PathBuf::from(file),
                        })
                    } else {
                        Err(DebugError::InvalidCommand(
                            "load command requires a file path".to_string(),
                        ))
                    }
                }
                Some(&"save") => {
                    if let Some(&file) = parts.get(1) {
                        Ok(DebugCommand::Save {
                            file: PathBuf::from(file),
                        })
                    } else {
                        Err(DebugError::InvalidCommand(
                            "save command requires a file path".to_string(),
                        ))
                    }
                }
                Some(&"inspect") => {
                    if let Some(&path) = parts.get(1) {
                        Ok(DebugCommand::Inspect {
                            path: path.to_string(),
                        })
                    } else {
                        Err(DebugError::InvalidCommand(
                            "inspect command requires a path".to_string(),
                        ))
                    }
                }
                Some(&"run") => {
                    if parts.len() > 1 {
                        let query = parts[1..].join(" ");
                        Ok(DebugCommand::Run { query })
                    } else {
                        Err(DebugError::InvalidCommand(
                            "run command requires a query".to_string(),
                        ))
                    }
                }
                Some(&"eval") => {
                    if parts.len() > 1 {
                        let expression = parts[1..].join(" ");
                        Ok(DebugCommand::Evaluate { expression })
                    } else {
                        Err(DebugError::InvalidCommand(
                            "eval command requires an expression".to_string(),
                        ))
                    }
                }
                Some(cmd) => Err(DebugError::InvalidCommand(format!(
                    "Unknown command: {}",
                    cmd
                ))),
                None => {
                    Err(DebugError::InvalidCommand("Empty command".to_string()))
                }
            }
        } else {
            // 直接查询
            Ok(DebugCommand::Run {
                query: input.to_string(),
            })
        }
    }
}

#[cfg(feature = "interactive-debug")]
impl Default for XQPathDebugger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_session_default() {
        let session = DebugSession::new();
        assert!(session.breakpoints.is_empty());
        assert!(session.watch_points.is_empty());
    }

    #[test]
    fn test_command_history() {
        let mut history = CommandHistory::new();
        history.add_command(DebugCommand::Help);
        assert_eq!(history.commands.len(), 1);
    }

    #[test]
    fn test_command_parsing() {
        assert!(matches!(
            DebugCommand::parse(":help"),
            Ok(DebugCommand::Help)
        ));
        assert!(matches!(
            DebugCommand::parse(":quit"),
            Ok(DebugCommand::Quit)
        ));

        match DebugCommand::parse(":load test.json") {
            Ok(DebugCommand::Load { file }) => {
                assert_eq!(file.to_str().unwrap(), "test.json");
            }
            _ => panic!("Expected Load command"),
        }
    }
}
