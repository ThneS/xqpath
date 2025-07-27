use datapath::{
    detect_format, extract, parse_path, JsonFormat, ValueFormat, YamlFormat,
};
use std::env;
use std::fs;
use std::io::{self, Read};

#[cfg(feature = "update")]
use datapath::update;

/// CLI 错误类型
#[derive(Debug)]
enum CliError {
    InvalidArguments(String),
    IoError(io::Error),
    ParseError(String),
    ExtractError(String),
    #[cfg(feature = "update")]
    UpdateError(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::InvalidArguments(msg) => {
                write!(f, "Invalid arguments: {msg}")
            }
            CliError::IoError(e) => write!(f, "IO error: {e}"),
            CliError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            CliError::ExtractError(msg) => write!(f, "Extract error: {msg}"),
            #[cfg(feature = "update")]
            CliError::UpdateError(msg) => write!(f, "Update error: {msg}"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<io::Error> for CliError {
    fn from(error: io::Error) -> Self {
        CliError::IoError(error)
    }
}

/// CLI 配置结构
#[derive(Debug)]
struct CliConfig {
    command: Command,
    file_path: Option<String>,
    path_expression: String,
    #[cfg(feature = "update")]
    new_value: Option<String>,
    output_format: Option<String>,
}

/// CLI 命令枚举
#[derive(Debug)]
enum Command {
    Get,
    #[cfg(feature = "update")]
    Set,
    Exists,
    Type,
    Count,
    Extract,
    Help,
    Version,
}

fn main() {
    let config = match parse_args() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error: {e}");
            print_usage();
            std::process::exit(1);
        }
    };

    let result = match config.command {
        Command::Get => run_get(&config),
        #[cfg(feature = "update")]
        Command::Set => run_set(&config),
        Command::Exists => run_exists(&config),
        Command::Type => run_type(&config),
        Command::Count => run_count(&config),
        Command::Extract => run_extract(&config),
        Command::Help => {
            print_usage();
            Ok(())
        }
        Command::Version => {
            println!("datapath {}", datapath::VERSION);
            println!(
                "Features: {}",
                if datapath::has_update_feature() {
                    "update"
                } else {
                    "basic"
                }
            );
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

/// 解析命令行参数
fn parse_args() -> Result<CliConfig, CliError> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(CliError::InvalidArguments(
            "No command specified".to_string(),
        ));
    }

    let command = match args[1].as_str() {
        "get" => Command::Get,
        #[cfg(feature = "update")]
        "set" => Command::Set,
        "exists" => Command::Exists,
        "type" => Command::Type,
        "count" => Command::Count,
        "extract" => Command::Extract,
        "help" | "--help" | "-h" => Command::Help,
        "version" | "--version" | "-V" => Command::Version,
        _ => {
            return Err(CliError::InvalidArguments(format!(
                "Unknown command: {}",
                args[1]
            )))
        }
    };

    let mut file_path = None;
    let mut path_expression = String::new();
    #[cfg(feature = "update")]
    let mut new_value = None;
    let mut output_format = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-f" | "--file" => {
                if i + 1 >= args.len() {
                    return Err(CliError::InvalidArguments(
                        "Missing file path".to_string(),
                    ));
                }
                file_path = Some(args[i + 1].clone());
                i += 2;
            }
            "-p" | "--path" => {
                if i + 1 >= args.len() {
                    return Err(CliError::InvalidArguments(
                        "Missing path expression".to_string(),
                    ));
                }
                path_expression = args[i + 1].clone();
                i += 2;
            }
            #[cfg(feature = "update")]
            "-v" | "--value" => {
                if i + 1 >= args.len() {
                    return Err(CliError::InvalidArguments(
                        "Missing value".to_string(),
                    ));
                }
                new_value = Some(args[i + 1].clone());
                i += 2;
            }
            "-o" | "--output" => {
                if i + 1 >= args.len() {
                    return Err(CliError::InvalidArguments(
                        "Missing output format".to_string(),
                    ));
                }
                output_format = Some(args[i + 1].clone());
                i += 2;
            }
            _ => {
                return Err(CliError::InvalidArguments(format!(
                    "Unknown option: {}",
                    args[i]
                )));
            }
        }
    }

    // 只有特定命令需要路径表达式
    match command {
        Command::Help | Command::Version => {
            // help 和 version 命令不需要路径表达式
        }
        _ => {
            if path_expression.is_empty() {
                return Err(CliError::InvalidArguments(
                    "Path expression is required".to_string(),
                ));
            }
        }
    }

    Ok(CliConfig {
        command,
        file_path,
        path_expression,
        #[cfg(feature = "update")]
        new_value,
        output_format,
    })
}

/// 读取输入数据
fn read_input(file_path: Option<&str>) -> Result<String, CliError> {
    match file_path {
        Some(path) => fs::read_to_string(path).map_err(CliError::from),
        None => {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            Ok(input)
        }
    }
}

/// 执行 get 命令
fn run_get(config: &CliConfig) -> Result<(), CliError> {
    let input_data = read_input(config.file_path.as_deref())?;

    // 自动检测格式
    let format = detect_format(&input_data)
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    // 解析数据
    let parsed_data = format
        .parse(&input_data)
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    // 解析路径
    let path = parse_path(&config.path_expression)
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    // 提取值
    let values = extract(&parsed_data, &path)
        .map_err(|e| CliError::ExtractError(e.to_string()))?;

    // 输出结果
    let output_format_name =
        config.output_format.as_deref().unwrap_or(format.name());
    let output_format = get_output_format(output_format_name)?;

    for (i, value) in values.iter().enumerate() {
        if i > 0 {
            println!(); // 在多个结果之间添加空行
        }
        let output = output_format
            .to_string(value)
            .map_err(|e| CliError::ParseError(e.to_string()))?;
        print!("{output}");
    }

    if !values.is_empty() {
        println!(); // 确保最后有换行
    }

    Ok(())
}

/// 执行 set 命令
#[cfg(feature = "update")]
fn run_set(config: &CliConfig) -> Result<(), CliError> {
    let input_data = read_input(config.file_path.as_deref())?;

    let new_value_str = config.new_value.as_ref().ok_or_else(|| {
        CliError::InvalidArguments(
            "Value is required for set command".to_string(),
        )
    })?;

    // 自动检测格式
    let format = detect_format(&input_data)
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    // 解析数据
    let mut parsed_data = format
        .parse(&input_data)
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    // 解析路径
    let path = parse_path(&config.path_expression)
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    // 解析新值（假设为 JSON 格式）
    let new_value: serde_json::Value = serde_json::from_str(new_value_str)
        .map_err(|e| {
            CliError::UpdateError(format!("Invalid JSON value: {e}"))
        })?;

    // 更新值
    update(&mut parsed_data, &path, new_value)
        .map_err(|e| CliError::UpdateError(e.to_string()))?;

    // 输出结果
    let output_format_name =
        config.output_format.as_deref().unwrap_or(format.name());
    let output_format = get_output_format(output_format_name)?;

    let output = output_format
        .to_string(&parsed_data)
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    print!("{output}");

    Ok(())
}

/// 执行 exists 命令
fn run_exists(config: &CliConfig) -> Result<(), CliError> {
    let input_data = read_input(config.file_path.as_deref())?;

    // 自动检测格式
    let format = detect_format(&input_data)
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    // 解析数据
    let parsed_data = format
        .parse(&input_data)
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    // 解析路径
    let path = parse_path(&config.path_expression)
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    // 检查路径是否存在
    let values = extract(&parsed_data, &path)
        .map_err(|e| CliError::ExtractError(e.to_string()))?;

    let exists = !values.is_empty();
    println!("{exists}");

    Ok(())
}

/// 执行 type 命令
fn run_type(config: &CliConfig) -> Result<(), CliError> {
    let input_data = read_input(config.file_path.as_deref())?;

    // 自动检测格式
    let format = detect_format(&input_data)
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    // 解析数据
    let parsed_data = format
        .parse(&input_data)
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    // 解析路径
    let path = parse_path(&config.path_expression)
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    // 获取值并输出类型
    let values = extract(&parsed_data, &path)
        .map_err(|e| CliError::ExtractError(e.to_string()))?;

    for value in values {
        let type_name = match value {
            serde_json::Value::Null => "null",
            serde_json::Value::Bool(_) => "boolean",
            serde_json::Value::Number(_) => "number",
            serde_json::Value::String(_) => "string",
            serde_json::Value::Array(_) => "array",
            serde_json::Value::Object(_) => "object",
        };
        println!("{type_name}");
    }

    Ok(())
}

/// 执行 count 命令
fn run_count(config: &CliConfig) -> Result<(), CliError> {
    let input_data = read_input(config.file_path.as_deref())?;

    // 自动检测格式
    let format = detect_format(&input_data)
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    // 解析数据
    let parsed_data = format
        .parse(&input_data)
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    // 解析路径
    let path = parse_path(&config.path_expression)
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    // 获取值并计数
    let values = extract(&parsed_data, &path)
        .map_err(|e| CliError::ExtractError(e.to_string()))?;

    println!("{}", values.len());

    Ok(())
}

/// 执行 extract 命令
fn run_extract(config: &CliConfig) -> Result<(), CliError> {
    let input_data = read_input(config.file_path.as_deref())?;

    // 自动检测格式
    let format = detect_format(&input_data)
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    // 解析数据
    let parsed_data = format
        .parse(&input_data)
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    // 解析路径
    let path = parse_path(&config.path_expression)
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    // 提取值（与 get 命令相同）
    let values = extract(&parsed_data, &path)
        .map_err(|e| CliError::ExtractError(e.to_string()))?;

    // 输出结果
    let output_format_name =
        config.output_format.as_deref().unwrap_or(format.name());
    let output_format = get_output_format(output_format_name)?;

    for (i, value) in values.iter().enumerate() {
        if i > 0 {
            println!(); // 在多个结果之间添加空行
        }
        let output = output_format
            .to_string(value)
            .map_err(|e| CliError::ParseError(e.to_string()))?;
        print!("{output}");
    }

    if !values.is_empty() {
        println!(); // 确保最后有换行
    }

    Ok(())
}

/// 获取输出格式
fn get_output_format(
    format_name: &str,
) -> Result<Box<dyn ValueFormat>, CliError> {
    match format_name.to_lowercase().as_str() {
        "json" => Ok(Box::new(JsonFormat)),
        "yaml" | "yml" => Ok(Box::new(YamlFormat)),
        _ => Err(CliError::InvalidArguments(format!(
            "Unsupported output format: {format_name}"
        ))),
    }
}

/// 打印使用说明
fn print_usage() {
    println!("DataPath - A minimal jq-like path extractor and updater for structured data");
    println!();
    println!("USAGE:");
    println!("    datapath <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    get        Extract values using path expression");
    #[cfg(feature = "update")]
    println!("    set        Update values using path expression");
    println!("    exists     Check if path exists (outputs true/false)");
    println!("    type       Get type of value at path");
    println!("    count      Count number of values at path");
    println!("    extract    Extract values (alias for get)");
    println!("    help       Print this help message");
    println!("    version    Print version information");
    println!();
    println!("OPTIONS:");
    println!("    -f, --file <FILE>     Input file (reads from stdin if not specified)");
    println!("    -p, --path <PATH>     Path expression (jq-style syntax)");
    #[cfg(feature = "update")]
    println!(
        "    -v, --value <VALUE>   New value for set command (JSON format)"
    );
    println!("    -o, --output <FORMAT> Output format (json, yaml)");
    println!();
    println!("EXAMPLES:");
    println!("    # Extract field from YAML file");
    println!("    datapath get -f config.yaml -p 'spec.containers[0].image'");
    println!();
    println!("    # Extract from stdin with wildcard");
    println!("    cat data.json | datapath get -p 'users[*].name'");
    println!();
    #[cfg(feature = "update")]
    {
        println!("    # Update field value");
        println!("    datapath set -f config.yaml -p 'version' -v '\"2.0\"'");
        println!();
    }
    println!("PATH SYNTAX:");
    println!("    .field       Access object field");
    println!("    [index]      Access array element");
    println!("    *            Wildcard (any field/element)");
    println!("    **           Recursive wildcard");
    println!(
        "    | type       Type filter (string, number, boolean, array, object)"
    );
    println!();
    println!(
        "For more information, visit: https://github.com/Thneonl/datapath"
    );
}
