use clap::{Parser, Subcommand, ValueEnum};
use onioncry::{
    CLI_VERSION, CheckOptions, FailOn, OnionCryError, init_config, render_config_schema_json,
    render_explain_pretty, render_llm, render_pretty, render_rules_pretty, render_sarif,
    rule_catalog, run_check_with_options, run_explain, write_config_schema,
};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(name = "onioncry")]
#[command(version = CLI_VERSION)]
#[command(about = "Check architectural boundaries in source projects")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Check(CheckArgs),
    Init(InitArgs),
    Explain(ExplainArgs),
    #[command(about = "List built-in rules")]
    Rules(RulesArgs),
    #[command(about = "Print the .onioncryrc JSON Schema")]
    Schema(SchemaArgs),
}

#[derive(Debug, Parser)]
struct CheckArgs {
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long, default_value_t = CheckOutputFormat::Pretty)]
    format: CheckOutputFormat,
    #[arg(long, default_value_t = FailOnArg::Error)]
    fail_on: FailOnArg,
    #[arg(long, alias = "tip", help = "Show remediation tips for diagnostics")]
    tips: bool,
    #[arg(
        long,
        value_name = "PATH",
        help = "Path to the violation baseline file"
    )]
    baseline: Option<PathBuf>,
    #[arg(
        long = "write-baseline",
        help = "Write current violations to the baseline file"
    )]
    write_baseline: bool,
    #[arg(
        long = "no-baseline",
        help = "Disable violation baseline consumption for this run"
    )]
    no_baseline: bool,
    #[arg(
        long = "llm-mode",
        conflicts_with_all = ["format", "tips"],
        help = "Show an LLM-optimized grouped diagnostic report"
    )]
    llm_mode: bool,
}

#[derive(Debug, Parser)]
struct InitArgs {
    #[arg(long)]
    force: bool,
}

#[derive(Debug, Parser)]
struct ExplainArgs {
    file: PathBuf,
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long, default_value_t = OutputFormat::Pretty)]
    format: OutputFormat,
    #[arg(long, alias = "tip", help = "Show remediation tips for diagnostics")]
    tips: bool,
}

#[derive(Debug, Parser)]
struct RulesArgs {
    #[arg(long, default_value_t = OutputFormat::Pretty)]
    format: OutputFormat,
}

#[derive(Debug, Parser)]
struct SchemaArgs {
    #[arg(long)]
    write: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CheckOutputFormat {
    Pretty,
    Json,
    Sarif,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Pretty,
    Json,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum FailOnArg {
    Error,
    Warning,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::Check(args) => run_check_command(args),
        Commands::Init(args) => run_init_command(args),
        Commands::Explain(args) => run_explain_command(args),
        Commands::Rules(args) => run_rules_command(args),
        Commands::Schema(args) => run_schema_command(args),
    }
}

fn run_schema_command(args: SchemaArgs) -> ExitCode {
    if let Some(path) = args.write {
        let cwd = match std::env::current_dir() {
            Ok(cwd) => cwd,
            Err(error) => {
                eprintln!("error: could not determine current directory: {error}");
                return ExitCode::from(2);
            }
        };

        match write_config_schema(&cwd, &path) {
            Ok(_) => {
                println!("created {}", path.display());
                ExitCode::SUCCESS
            }
            Err(error) => {
                print_error(&error);
                ExitCode::from(2)
            }
        }
    } else {
        match render_config_schema_json() {
            Ok(schema) => {
                println!("{schema}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                print_error(&error);
                ExitCode::from(2)
            }
        }
    }
}

fn run_rules_command(args: RulesArgs) -> ExitCode {
    let rules = rule_catalog();
    match args.format {
        OutputFormat::Pretty => {
            print!("{}", render_rules_pretty(&rules));
            ExitCode::SUCCESS
        }
        OutputFormat::Json => match serde_json::to_string_pretty(&rules) {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("error: could not render JSON output: {error}");
                ExitCode::from(2)
            }
        },
    }
}

fn run_explain_command(args: ExplainArgs) -> ExitCode {
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            eprintln!("error: could not determine current directory: {error}");
            return ExitCode::from(2);
        }
    };

    match run_explain(&cwd, args.config.as_deref(), &args.file) {
        Ok(report) => {
            match args.format {
                OutputFormat::Pretty => {
                    print!("{}", render_explain_pretty(&report, args.tips));
                }
                OutputFormat::Json => match serde_json::to_string_pretty(&report) {
                    Ok(json) => println!("{json}"),
                    Err(error) => {
                        eprintln!("error: could not render JSON output: {error}");
                        return ExitCode::from(2);
                    }
                },
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            print_error(&error);
            ExitCode::from(2)
        }
    }
}

fn run_init_command(args: InitArgs) -> ExitCode {
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            eprintln!("error: could not determine current directory: {error}");
            return ExitCode::from(2);
        }
    };

    match init_config(&cwd, args.force) {
        Ok(path) => {
            println!("created {}", path.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            print_error(&error);
            ExitCode::from(2)
        }
    }
}

fn run_check_command(args: CheckArgs) -> ExitCode {
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            eprintln!("error: could not determine current directory: {error}");
            return ExitCode::from(2);
        }
    };

    match run_check_with_options(
        &cwd,
        CheckOptions {
            explicit_config: args.config.as_deref(),
            fail_on: args.fail_on.into(),
            baseline_path: args.baseline.as_deref(),
            write_baseline: args.write_baseline,
            no_baseline: args.no_baseline,
        },
    ) {
        Ok(outcome) => {
            if let Some(baseline_warning) = &outcome.baseline_warning {
                eprintln!(
                    "warning: {} stale baseline {} in {}; rerun --write-baseline to remove fixed entries",
                    baseline_warning.stale_entry_count,
                    pluralize_entry(baseline_warning.stale_entry_count),
                    baseline_warning.path.display()
                );
            }

            if let Some(baseline_write) = &outcome.baseline_write {
                eprintln!(
                    "wrote baseline {} ({} {})",
                    baseline_write.path.display(),
                    baseline_write.entry_count,
                    pluralize_entry(baseline_write.entry_count)
                );
            }

            let report = outcome.report;
            if args.llm_mode {
                print!("{}", render_llm(&report));
            } else {
                match args.format {
                    CheckOutputFormat::Pretty => {
                        print!("{}", render_pretty(&report, args.tips));
                    }
                    CheckOutputFormat::Json => match serde_json::to_string_pretty(&report) {
                        Ok(json) => println!("{json}"),
                        Err(error) => {
                            eprintln!("error: could not render JSON output: {error}");
                            return ExitCode::from(2);
                        }
                    },
                    CheckOutputFormat::Sarif => {
                        let rules = rule_catalog();
                        match render_sarif(&report, &rules) {
                            Ok(sarif) => println!("{sarif}"),
                            Err(error) => {
                                eprintln!("error: could not render SARIF output: {error}");
                                return ExitCode::from(2);
                            }
                        }
                    }
                }
            };

            if report.should_exit_with_failure() {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(error) => {
            print_error(&error);
            ExitCode::from(2)
        }
    }
}

fn print_error(error: &OnionCryError) {
    eprintln!("error: {error}");
}

fn pluralize_entry(count: usize) -> &'static str {
    if count == 1 { "entry" } else { "entries" }
}

impl std::fmt::Display for CheckOutputFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckOutputFormat::Pretty => formatter.write_str("pretty"),
            CheckOutputFormat::Json => formatter.write_str("json"),
            CheckOutputFormat::Sarif => formatter.write_str("sarif"),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Pretty => formatter.write_str("pretty"),
            OutputFormat::Json => formatter.write_str("json"),
        }
    }
}

impl std::fmt::Display for FailOnArg {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FailOnArg::Error => formatter.write_str("error"),
            FailOnArg::Warning => formatter.write_str("warning"),
        }
    }
}

impl From<FailOnArg> for FailOn {
    fn from(value: FailOnArg) -> Self {
        match value {
            FailOnArg::Error => FailOn::Error,
            FailOnArg::Warning => FailOn::Warning,
        }
    }
}
