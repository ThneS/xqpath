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
#[command(about = "A minimal jq-like path extractor and updater for structured data in Rust")]
#[command(long_about = None)]
struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,
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
    
    // 设置颜色输出 (针对每个命令的no_color参数)
    let no_color = match &cli.command {
        Commands::Get { no_color, .. } | 
        Commands::Exists { no_color, .. } |
        Commands::Type { no_color, .. } |
        Commands::Count { no_color, .. } |
        Commands::Length { no_color, .. } |
        Commands::Keys { no_color, .. } |
        Commands::Validate { no_color, .. } |
        Commands::Convert { no_color, .. } => *no_color,
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
            Commands::Get { verbose, .. } | 
            Commands::Exists { verbose, .. } |
            Commands::Type { verbose, .. } |
            Commands::Count { verbose, .. } |
            Commands::Length { verbose, .. } |
            Commands::Keys { verbose, .. } |
            Commands::Validate { verbose, .. } |
            Commands::Convert { verbose, .. } => *verbose,
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

fn run_command(cli: &Cli) -> Result<()> {
    match &cli.command {
        Commands::Get { path, file, output, pretty, verbose, .. } => {
            run_get(path, file.as_ref(), output, *pretty, *verbose)
        },
        #[cfg(feature = "update")]
        Commands::Set { path, value, file, output, pretty, verbose, .. } => {
            run_set(path, value, file.as_ref(), output, *pretty, *verbose)
        },
        Commands::Exists { path, file, verbose, .. } => {
            run_exists(path, file.as_ref(), *verbose)
        },
        Commands::Type { path, file, verbose, .. } => {
            run_type(path, file.as_ref(), *verbose)
        },
        Commands::Count { path, file, verbose, .. } => {
            run_count(path, file.as_ref(), *verbose)
        },
        Commands::Length { path, file, verbose, .. } => {
            run_length(path, file.as_ref(), *verbose)
        },
        Commands::Keys { path, file, output, pretty, verbose, .. } => {
            run_keys(path, file.as_ref(), output, *pretty, *verbose)
        },
        Commands::Interactive { file } => {
            run_interactive(file.as_ref())
        },
        Commands::Validate { file, verbose, .. } => {
            run_validate(file.as_ref(), *verbose)
        },
        Commands::Convert { to, file, pretty, verbose, .. } => {
            run_convert(to, file.as_ref(), *pretty, *verbose)
        },
        Commands::Examples => run_examples(),
    }
}

fn read_input(file: Option<&PathBuf>) -> Result<String> {
    match file {
        Some(path) => {
            fs::read_to_string(path)
                .with_context(|| format!("Failed to read file: {}", path.display()))
        }
        None => {
            let mut input = String::new();
            io::stdin()
                .read_to_string(&mut input)
                .context("Failed to read from stdin")?;
            Ok(input)
        }
    }
}

fn parse_and_extract(input: &str, path: &str) -> Result<(Box<dyn ValueFormat>, Vec<Value>)> {
    let format = detect_format(input)
        .context("Failed to detect input format")?;
    
    let parsed_data = format
        .parse(input)
        .context("Failed to parse input data")?;
    
    let path_obj = parse_path(path)
        .context("Failed to parse path expression")?;
    
    let values = extract(&parsed_data, &path_obj)
        .context("Failed to extract values")?;
    
    let owned_values: Vec<Value> = values.into_iter().map(|v| v.clone()).collect();
    
    Ok((format, owned_values))
}

fn output_values(values: &[Value], format: &dyn ValueFormat, output: &OutputFormat, pretty: bool) -> Result<()> {
    let output_format = match output {
        OutputFormat::Auto => format.name(),
        _ => match output {
            OutputFormat::Json | OutputFormat::Compact => "json",
            OutputFormat::JsonPretty => "json",
            OutputFormat::Yaml => "yaml",
            OutputFormat::Auto => unreachable!(),
        }
    };
    
    let formatter = get_output_format(output_format)?;
    
    for (i, value) in values.iter().enumerate() {
        if i > 0 {
            println!();
        }
        
        let output_str = if pretty && matches!(output, OutputFormat::JsonPretty | OutputFormat::Auto) {
            serde_json::to_string_pretty(value)
                .context("Failed to format output")?
        } else if matches!(output, OutputFormat::Compact) {
            serde_json::to_string(value)
                .context("Failed to format output")?
        } else {
            formatter.to_string(value)
                .context("Failed to format output")?
        };
        
        print!("{}", output_str);
    }
    
    if !values.is_empty() {
        println!();
    }
    
    Ok(())
}

fn run_get(path: &str, file: Option<&PathBuf>, output: &OutputFormat, pretty: bool, verbose: bool) -> Result<()> {
    let input = read_input(file)?;
    let (format, values) = parse_and_extract(&input, path)?;
    
    if verbose {
        eprintln!("{} Found {} value(s)", "Info:".blue().bold(), values.len());
    }
    
    output_values(&values, format.as_ref(), output, pretty)?;
    Ok(())
}

#[cfg(feature = "update")]
fn run_set(path: &str, new_value_str: &str, file: Option<&PathBuf>, output: &OutputFormat, pretty: bool, verbose: bool) -> Result<()> {
    let input = read_input(file)?;
    
    let format = detect_format(&input)
        .context("Failed to detect input format")?;
    
    let mut parsed_data = format
        .parse(&input)
        .context("Failed to parse input data")?;
    
    let path_obj = parse_path(path)
        .context("Failed to parse path expression")?;
    
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
        }
    };
    
    let formatter = get_output_format(output_format)?;
    let output_str = formatter.to_string(&parsed_data)
        .context("Failed to format output")?;
    
    print!("{}", output_str);
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
        println!("{}", exists);
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
            println!("{}", type_name);
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
                    println!("{}", len);
                }
            }
            None => {
                if verbose {
                    println!("{} Value has no length property", "Info:".yellow().bold());
                } else {
                    println!("null");
                }
            }
        }
    }
    
    Ok(())
}

fn run_keys(path: &str, file: Option<&PathBuf>, output: &OutputFormat, pretty: bool, verbose: bool) -> Result<()> {
    let input = read_input(file)?;
    let (format, values) = parse_and_extract(&input, path)?;
    
    for value in values {
        match &value {
            Value::Object(obj) => {
                let keys: Vec<Value> = obj.keys()
                    .map(|k| Value::String(k.clone()))
                    .collect();
                let keys_array = Value::Array(keys);
                output_values(&[keys_array], format.as_ref(), output, pretty)?;
            }
            Value::Array(arr) => {
                let indices: Vec<Value> = (0..arr.len())
                    .map(|i| Value::Number(serde_json::Number::from(i)))
                    .collect();
                let indices_array = Value::Array(indices);
                output_values(&[indices_array], format.as_ref(), output, pretty)?;
            }
            _ => {
                if verbose {
                    println!("{} Value is not an object or array", "Info:".yellow().bold());
                } else {
                    println!("null");
                }
            }
        }
    }
    
    Ok(())
}

fn run_interactive(_file: Option<&PathBuf>) -> Result<()> {
    println!("{}", "🚀 Interactive mode is not yet implemented".yellow().bold());
    println!("This feature will be available in a future release.");
    println!("For now, you can use the individual commands like 'get', 'exists', etc.");
    Ok(())
}

fn run_validate(file: Option<&PathBuf>, verbose: bool) -> Result<()> {
    let input = read_input(file)?;
    
    match detect_format(&input) {
        Ok(format) => {
            match format.parse(&input) {
                Ok(_) => {
                    if verbose {
                        println!("{} Valid {} format", "✓".green().bold(), format.name());
                    } else {
                        println!("valid");
                    }
                }
                Err(e) => {
                    if verbose {
                        println!("{} Invalid {}: {}", "✗".red().bold(), format.name(), e);
                    } else {
                        println!("invalid");
                    }
                    return Err(anyhow::anyhow!("Validation failed: {}", e));
                }
            }
        }
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

fn run_convert(to: &OutputFormat, file: Option<&PathBuf>, pretty: bool, _verbose: bool) -> Result<()> {
    let input = read_input(file)?;
    
    let input_format = detect_format(&input)
        .context("Failed to detect input format")?;
    
    let parsed_data = input_format
        .parse(&input)
        .context("Failed to parse input data")?;
    
    let output_format = match to {
        OutputFormat::Auto => return Err(anyhow::anyhow!("Cannot convert to 'auto' format")),
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
        formatter.to_string(&parsed_data)
            .context("Failed to format output")?
    };
    
    print!("{}", output);
    Ok(())
}

fn run_examples() -> Result<()> {
    println!("{}", "XQPath Usage Examples".bold().underline());
    println!();
    
    println!("{}", "Basic Operations:".bold());
    println!("  {} Extract field from JSON/YAML:", "•".blue());
    println!("    {}", "xqpath get '.user.name' -f data.json".dimmed());
    println!("    {}", "cat config.yaml | xqpath get '.spec.containers[0].image'".dimmed());
    println!();
    
    println!("  {} Check if path exists:", "•".blue());
    println!("    {}", "xqpath exists '.user.email' -f data.json".dimmed());
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
        println!("    {}", "xqpath set '.version' '\"2.0\"' -f config.yaml".dimmed());
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
    println!("    {}", "xqpath convert yaml -f data.json --pretty".dimmed());
    println!();
    
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
        _ => Err(anyhow::anyhow!("Unsupported output format: {}", format_name)),
    }
}


