use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use serde_json::Value;

use xqpath::{
    detect_format, extract, parse_path, JsonFormat, ValueFormat, YamlFormat,
};

#[cfg(feature = "update")]
use xqpath::update;

/// XQPath - A minimal jq-like path extractor and updater for structured data
#[derive(Parser)]
#[command(name = "xqpath")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(
    about = "A minimal jq-like path extractor and updater for structured data in Rust"
)]
#[command(long_about = None)]
struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,

    // 全局调试选项 (v1.4.1+)
    /// Enable debug mode
    #[cfg(feature = "debug")]
    #[arg(long, global = true)]
    debug: bool,

    /// Set log level
    #[cfg(feature = "debug")]
    #[arg(long, global = true, value_enum)]
    log_level: Option<DebugLogLevel>,

    /// Log to file
    #[cfg(feature = "debug")]
    #[arg(long, global = true, value_name = "FILE")]
    log_file: Option<PathBuf>,

    /// Show execution timing
    #[cfg(feature = "debug")]
    #[arg(long, global = true)]
    timing: bool,

    /// Enable path tracing
    #[cfg(feature = "debug")]
    #[arg(long, global = true)]
    trace_path: bool,

    /// Show memory statistics
    #[cfg(feature = "debug")]
    #[arg(long, global = true)]
    memory_stats: bool,

    /// Enable profiling
    #[cfg(feature = "profiling")]
    #[arg(long, global = true)]
    profile: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract values using path expression (default command)
    Get {
        /// Path expression (jq-style syntax)
        path: String,

        /// Input file (reads from stdin if not specified)
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Auto)]
        output: OutputFormat,

        /// Enable pretty printing for JSON output
        #[arg(long)]
        pretty: bool,

        /// Disable colored output
        #[arg(long)]
        no_color: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Update values using path expression
    #[cfg(feature = "update")]
    Set {
        /// Path expression (jq-style syntax)
        path: String,
        /// New value (JSON format)
        value: String,

        /// Input file (reads from stdin if not specified)
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Auto)]
        output: OutputFormat,

        /// Enable pretty printing for JSON output
        #[arg(long)]
        pretty: bool,

        /// Disable colored output
        #[arg(long)]
        no_color: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Check if path exists
    Exists {
        /// Path expression (jq-style syntax)
        path: String,

        /// Input file (reads from stdin if not specified)
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,

        /// Disable colored output
        #[arg(long)]
        no_color: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Get type of value at path
    Type {
        /// Path expression (jq-style syntax)
        path: String,

        /// Input file (reads from stdin if not specified)
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,

        /// Disable colored output
        #[arg(long)]
        no_color: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Count number of values at path
    Count {
        /// Path expression (jq-style syntax)
        path: String,

        /// Input file (reads from stdin if not specified)
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,

        /// Disable colored output
        #[arg(long)]
        no_color: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Get length of value at path (for arrays, objects, strings)
    Length {
        /// Path expression (jq-style syntax)
        path: String,

        /// Input file (reads from stdin if not specified)
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,

        /// Disable colored output
        #[arg(long)]
        no_color: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Get keys of object at path
    Keys {
        /// Path expression (jq-style syntax)
        path: String,

        /// Input file (reads from stdin if not specified)
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Auto)]
        output: OutputFormat,

        /// Enable pretty printing for JSON output
        #[arg(long)]
        pretty: bool,

        /// Disable colored output
        #[arg(long)]
        no_color: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Interactive mode for exploring data
    Interactive {
        /// Input file (reads from stdin if not specified)
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,
    },

    /// Validate data format
    Validate {
        /// Input file (reads from stdin if not specified)
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,

        /// Disable colored output
        #[arg(long)]
        no_color: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Convert between formats
    Convert {
        /// Target format
        #[arg(value_enum)]
        to: OutputFormat,

        /// Input file (reads from stdin if not specified)
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,

        /// Enable pretty printing for JSON output
        #[arg(long)]
        pretty: bool,

        /// Disable colored output
        #[arg(long)]
        no_color: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show examples of usage
    Examples,

    // 调试命令 (v1.4.1+)
    /// Debug mode execution with detailed tracing
    #[cfg(feature = "debug")]
    Debug {
        /// Path expression (jq-style syntax)
        path: String,

        /// Input file (reads from stdin if not specified)
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,

        /// Enable interactive debug mode
        #[arg(long)]
        interactive: bool,
    },

    /// Trace path execution
    #[cfg(feature = "debug")]
    Trace {
        /// Path expression (jq-style syntax)
        path: String,

        /// Input file (reads from stdin if not specified)
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,

        /// Show detailed execution steps
        #[arg(long)]
        detailed: bool,
    },

    // 性能分析命令 (v1.4.2+)
    /// Performance profiling with detailed metrics
    #[cfg(feature = "profiling")]
    Profile {
        /// Path expression (jq-style syntax)
        path: String,

        /// Input file (reads from stdin if not specified)
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,

        /// Generate HTML report
        #[arg(long)]
        html: bool,

        /// Output file for the report
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Include memory analysis
        #[arg(long)]
        memory: bool,

        /// Show optimization hints
        #[arg(long)]
        hints: bool,
    },

    /// Benchmark query performance
    #[cfg(feature = "benchmark")]
    Benchmark {
        /// Path expression (jq-style syntax)
        path: String,

        /// Input file (reads from stdin if not specified)
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,

        /// Number of iterations
        #[arg(short, long, default_value = "100")]
        iterations: usize,

        /// Warmup iterations
        #[arg(long, default_value = "10")]
        warmup: usize,

        /// Output format for benchmark results
        #[arg(long, value_enum, default_value_t = BenchmarkOutputFormat::Text)]
        format: BenchmarkOutputFormat,

        /// Output file for the benchmark results
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Compare with baseline file
        #[arg(long, value_name = "FILE")]
        baseline: Option<PathBuf>,
    },

    /// Monitor performance metrics in real-time
    #[cfg(feature = "profiling")]
    Monitor {
        /// Path expression (jq-style syntax)
        path: String,

        /// Input file (reads from stdin if not specified)
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,

        /// Monitoring duration in seconds
        #[arg(short, long, default_value = "60")]
        duration: u64,

        /// Update interval in milliseconds
        #[arg(long, default_value = "1000")]
        interval: u64,

        /// Generate continuous reports
        #[arg(long)]
        continuous: bool,
    },

    /// Configuration management commands (v1.4.3+)
    #[cfg(feature = "config-management")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Interactive debugger (v1.4.3+)
    #[cfg(feature = "interactive-debug")]
    InteractiveDebug {
        /// Input file to load (optional)
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,
    },
}

// 调试日志级别
#[cfg(feature = "debug")]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum DebugLogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

// 配置管理命令 (v1.4.3+)
#[cfg(feature = "config-management")]
#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,

    /// Set a configuration value
    Set {
        /// Configuration key (e.g., "debug.level")
        key: String,
        /// Configuration value
        value: String,
    },

    /// Reset configuration to defaults
    Reset,

    /// Create configuration template
    Template {
        /// Template name
        name: String,
    },

    /// Create configuration profile
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },

    /// Show configuration audit log
    Audit,

    /// Migrate configuration files
    Migrate,
}

#[cfg(feature = "config-management")]
#[derive(Subcommand)]
enum ProfileAction {
    /// Create new profile
    Create {
        /// Profile name
        name: String,
    },

    /// Switch to profile
    Switch {
        /// Profile name
        name: String,
    },

    /// List all profiles
    List,
}

// 基准测试输出格式 (v1.4.2+)
#[cfg(feature = "benchmark")]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum BenchmarkOutputFormat {
    /// Plain text output
    Text,
    /// JSON format
    Json,
    /// HTML report
    Html,
    /// CSV format
    Csv,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    /// Auto-detect from input
    Auto,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// Pretty JSON format
    JsonPretty,
    /// Compact output (single line)
    Compact,
}

impl OutputFormat {
    // 移除未使用的get_formatter方法
}

fn main() {
    let cli = Cli::parse();

    // 初始化调试系统 (v1.4.1+)
    #[cfg(feature = "debug")]
    initialize_debug_system(&cli);

    // 设置颜色输出 (针对每个命令的no_color参数)
    let no_color = match &cli.command {
        Commands::Get { no_color, .. }
        | Commands::Exists { no_color, .. }
        | Commands::Type { no_color, .. }
        | Commands::Count { no_color, .. }
        | Commands::Length { no_color, .. }
        | Commands::Keys { no_color, .. }
        | Commands::Validate { no_color, .. }
        | Commands::Convert { no_color, .. } => *no_color,
        #[cfg(feature = "update")]
        Commands::Set { no_color, .. } => *no_color,
        _ => false,
    };

    if no_color {
        colored::control::set_override(false);
    }

    let result = run_command(&cli);

    if let Err(e) = result {
        let verbose = match &cli.command {
            Commands::Get { verbose, .. }
            | Commands::Exists { verbose, .. }
            | Commands::Type { verbose, .. }
            | Commands::Count { verbose, .. }
            | Commands::Length { verbose, .. }
            | Commands::Keys { verbose, .. }
            | Commands::Validate { verbose, .. }
            | Commands::Convert { verbose, .. } => *verbose,
            #[cfg(feature = "update")]
            Commands::Set { verbose, .. } => *verbose,
            _ => false,
        };

        if verbose {
            eprintln!("{} {:#}", "Error:".red().bold(), e);
        } else {
            eprintln!("{} {}", "Error:".red().bold(), e);
        }
        std::process::exit(1);
    }
}

// v1.4.1 调试系统初始化

#[cfg(feature = "debug")]
fn initialize_debug_system(cli: &Cli) {
    // 设置日志级别
    if let Some(level) = cli.log_level {
        match level {
            DebugLogLevel::Trace => println!("🔍 Debug level set to: TRACE"),
            DebugLogLevel::Debug => println!("🔍 Debug level set to: DEBUG"),
            DebugLogLevel::Info => println!("🔍 Debug level set to: INFO"),
            DebugLogLevel::Warn => println!("🔍 Debug level set to: WARN"),
            DebugLogLevel::Error => println!("🔍 Debug level set to: ERROR"),
        }
    }

    // 设置日志文件
    if let Some(log_file) = &cli.log_file {
        println!("📝 Logging to file: {}", log_file.display());
    }

    // 启用调试模式
    if cli.debug {
        println!("🐛 Debug mode enabled");
    }

    // 启用时间统计
    if cli.timing {
        println!("⏱️  Timing enabled");
    }

    // 启用路径跟踪
    if cli.trace_path {
        println!("📊 Path tracing enabled");
    }

    // 启用内存统计
    if cli.memory_stats {
        println!("💾 Memory statistics enabled");
    }
}

fn run_command(cli: &Cli) -> Result<()> {
    match &cli.command {
        Commands::Get {
            path,
            file,
            output,
            pretty,
            verbose,
            ..
        } => run_get(path, file.as_ref(), output, *pretty, *verbose),
        #[cfg(feature = "update")]
        Commands::Set {
            path,
            value,
            file,
            output,
            pretty,
            verbose,
            ..
        } => run_set(path, value, file.as_ref(), output, *pretty, *verbose),
        Commands::Exists {
            path,
            file,
            verbose,
            ..
        } => run_exists(path, file.as_ref(), *verbose),
        Commands::Type {
            path,
            file,
            verbose,
            ..
        } => run_type(path, file.as_ref(), *verbose),
        Commands::Count {
            path,
            file,
            verbose,
            ..
        } => run_count(path, file.as_ref(), *verbose),
        Commands::Length {
            path,
            file,
            verbose,
            ..
        } => run_length(path, file.as_ref(), *verbose),
        Commands::Keys {
            path,
            file,
            output,
            pretty,
            verbose,
            ..
        } => run_keys(path, file.as_ref(), output, *pretty, *verbose),
        Commands::Interactive { file } => run_interactive(file.as_ref()),
        Commands::Validate { file, verbose, .. } => {
            run_validate(file.as_ref(), *verbose)
        }
        Commands::Convert {
            to,
            file,
            pretty,
            verbose,
            ..
        } => run_convert(to, file.as_ref(), *pretty, *verbose),
        Commands::Examples => run_examples(),
        #[cfg(feature = "debug")]
        Commands::Debug {
            path,
            file,
            interactive,
        } => run_debug(path, file.as_ref(), *interactive),
        #[cfg(feature = "debug")]
        Commands::Trace {
            path,
            file,
            detailed,
        } => run_trace(path, file.as_ref(), *detailed),
        // v1.4.2 性能分析命令
        #[cfg(feature = "profiling")]
        Commands::Profile {
            path,
            file,
            html,
            output,
            memory,
            hints,
        } => run_profile(
            path,
            file.as_ref(),
            *html,
            output.as_ref(),
            *memory,
            *hints,
        ),
        #[cfg(feature = "benchmark")]
        Commands::Benchmark {
            path,
            file,
            iterations,
            warmup,
            format,
            output,
            baseline,
        } => run_benchmark(
            path,
            file.as_ref(),
            *iterations,
            *warmup,
            format,
            output.as_ref(),
            baseline.as_ref(),
        ),
        #[cfg(feature = "profiling")]
        Commands::Monitor {
            path,
            file,
            duration,
            interval,
            continuous,
        } => {
            run_monitor(path, file.as_ref(), *duration, *interval, *continuous)
        }
        // v1.4.3 配置管理命令
        #[cfg(feature = "config-management")]
        Commands::Config { action } => run_config(action),
        // v1.4.3 交互式调试器命令
        #[cfg(feature = "interactive-debug")]
        Commands::InteractiveDebug { file } => {
            run_interactive_debugger(file.as_ref())
        }
    }
}

fn read_input(file: Option<&PathBuf>) -> Result<String> {
    match file {
        Some(path) => fs::read_to_string(path).with_context(|| {
            format!("Failed to read file: {}", path.display())
        }),
        None => {
            let mut input = String::new();
            io::stdin()
                .read_to_string(&mut input)
                .context("Failed to read from stdin")?;
            Ok(input)
        }
    }
}

fn parse_and_extract(
    input: &str,
    path: &str,
) -> Result<(Box<dyn ValueFormat>, Vec<Value>)> {
    let format =
        detect_format(input).context("Failed to detect input format")?;

    let parsed_data =
        format.parse(input).context("Failed to parse input data")?;

    let path_obj =
        parse_path(path).context("Failed to parse path expression")?;

    let values =
        extract(&parsed_data, &path_obj).context("Failed to extract values")?;

    let owned_values: Vec<Value> = values.into_iter().cloned().collect();

    Ok((format, owned_values))
}

fn output_values(
    values: &[Value],
    format: &dyn ValueFormat,
    output: &OutputFormat,
    pretty: bool,
) -> Result<()> {
    let output_format = match output {
        OutputFormat::Auto => format.name(),
        _ => match output {
            OutputFormat::Json | OutputFormat::Compact => "json",
            OutputFormat::JsonPretty => "json",
            OutputFormat::Yaml => "yaml",
            OutputFormat::Auto => unreachable!(),
        },
    };

    let formatter = get_output_format(output_format)?;

    for (i, value) in values.iter().enumerate() {
        if i > 0 {
            println!();
        }

        let output_str = if pretty
            && matches!(output, OutputFormat::JsonPretty | OutputFormat::Auto)
        {
            serde_json::to_string_pretty(value)
                .context("Failed to format output")?
        } else if matches!(output, OutputFormat::Compact) {
            serde_json::to_string(value).context("Failed to format output")?
        } else {
            formatter
                .to_string(value)
                .context("Failed to format output")?
        };

        print!("{output_str}");
    }

    if !values.is_empty() {
        println!();
    }

    Ok(())
}

fn run_get(
    path: &str,
    file: Option<&PathBuf>,
    output: &OutputFormat,
    pretty: bool,
    verbose: bool,
) -> Result<()> {
    let start_time = std::time::Instant::now();
    let input = read_input(file)?;
    let (format, values) = parse_and_extract(&input, path)?;

    if verbose {
        eprintln!("{} Found {} value(s)", "Info:".blue().bold(), values.len());
        let duration = start_time.elapsed();
        eprintln!(
            "{} Execution time: {:?}",
            "Timing:".green().bold(),
            duration
        );
    }

    output_values(&values, format.as_ref(), output, pretty)?;
    Ok(())
}

#[cfg(feature = "update")]
fn run_set(
    path: &str,
    new_value_str: &str,
    file: Option<&PathBuf>,
    output: &OutputFormat,
    _pretty: bool,
    _verbose: bool,
) -> Result<()> {
    let input = read_input(file)?;

    let format =
        detect_format(&input).context("Failed to detect input format")?;

    let mut parsed_data =
        format.parse(&input).context("Failed to parse input data")?;

    let path_obj =
        parse_path(path).context("Failed to parse path expression")?;

    let new_value: serde_json::Value = serde_json::from_str(new_value_str)
        .context("Invalid JSON value for update")?;

    update(&mut parsed_data, &path_obj, new_value)
        .context("Failed to update value")?;

    let output_format = match output {
        OutputFormat::Auto => format.name(),
        _ => match output {
            OutputFormat::Json | OutputFormat::Compact => "json",
            OutputFormat::JsonPretty => "json",
            OutputFormat::Yaml => "yaml",
            OutputFormat::Auto => unreachable!(),
        },
    };

    let formatter = get_output_format(output_format)?;
    let output_str = formatter
        .to_string(&parsed_data)
        .context("Failed to format output")?;

    print!("{output_str}");
    Ok(())
}

fn run_exists(path: &str, file: Option<&PathBuf>, verbose: bool) -> Result<()> {
    let input = read_input(file)?;
    let (_, values) = parse_and_extract(&input, path)?;

    let exists = !values.is_empty();

    if verbose {
        if exists {
            println!("{} Path exists", "✓".green().bold());
        } else {
            println!("{} Path does not exist", "✗".red().bold());
        }
    } else {
        println!("{exists}");
    }

    Ok(())
}

fn run_type(path: &str, file: Option<&PathBuf>, verbose: bool) -> Result<()> {
    let input = read_input(file)?;
    let (_, values) = parse_and_extract(&input, path)?;

    for value in values {
        let type_name = match value {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        };

        if verbose {
            println!("{} {}", "Type:".blue().bold(), type_name);
        } else {
            println!("{type_name}");
        }
    }

    Ok(())
}

fn run_count(path: &str, file: Option<&PathBuf>, verbose: bool) -> Result<()> {
    let input = read_input(file)?;
    let (_, values) = parse_and_extract(&input, path)?;

    if verbose {
        println!("{} {} value(s) found", "Count:".blue().bold(), values.len());
    } else {
        println!("{}", values.len());
    }

    Ok(())
}

fn run_length(path: &str, file: Option<&PathBuf>, verbose: bool) -> Result<()> {
    let input = read_input(file)?;
    let (_, values) = parse_and_extract(&input, path)?;

    for value in values {
        let length = match &value {
            Value::Array(arr) => Some(arr.len()),
            Value::Object(obj) => Some(obj.len()),
            Value::String(s) => Some(s.len()),
            _ => None,
        };

        match length {
            Some(len) => {
                if verbose {
                    println!("{} {}", "Length:".blue().bold(), len);
                } else {
                    println!("{len}");
                }
            }
            None => {
                if verbose {
                    println!(
                        "{} Value has no length property",
                        "Info:".yellow().bold()
                    );
                } else {
                    println!("null");
                }
            }
        }
    }

    Ok(())
}

fn run_keys(
    path: &str,
    file: Option<&PathBuf>,
    output: &OutputFormat,
    pretty: bool,
    verbose: bool,
) -> Result<()> {
    let input = read_input(file)?;
    let (format, values) = parse_and_extract(&input, path)?;

    for value in values {
        match &value {
            Value::Object(obj) => {
                let keys: Vec<Value> =
                    obj.keys().map(|k| Value::String(k.clone())).collect();
                let keys_array = Value::Array(keys);
                output_values(&[keys_array], format.as_ref(), output, pretty)?;
            }
            Value::Array(arr) => {
                let indices: Vec<Value> = (0..arr.len())
                    .map(|i| Value::Number(serde_json::Number::from(i)))
                    .collect();
                let indices_array = Value::Array(indices);
                output_values(
                    &[indices_array],
                    format.as_ref(),
                    output,
                    pretty,
                )?;
            }
            _ => {
                if verbose {
                    println!(
                        "{} Value is not an object or array",
                        "Info:".yellow().bold()
                    );
                } else {
                    println!("null");
                }
            }
        }
    }

    Ok(())
}

fn run_interactive(_file: Option<&PathBuf>) -> Result<()> {
    println!(
        "{}",
        "🚀 Interactive mode is not yet implemented".yellow().bold()
    );
    println!("This feature will be available in a future release.");
    println!("For now, you can use the individual commands like 'get', 'exists', etc.");
    Ok(())
}

fn run_validate(file: Option<&PathBuf>, verbose: bool) -> Result<()> {
    let input = read_input(file)?;

    match detect_format(&input) {
        Ok(format) => match format.parse(&input) {
            Ok(_) => {
                if verbose {
                    println!(
                        "{} Valid {} format",
                        "✓".green().bold(),
                        format.name()
                    );
                } else {
                    println!("valid");
                }
            }
            Err(e) => {
                if verbose {
                    println!(
                        "{} Invalid {}: {}",
                        "✗".red().bold(),
                        format.name(),
                        e
                    );
                } else {
                    println!("invalid");
                }
                return Err(anyhow::anyhow!("Validation failed: {}", e));
            }
        },
        Err(e) => {
            if verbose {
                println!("{} Cannot detect format: {}", "✗".red().bold(), e);
            } else {
                println!("unknown");
            }
            return Err(anyhow::anyhow!("Format detection failed: {}", e));
        }
    }

    Ok(())
}

fn run_convert(
    to: &OutputFormat,
    file: Option<&PathBuf>,
    pretty: bool,
    _verbose: bool,
) -> Result<()> {
    let input = read_input(file)?;

    let input_format =
        detect_format(&input).context("Failed to detect input format")?;

    let parsed_data = input_format
        .parse(&input)
        .context("Failed to parse input data")?;

    let output_format = match to {
        OutputFormat::Auto => {
            return Err(anyhow::anyhow!("Cannot convert to 'auto' format"))
        }
        OutputFormat::Json | OutputFormat::Compact => "json",
        OutputFormat::JsonPretty => "json",
        OutputFormat::Yaml => "yaml",
    };

    let formatter = get_output_format(output_format)?;

    let output = if matches!(to, OutputFormat::JsonPretty) || pretty {
        serde_json::to_string_pretty(&parsed_data)
            .context("Failed to format as pretty JSON")?
    } else if matches!(to, OutputFormat::Compact) {
        serde_json::to_string(&parsed_data)
            .context("Failed to format as compact JSON")?
    } else {
        formatter
            .to_string(&parsed_data)
            .context("Failed to format output")?
    };

    print!("{output}");
    Ok(())
}

fn run_examples() -> Result<()> {
    println!("{}", "XQPath Usage Examples".bold().underline());
    println!();

    println!("{}", "Basic Operations:".bold());
    println!("  {} Extract field from JSON/YAML:", "•".blue());
    println!("    {}", "xqpath get '.user.name' -f data.json".dimmed());
    println!(
        "    {}",
        "cat config.yaml | xqpath get '.spec.containers[0].image'".dimmed()
    );
    println!();

    println!("  {} Check if path exists:", "•".blue());
    println!(
        "    {}",
        "xqpath exists '.user.email' -f data.json".dimmed()
    );
    println!();

    println!("  {} Get value type:", "•".blue());
    println!("    {}", "xqpath type '.users' -f data.json".dimmed());
    println!();

    println!("  {} Count array elements:", "•".blue());
    println!("    {}", "xqpath count '.users[*]' -f data.json".dimmed());
    println!();

    #[cfg(feature = "update")]
    {
        println!("{}", "Update Operations:".bold());
        println!("  {} Update a field:", "•".blue());
        println!(
            "    {}",
            "xqpath set '.version' '\"2.0\"' -f config.yaml".dimmed()
        );
        println!();
    }

    println!("{}", "Advanced Features:".bold());
    println!("  {} Get object keys:", "•".blue());
    println!("    {}", "xqpath keys '.user' -f data.json".dimmed());
    println!();

    println!("  {} Get length:", "•".blue());
    println!("    {}", "xqpath length '.users' -f data.json".dimmed());
    println!();

    println!("  {} Validate format:", "•".blue());
    println!("    {}", "xqpath validate -f data.json".dimmed());
    println!();

    println!("  {} Convert formats:", "•".blue());
    println!("    {}", "xqpath convert json -f config.yaml".dimmed());
    println!(
        "    {}",
        "xqpath convert yaml -f data.json --pretty".dimmed()
    );
    println!();

    // v1.4.2 性能分析功能示例
    #[cfg(any(feature = "profiling", feature = "benchmark"))]
    {
        println!("{}", "Performance Analysis (v1.4.2):".bold());

        #[cfg(feature = "profiling")]
        {
            println!("  {} Profile query performance:", "•".magenta());
            println!(
                "    {}",
                "xqpath profile '.users[*].name' -f data.json".dimmed()
            );
            println!(
                "    {}",
                "xqpath profile '.data' --memory --hints".dimmed()
            );
            println!(
                "    {}",
                "xqpath profile '.complex' --html -o report.html".dimmed()
            );
            println!();

            println!("  {} Monitor real-time performance:", "•".magenta());
            println!(
                "    {}",
                "xqpath monitor '.users[*]' -f data.json -d 30".dimmed()
            );
            println!(
                "    {}",
                "xqpath monitor '.data' --interval 500 --continuous".dimmed()
            );
            println!();
        }

        #[cfg(feature = "benchmark")]
        {
            println!("  {} Benchmark query performance:", "•".magenta());
            println!(
                "    {}",
                "xqpath benchmark '.users[*].name' -f data.json".dimmed()
            );
            println!(
                "    {}",
                "xqpath benchmark '.data' -i 1000 --format html -o bench.html"
                    .dimmed()
            );
            println!(
                "    {}",
                "xqpath benchmark '.query' --baseline prev_results.json"
                    .dimmed()
            );
            println!();
        }
    }

    println!("{}", "Path Syntax:".bold());
    println!("  {} Object field access:", "•".green());
    println!("    {}", ".field, .nested.field".dimmed());
    println!();

    println!("  {} Array element access:", "•".green());
    println!("    {}", ".array[0], .users[1].name".dimmed());
    println!();

    println!("  {} Wildcard matching:", "•".green());
    println!("    {}", ".users[*].name    # All user names".dimmed());
    println!("    {}", ".**               # Recursive search".dimmed());
    println!();

    println!("  {} Type filtering:", "•".green());
    println!("    {}", ".data | string    # Only string values".dimmed());
    println!("    {}", ".items | array    # Only array values".dimmed());
    println!();

    println!("{}", "Output Options:".bold());
    println!("  {} Format control:", "•".yellow());
    println!("    {}", "--output json     # Force JSON output".dimmed());
    println!("    {}", "--output yaml     # Force YAML output".dimmed());
    println!("    {}", "--pretty          # Pretty-print JSON".dimmed());
    println!("    {}", "--no-color        # Disable colors".dimmed());
    println!();

    Ok(())
}

fn get_output_format(format_name: &str) -> Result<Box<dyn ValueFormat>> {
    match format_name.to_lowercase().as_str() {
        "json" => Ok(Box::new(JsonFormat)),
        "yaml" | "yml" => Ok(Box::new(YamlFormat)),
        _ => Err(anyhow::anyhow!(
            "Unsupported output format: {}",
            format_name
        )),
    }
}

// v1.4.2 性能分析命令实现

#[cfg(feature = "profiling")]
fn run_profile(
    path: &str,
    file: Option<&PathBuf>,
    html: bool,
    output: Option<&PathBuf>,
    memory: bool,
    hints: bool,
) -> Result<()> {
    use xqpath::{profile_complete, query_memory};

    let input = read_input(file)?;

    println!("{}", "🔍 Performance Profiling".bold().blue());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if memory {
        let (_result, memory_report) = query_memory!(input, path)
            .map_err(|e| anyhow::anyhow!("Memory query failed: {}", e))?;
        println!("✅ Query executed successfully");
        println!("📊 Memory Analysis:");
        println!(
            "   Peak Memory: {:.2} MB",
            memory_report.peak_memory_bytes as f64 / 1024.0 / 1024.0
        );
        println!(
            "   Current Memory: {:.2} MB",
            memory_report.current_memory_bytes as f64 / 1024.0 / 1024.0
        );

        if let Some(efficiency) = memory_report.metrics.get("memory_efficiency")
        {
            println!("   Memory Efficiency: {efficiency:.1}%");
        }
    } else {
        let (_result, profile) = profile_complete!(input, path)
            .map_err(|e| anyhow::anyhow!("Profile query failed: {}", e))?;
        println!("✅ Query executed successfully");
        println!("📊 Performance Metrics:");
        println!("   Execution Time: {:?}", profile.execution_time);
        println!(
            "   Peak Memory: {:.2} MB",
            profile.peak_memory_bytes as f64 / 1024.0 / 1024.0
        );
        println!("   CPU Usage: {:.1}%", profile.cpu_usage_percent);

        if hints && !profile.optimization_hints.is_empty() {
            println!("\n💡 Optimization Hints:");
            for hint in &profile.optimization_hints {
                println!("   • {hint}");
            }
        }

        if html {
            let output_path = output
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("profile_report.html"));

            std::fs::write(&output_path, profile.to_html())
                .context("Failed to write HTML report")?;

            println!("\n📄 HTML report saved to: {}", output_path.display());
        }
    }

    Ok(())
}

#[cfg(feature = "benchmark")]
fn run_benchmark(
    path: &str,
    file: Option<&PathBuf>,
    iterations: usize,
    warmup: usize,
    format: &BenchmarkOutputFormat,
    output: Option<&PathBuf>,
    baseline: Option<&PathBuf>,
) -> Result<()> {
    use std::time::Duration;
    use xqpath::{
        benchmark_query, BenchmarkConfig,
        BenchmarkOutputFormat as LibBenchmarkFormat, BenchmarkSuite,
    };

    let input = read_input(file)?;

    println!("{}", "⚡ Performance Benchmark".bold().yellow());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 快速基准测试
    let (_result, benchmark_result) = benchmark_query!(input, path, iterations)
        .map_err(|e| anyhow::anyhow!("Benchmark query failed: {}", e))?;
    println!("✅ Query executed successfully");
    println!("📊 Quick Benchmark Results:");
    println!("   {}", benchmark_result.summary());

    // 详细基准测试套件
    let config = BenchmarkConfig {
        warmup_iterations: warmup,
        test_iterations: iterations,
        min_test_time: Duration::from_millis(10),
        max_test_time: Duration::from_secs(30),
    };

    let mut suite = BenchmarkSuite::with_config(config);
    let input_clone = input.clone();
    let path_clone = path.to_string();

    suite.add_test("query_benchmark", move || {
        let _result = xqpath::query!(input_clone, &path_clone)?;
        Ok(())
    });

    let results = suite
        .run()
        .map_err(|e| anyhow::anyhow!("Suite run failed: {}", e))?;

    println!("\n📊 Detailed Benchmark Results:");
    for result in &results {
        println!("   {}", result.summary());
    }

    // 保存结果
    if let Some(output_path) = output {
        let lib_format = match format {
            BenchmarkOutputFormat::Text => LibBenchmarkFormat::Json, // 使用JSON作为Text的替代
            BenchmarkOutputFormat::Json => LibBenchmarkFormat::Json,
            BenchmarkOutputFormat::Html => LibBenchmarkFormat::Html,
            BenchmarkOutputFormat::Csv => LibBenchmarkFormat::Csv,
        };

        BenchmarkSuite::save_results_to_file(
            &results,
            output_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid output path"))?,
            lib_format,
        )
        .map_err(|e| anyhow::anyhow!("Failed to save results: {}", e))?;
        println!("\n📄 Benchmark results saved to: {}", output_path.display());
    }

    // 比较基准线
    if let Some(baseline_path) = baseline {
        println!("\n📈 Baseline comparison not yet implemented");
        println!("   Baseline file: {}", baseline_path.display());
    }

    Ok(())
}

#[cfg(feature = "profiling")]
fn run_monitor(
    path: &str,
    file: Option<&PathBuf>,
    duration: u64,
    interval: u64,
    continuous: bool,
) -> Result<()> {
    use std::thread;
    use std::time::{Duration, Instant};
    use xqpath::{query, PerformanceMonitor};

    let input = read_input(file)?;

    println!("{}", "📊 Performance Monitor".bold().green());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Duration: {duration} seconds, Interval: {interval} ms");
    println!("Path: {path}");
    println!();

    let mut monitor = PerformanceMonitor::new();
    monitor.start();

    let start_time = Instant::now();
    let total_duration = Duration::from_secs(duration);
    let update_interval = Duration::from_millis(interval);

    let mut iteration = 0;

    while start_time.elapsed() < total_duration {
        // 执行查询
        let query_start = Instant::now();
        let _result = query!(input, path)
            .map_err(|e| anyhow::anyhow!("Monitor query failed: {}", e))?;
        let query_time = query_start.elapsed();

        // 获取当前指标
        let metrics = monitor.get_current_metrics();

        iteration += 1;
        println!("Iteration {iteration}: Query time: {query_time:?}");

        if continuous {
            for (name, value) in metrics {
                println!("  {name}: {value:.2}");
            }
            println!();
        }

        thread::sleep(update_interval);
    }

    let final_report = monitor.stop();

    println!("🏁 Final Performance Report:");
    println!("   Total iterations: {iteration}");
    println!("   {}", final_report.summary());

    // 保存最终报告
    let report_path = PathBuf::from("monitor_report.html");
    std::fs::write(&report_path, final_report.to_html())
        .context("Failed to write monitor report")?;

    println!("\n📄 Monitor report saved to: {}", report_path.display());

    Ok(())
}

// v1.4.1 调试命令实现

#[cfg(feature = "debug")]
fn run_debug(
    path: &str,
    file: Option<&PathBuf>,
    interactive: bool,
) -> Result<()> {
    use xqpath::query_debug;

    let input = read_input(file)?;

    println!("{}", "🔍 Debug Mode Execution".bold().blue());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Path: {path}");
    if let Some(file_path) = file {
        println!("Input: {}", file_path.display());
    } else {
        println!("Input: stdin");
    }
    println!();

    if interactive {
        println!("🎯 Interactive Debug Mode");
        println!("Type 'help' for commands, 'quit' to exit");
        // TODO: 实现交互式调试模式
        println!("⚠️  Interactive mode will be implemented in future version");
        println!();
    }

    // 执行调试查询
    println!("🚀 Executing debug query...");
    let result =
        query_debug!(input, path, |debug_info: &xqpath::debug::DebugInfo| {
            println!("🔍 Debug Info:");
            if let Some(parse_time) = debug_info.parse_duration {
                println!("   Parse time: {parse_time:?}");
            }
            if let Some(exec_time) = debug_info.execution_duration {
                println!("   Execution time: {exec_time:?}");
            }
            if !debug_info.execution_path.is_empty() {
                println!("   Execution path: {}", debug_info.execution_path);
            }
            if let Some(memory) = debug_info.memory_used {
                println!("   Memory used: {memory} bytes");
            }
            println!("   Queries executed: {}", debug_info.queries_executed);
        });

    match result {
        Ok(values) => {
            println!("✅ Query executed successfully");
            println!("📊 Results: {} value(s) found", values.len());

            for (i, value) in values.iter().enumerate() {
                println!(
                    "Result {}: {}",
                    i + 1,
                    serde_json::to_string_pretty(value)?
                );
            }
        }
        Err(e) => {
            println!("❌ Query failed with error:");
            println!("   {e}");

            // 分析错误并提供建议
            provide_error_suggestions(path, &e.to_string());
        }
    }

    Ok(())
}

#[cfg(feature = "debug")]
fn run_trace(path: &str, file: Option<&PathBuf>, detailed: bool) -> Result<()> {
    use xqpath::trace_query;

    let input = read_input(file)?;

    println!("{}", "📊 Path Execution Trace".bold().green());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Path: {path}");
    if detailed {
        println!("Mode: Detailed trace");
    } else {
        println!("Mode: Standard trace");
    }
    println!();

    // 执行跟踪查询
    println!("🚀 Starting path execution trace...");
    let result = trace_query!(input, path);

    match result {
        Ok((values, stats)) => {
            println!("✅ Trace completed successfully");
            println!("📊 Execution Time: {:?}", stats.duration);
            println!("📊 Final Results: {} value(s)", values.len());

            if detailed {
                println!("\n📋 Detailed Results:");
                for (i, value) in values.iter().enumerate() {
                    println!("  [{}] Type: {}", i + 1, get_value_type(value));
                    println!("      Value: {}", format_value_preview(value));
                }
            } else {
                println!("\n📋 Results Summary:");
                for (i, value) in values.iter().enumerate() {
                    println!(
                        "  [{}] {}: {}",
                        i + 1,
                        get_value_type(value),
                        format_value_preview(value)
                    );
                }
            }
        }
        Err(e) => {
            println!("❌ Trace failed with error:");
            println!("   {e}");

            // 分析错误并提供建议
            provide_error_suggestions(path, &e.to_string());
        }
    }

    Ok(())
}

#[cfg(feature = "debug")]
fn provide_error_suggestions(path: &str, error: &str) {
    println!("\n💡 Error Analysis & Suggestions:");

    if error.contains("parse") || error.contains("syntax") {
        println!("   🔍 Parse Error Detected:");
        println!("   • Check path syntax: {path}");
        println!("   • Common issues:");
        println!(
            "     - Missing quotes around field names with special characters"
        );
        println!("     - Incorrect array index syntax");
        println!("     - Unmatched brackets or parentheses");
    } else if error.contains("field") || error.contains("key") {
        println!("   🔍 Field Access Error:");
        println!("   • Field might not exist in the data");
        println!("   • Try using optional operator: .field?");
        println!("   • Check if data structure matches expectation");
    } else if error.contains("index") || error.contains("array") {
        println!("   🔍 Array Access Error:");
        println!("   • Array index might be out of bounds");
        println!("   • Use wildcard for all elements: [*]");
        println!("   • Check if the value is actually an array");
    } else if error.contains("type") {
        println!("   🔍 Type Error:");
        println!("   • Operation not supported for this data type");
        println!("   • Use type filters: | string, | array, | object");
        println!("   • Check data type before operation");
    } else {
        println!("   🔍 General Error:");
        println!("   • Try simplifying the path expression");
        println!("   • Test with shorter path segments");
        println!("   • Use --verbose for more details");
    }

    println!("\n📖 For more help, run: xqpath examples");
}

#[cfg(feature = "debug")]
fn get_value_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(feature = "debug")]
fn format_value_preview(value: &Value) -> String {
    match value {
        Value::String(s) if s.len() > 50 => format!("\"{}...\"", &s[..47]),
        Value::Array(arr) => format!("[{} elements]", arr.len()),
        Value::Object(obj) => format!("{{{}keys}}", obj.len()),
        _ => value.to_string(),
    }
}

// v1.4.3 配置管理命令实现
#[cfg(feature = "config-management")]
fn run_config(action: &ConfigAction) -> Result<()> {
    use xqpath::config::ConfigManager;

    let mut manager = match ConfigManager::new() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("❌ 配置管理器初始化失败: {e}");
            return Ok(());
        }
    };

    match action {
        ConfigAction::Show => {
            let config = manager.get_config();
            println!("📋 当前配置:");
            println!("活动配置文件: {}", manager.get_active_profile());
            println!();

            // 显示配置内容（这里使用简化的显示）
            println!("🔧 调试配置:");
            println!("  level: {}", config.debug.level);
            println!("  output: {}", config.debug.output);
            println!("  timing: {}", config.debug.timing);

            println!("\n⚡ 性能配置:");
            println!("  memory_limit: {}", config.performance.memory_limit);
            println!("  timeout: {}", config.performance.timeout);
            println!("  cache_size: {}", config.performance.cache_size);
            println!("  parallel_jobs: {}", config.performance.parallel_jobs);

            println!("\n🎯 功能配置:");
            println!("  colored_output: {}", config.features.colored_output);
            println!(
                "  interactive_mode: {}",
                config.features.interactive_mode
            );
            println!("  auto_backup: {}", config.features.auto_backup);
        }

        ConfigAction::Set { key, value } => {
            match manager.set_config_value(key, value) {
                Ok(()) => {
                    if let Ok(()) = manager.save_config() {
                        println!("✅ 配置已更新: {key} = {value}");
                    } else {
                        eprintln!("❌ 配置保存失败");
                    }
                }
                Err(e) => {
                    eprintln!("❌ 配置设置失败: {e}");
                }
            }
        }

        ConfigAction::Reset => match manager.reset_config() {
            Ok(()) => {
                println!("🔄 配置已重置为默认值");
            }
            Err(e) => {
                eprintln!("❌ 配置重置失败: {e}");
            }
        },

        ConfigAction::Template { name } => {
            match manager.create_template(name) {
                Ok(()) => {
                    println!("📄 配置模板已创建: {name}");
                }
                Err(e) => {
                    eprintln!("❌ 模板创建失败: {e}");
                }
            }
        }

        ConfigAction::Profile { action } => match action {
            ProfileAction::Create { name } => {
                match manager.create_profile(name) {
                    Ok(()) => {
                        println!("📁 配置文件已创建: {name}");
                    }
                    Err(e) => {
                        eprintln!("❌ 配置文件创建失败: {e}");
                    }
                }
            }
            ProfileAction::Switch { name } => {
                match manager.switch_profile(name) {
                    Ok(()) => {
                        println!("🔄 已切换到配置文件: {name}");
                    }
                    Err(e) => {
                        eprintln!("❌ 配置文件切换失败: {e}");
                    }
                }
            }
            ProfileAction::List => {
                let profiles = manager.list_profiles();
                let active = manager.get_active_profile();

                println!("📁 可用的配置文件:");
                for profile in profiles {
                    if profile == active {
                        println!("  • {} (当前)", profile.green().bold());
                    } else {
                        println!("  • {profile}");
                    }
                }
            }
        },

        ConfigAction::Audit => {
            println!("📊 配置审计功能开发中...");
        }

        ConfigAction::Migrate => {
            println!("🔄 配置迁移功能开发中...");
        }
    }

    Ok(())
}

// v1.4.3 交互式调试器命令实现
#[cfg(feature = "interactive-debug")]
fn run_interactive_debugger(file: Option<&PathBuf>) -> Result<()> {
    use xqpath::debugger::XQPathDebugger;

    println!("🚀 启动 XQPath 交互式调试器...");

    let mut debugger = XQPathDebugger::new();

    // 如果指定了文件，预加载它
    if let Some(file_path) = file {
        println!("📁 预加载文件: {}", file_path.display());
        // 这里需要实现文件预加载逻辑
    }

    match debugger.run() {
        Ok(()) => {
            println!("👋 调试器会话结束");
        }
        Err(e) => {
            eprintln!("❌ 调试器错误: {e}");
        }
    }

    Ok(())
}
