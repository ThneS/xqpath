//! # 交互式调试器
//!
//! 提供交互式的XQPath查询调试环境，支持断点、监视点、单步执行等功能。

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::new_without_default)]
#![allow(clippy::io_other_error)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 交互式调试器主结构
#[derive(Debug)]
pub struct XQPathDebugger {
    data_inspector: DataInspector,
    command_history: CommandHistory,
    session: DebugSession,
    query_evaluator: QueryEvaluator,
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
                write!(f, "Invalid command: {cmd}")
            }
            DebugError::FileNotFound(path) => {
                write!(f, "File not found: {path:?}")
            }
            DebugError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            DebugError::EvaluationError(msg) => {
                write!(f, "Evaluation error: {msg}")
            }
            DebugError::IOError(err) => write!(f, "IO error: {err}"),
        }
    }
}

impl std::error::Error for DebugError {}

impl From<std::io::Error> for DebugError {
    fn from(err: std::io::Error) -> Self {
        DebugError::IOError(err)
    }
}

impl XQPathDebugger {
    /// 创建新的调试器实例
    pub fn new() -> Self {
        Self {
            data_inspector: DataInspector::default(),
            command_history: CommandHistory::new(),
            session: DebugSession::new(),
            query_evaluator: QueryEvaluator::new(),
        }
    }

    /// 启动交互式调试会话
    pub fn run(&mut self) -> DebugResult<()> {
        println!("🔍 XQPath Interactive Debugger");
        println!("Type ':help' for available commands, ':quit' to exit\n");

        use std::io::{self, Write};

        loop {
            print!("xqpath> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let line = input.trim();
                    if line.is_empty() {
                        continue;
                    }

                    // 解析命令并添加到历史
                    if let Ok(command) = DebugCommand::parse(line) {
                        self.command_history.add_command(command);
                    }

                    match self.execute_command(line) {
                        Ok(should_continue) => {
                            if !should_continue {
                                break;
                            }
                        }
                        Err(err) => {
                            eprintln!("Error: {err}");
                        }
                    }
                }
                Err(_) => {
                    println!("Input error, exiting...");
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
            DebugCommand::Load { file } => self.load_data_file(file),
            DebugCommand::Save { file } => self.save_data_file(file),
            DebugCommand::Inspect { path } => self.inspect_path(path),
            DebugCommand::Run { query } => self.run_query(query),
            DebugCommand::Evaluate { expression } => {
                self.evaluate_expression(expression)
            }
            DebugCommand::SetBreakpoint { path, condition } => {
                self.set_breakpoint(path, condition)
            }
            DebugCommand::RemoveBreakpoint { id } => self.remove_breakpoint(id),
            DebugCommand::ListBreakpoints => self.list_breakpoints(),
            DebugCommand::SetWatchPoint {
                expression,
                condition,
            } => self.set_watchpoint(expression, condition),
            DebugCommand::RemoveWatchPoint { id } => self.remove_watchpoint(id),
            DebugCommand::ListWatchPoints => self.list_watchpoints(),
            DebugCommand::ListVariables => self.list_variables(),
            DebugCommand::ShowCallStack => self.show_call_stack(),
            DebugCommand::Reset => self.reset_session(),
            _ => {
                println!("⚠️  Command not yet implemented");
                Ok(true)
            }
        }
    }

    /// 加载数据文件
    fn load_data_file(&mut self, file: PathBuf) -> DebugResult<bool> {
        match fs::read_to_string(&file) {
            Ok(content) => {
                match serde_json::from_str::<Value>(&content) {
                    Ok(data) => {
                        self.session.current_data = Some(data.clone());
                        self.data_inspector.inspect_target = Some(data);
                        println!("✅ Successfully loaded: {:?}", file);
                        println!(
                            "📊 Data type: {}",
                            self.get_data_type(&self.session.current_data)
                        );
                        Ok(true)
                    }
                    Err(e) => {
                        // 尝试YAML解析
                        match serde_yaml::from_str::<Value>(&content) {
                            Ok(data) => {
                                self.session.current_data = Some(data.clone());
                                self.data_inspector.inspect_target = Some(data);
                                println!(
                                    "✅ Successfully loaded YAML: {:?}",
                                    file
                                );
                                println!(
                                    "📊 Data type: {}",
                                    self.get_data_type(
                                        &self.session.current_data
                                    )
                                );
                                Ok(true)
                            }
                            Err(_) => {
                                println!("❌ Failed to parse file as JSON or YAML: {}", e);
                                Ok(true)
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to read file {:?}: {}", file, e);
                Ok(true)
            }
        }
    }

    /// 保存数据文件
    fn save_data_file(&mut self, file: PathBuf) -> DebugResult<bool> {
        if let Some(ref data) = self.session.current_data {
            let content = serde_json::to_string_pretty(data).map_err(|e| {
                DebugError::IOError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e,
                ))
            })?;

            fs::write(&file, content)?;
            println!("✅ Data saved to: {:?}", file);
        } else {
            println!("❌ No data loaded to save");
        }
        Ok(true)
    }

    /// 检查路径
    fn inspect_path(&mut self, path: String) -> DebugResult<bool> {
        if let Some(ref data) = self.session.current_data {
            let data_str = serde_json::to_string(data)
                .map_err(|e| DebugError::ParseError(e.to_string()))?;

            match query_one!(&data_str, &path) {
                Ok(result) => {
                    match result {
                        Some(value) => {
                            println!("🔍 Path: {}", path);
                            println!(
                                "📊 Type: {}",
                                self.get_value_type(&value)
                            );
                            println!(
                                "📋 Value: {}",
                                serde_json::to_string_pretty(&value)
                                    .unwrap_or_else(
                                        |_| "Unable to serialize".to_string()
                                    )
                            );

                            // 更新检查器状态
                            self.data_inspector.inspect_target = Some(value);
                            self.data_inspector.inspect_path = Some(path);
                        }
                        None => {
                            println!("❌ Path not found: {}", path);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Query error: {}", e);
                }
            }
        } else {
            println!(
                "❌ No data loaded. Use ':load <file>' to load data first."
            );
        }
        Ok(true)
    }

    /// 运行查询
    fn run_query(&mut self, query_str: String) -> DebugResult<bool> {
        if let Some(ref data) = self.session.current_data {
            let data_str = serde_json::to_string(data)
                .map_err(|e| DebugError::ParseError(e.to_string()))?;

            let start_time = std::time::Instant::now();

            match query!(&data_str, &query_str) {
                Ok(results) => {
                    let duration = start_time.elapsed();

                    println!("✅ Query executed successfully");
                    println!("⏱️  Execution time: {:?}", duration);
                    println!("📊 Results: {} value(s) found", results.len());

                    for (i, result) in results.iter().enumerate() {
                        if i < 10 {
                            // 限制显示前10个结果
                            println!(
                                "  [{}] {}: {}",
                                i + 1,
                                self.get_value_type(result),
                                serde_json::to_string(result).unwrap_or_else(
                                    |_| "Unable to serialize".to_string()
                                )
                            );
                        }
                    }

                    if results.len() > 10 {
                        println!(
                            "  ... and {} more results",
                            results.len() - 10
                        );
                    }

                    // 更新查询历史
                    self.query_evaluator.current_query = Some(query_str);
                    self.query_evaluator.last_result = results.first().cloned();
                }
                Err(e) => {
                    println!("❌ Query error: {}", e);
                }
            }
        } else {
            println!(
                "❌ No data loaded. Use ':load <file>' to load data first."
            );
        }
        Ok(true)
    }

    /// 评估表达式
    fn evaluate_expression(&mut self, expression: String) -> DebugResult<bool> {
        // 对于简单实现，我们将表达式作为查询处理
        self.run_query(expression)
    }

    /// 设置断点
    fn set_breakpoint(
        &mut self,
        path: String,
        condition: Option<String>,
    ) -> DebugResult<bool> {
        let id = self.session.breakpoints.len() as u32 + 1;
        let breakpoint = Breakpoint {
            id,
            path: path.clone(),
            condition,
            enabled: true,
        };

        self.session.breakpoints.push(breakpoint);
        println!("✅ Breakpoint {} set at: {}", id, path);
        Ok(true)
    }

    /// 移除断点
    fn remove_breakpoint(&mut self, id: u32) -> DebugResult<bool> {
        if let Some(pos) =
            self.session.breakpoints.iter().position(|bp| bp.id == id)
        {
            let removed = self.session.breakpoints.remove(pos);
            println!("✅ Removed breakpoint {}: {}", id, removed.path);
        } else {
            println!("❌ Breakpoint {} not found", id);
        }
        Ok(true)
    }

    /// 列出断点
    fn list_breakpoints(&self) -> DebugResult<bool> {
        if self.session.breakpoints.is_empty() {
            println!("📋 No breakpoints set");
        } else {
            println!("📋 Breakpoints:");
            for bp in &self.session.breakpoints {
                let status = if bp.enabled { "✅" } else { "❌" };
                let condition = bp
                    .condition
                    .as_ref()
                    .map(|c| format!(" (condition: {})", c))
                    .unwrap_or_default();
                println!("  {} [{}] {}{}", status, bp.id, bp.path, condition);
            }
        }
        Ok(true)
    }

    /// 设置监视点
    fn set_watchpoint(
        &mut self,
        expression: String,
        condition: Option<String>,
    ) -> DebugResult<bool> {
        let id = self.session.watch_points.len() as u32 + 1;
        let watchpoint = WatchPoint {
            id,
            expression: expression.clone(),
            condition,
            enabled: true,
        };

        self.session.watch_points.push(watchpoint);
        println!("✅ Watchpoint {} set for: {}", id, expression);
        Ok(true)
    }

    /// 移除监视点
    fn remove_watchpoint(&mut self, id: u32) -> DebugResult<bool> {
        if let Some(pos) =
            self.session.watch_points.iter().position(|wp| wp.id == id)
        {
            let removed = self.session.watch_points.remove(pos);
            println!("✅ Removed watchpoint {}: {}", id, removed.expression);
        } else {
            println!("❌ Watchpoint {} not found", id);
        }
        Ok(true)
    }

    /// 列出监视点
    fn list_watchpoints(&self) -> DebugResult<bool> {
        if self.session.watch_points.is_empty() {
            println!("📋 No watchpoints set");
        } else {
            println!("📋 Watchpoints:");
            for wp in &self.session.watch_points {
                let status = if wp.enabled { "✅" } else { "❌" };
                let condition = wp
                    .condition
                    .as_ref()
                    .map(|c| format!(" (condition: {})", c))
                    .unwrap_or_default();
                println!(
                    "  {} [{}] {}{}",
                    status, wp.id, wp.expression, condition
                );
            }
        }
        Ok(true)
    }

    /// 列出变量
    fn list_variables(&self) -> DebugResult<bool> {
        println!("📋 Current Variables:");

        if let Some(ref data) = self.session.current_data {
            println!(
                "  📊 current_data: {} ({} bytes)",
                self.get_value_type(data),
                serde_json::to_string(data).map(|s| s.len()).unwrap_or(0)
            );
        }

        if let Some(ref result) = self.query_evaluator.last_result {
            println!("  📊 last_result: {}", self.get_value_type(result));
        }

        if let Some(ref query) = self.query_evaluator.current_query {
            println!("  📊 current_query: \"{}\"", query);
        }

        Ok(true)
    }

    /// 显示调用栈
    fn show_call_stack(&self) -> DebugResult<bool> {
        println!("📋 Call Stack:");
        if self.session.call_stack.frames.is_empty() {
            println!("  (empty - no active execution)");
        } else {
            for (i, frame) in self.session.call_stack.frames.iter().enumerate()
            {
                let marker = if i == self.session.call_stack.current_frame {
                    "➤"
                } else {
                    " "
                };
                println!(
                    "  {} [{}] {} (line {})",
                    marker, i, frame.function_name, frame.line
                );
                println!("      query: {}", frame.query);
            }
        }
        Ok(true)
    }

    /// 重置会话
    fn reset_session(&mut self) -> DebugResult<bool> {
        self.session = DebugSession::new();
        self.data_inspector = DataInspector::default();
        self.query_evaluator = QueryEvaluator::new();
        println!("✅ Session reset");
        Ok(true)
    }

    /// 获取数据类型描述
    fn get_data_type(&self, data: &Option<Value>) -> String {
        match data {
            Some(value) => self.get_value_type(value),
            None => "none".to_string(),
        }
    }

    /// 获取值类型描述
    fn get_value_type(&self, value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(_) => "boolean".to_string(),
            Value::Number(n) => {
                if n.is_f64() {
                    "number (float)".to_string()
                } else {
                    "number (integer)".to_string()
                }
            }
            Value::String(_) => "string".to_string(),
            Value::Array(arr) => format!("array (length: {})", arr.len()),
            Value::Object(obj) => format!("object (keys: {})", obj.len()),
        }
    }

    /// 显示帮助信息
    fn show_help(&self) {
        println!("🔍 XQPath Interactive Debugger Commands:");
        println!();
        println!("📂 Data Management:");
        println!("  :load <file>             - Load data from JSON/YAML file");
        println!("  :save <file>             - Save current data to file");
        println!();
        println!("🔍 Query & Inspection:");
        println!("  :inspect <path>          - Inspect data at specific path");
        println!("  :run <query>             - Run a query expression");
        println!("  :eval <expression>       - Evaluate an expression");
        println!();
        println!("🔴 Breakpoints:");
        println!("  :bp <path> [condition]   - Set breakpoint at path");
        println!("  :bp-rm <id>              - Remove breakpoint by ID");
        println!("  :bp-list                 - List all breakpoints");
        println!();
        println!("👁️  Watchpoints:");
        println!("  :watch <expr> [condition] - Set watchpoint for expression");
        println!("  :watch-rm <id>           - Remove watchpoint by ID");
        println!("  :watch-list              - List all watchpoints");
        println!();
        println!("📊 Debug Info:");
        println!("  :vars                    - List current variables");
        println!("  :stack                   - Show call stack");
        println!("  :reset                   - Reset debugging session");
        println!();
        println!("🛠️  General:");
        println!("  :help                    - Show this help message");
        println!("  :quit                    - Exit the debugger");
        println!();
        println!("💡 Tip: You can also run queries directly without ':run'");
        println!("    Example: .users[*].name");
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

impl QueryEvaluator {
    /// 创建新的查询求值器
    pub fn new() -> Self {
        Self {
            current_query: None,
            last_result: None,
            evaluation_context: EvaluationContext::new(),
        }
    }
}

impl EvaluationContext {
    /// 创建新的评估上下文
    pub fn new() -> Self {
        Self {
            data: None,
            path: None,
            filters: Vec::new(),
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
                Some(&"help") | Some(&"h") => Ok(DebugCommand::Help),
                Some(&"quit") | Some(&"q") | Some(&"exit") => {
                    Ok(DebugCommand::Quit)
                }
                Some(&"load") | Some(&"l") => {
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
                Some(&"save") | Some(&"s") => {
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
                Some(&"inspect") | Some(&"i") => {
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
                Some(&"run") | Some(&"r") => {
                    if parts.len() > 1 {
                        let query = parts[1..].join(" ");
                        Ok(DebugCommand::Run { query })
                    } else {
                        Err(DebugError::InvalidCommand(
                            "run command requires a query".to_string(),
                        ))
                    }
                }
                Some(&"eval") | Some(&"e") => {
                    if parts.len() > 1 {
                        let expression = parts[1..].join(" ");
                        Ok(DebugCommand::Evaluate { expression })
                    } else {
                        Err(DebugError::InvalidCommand(
                            "eval command requires an expression".to_string(),
                        ))
                    }
                }
                Some(&"bp") => {
                    if let Some(&path) = parts.get(1) {
                        let condition = if parts.len() > 2 {
                            Some(parts[2..].join(" "))
                        } else {
                            None
                        };
                        Ok(DebugCommand::SetBreakpoint {
                            path: path.to_string(),
                            condition,
                        })
                    } else {
                        Err(DebugError::InvalidCommand(
                            "bp command requires a path".to_string(),
                        ))
                    }
                }
                Some(&"bp-rm") => {
                    if let Some(&id_str) = parts.get(1) {
                        match id_str.parse::<u32>() {
                            Ok(id) => Ok(DebugCommand::RemoveBreakpoint { id }),
                            Err(_) => Err(DebugError::InvalidCommand(
                                "bp-rm command requires a valid ID number"
                                    .to_string(),
                            )),
                        }
                    } else {
                        Err(DebugError::InvalidCommand(
                            "bp-rm command requires an ID".to_string(),
                        ))
                    }
                }
                Some(&"bp-list") => Ok(DebugCommand::ListBreakpoints),
                Some(&"watch") => {
                    if let Some(&expr) = parts.get(1) {
                        let condition = if parts.len() > 2 {
                            Some(parts[2..].join(" "))
                        } else {
                            None
                        };
                        Ok(DebugCommand::SetWatchPoint {
                            expression: expr.to_string(),
                            condition,
                        })
                    } else {
                        Err(DebugError::InvalidCommand(
                            "watch command requires an expression".to_string(),
                        ))
                    }
                }
                Some(&"watch-rm") => {
                    if let Some(&id_str) = parts.get(1) {
                        match id_str.parse::<u32>() {
                            Ok(id) => Ok(DebugCommand::RemoveWatchPoint { id }),
                            Err(_) => Err(DebugError::InvalidCommand(
                                "watch-rm command requires a valid ID number"
                                    .to_string(),
                            )),
                        }
                    } else {
                        Err(DebugError::InvalidCommand(
                            "watch-rm command requires an ID".to_string(),
                        ))
                    }
                }
                Some(&"watch-list") => Ok(DebugCommand::ListWatchPoints),
                Some(&"vars") | Some(&"v") => Ok(DebugCommand::ListVariables),
                Some(&"stack") => Ok(DebugCommand::ShowCallStack),
                Some(&"reset") => Ok(DebugCommand::Reset),
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
