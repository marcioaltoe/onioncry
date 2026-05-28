use clap::{Parser, Subcommand, ValueEnum};
use onioncry::{
    FailOn, OnionCryError, init_config, render_explain_pretty, render_llm, render_pretty,
    run_check, run_explain,
};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(name = "onioncry")]
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
}

#[derive(Debug, Parser)]
struct CheckArgs {
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long, default_value_t = OutputFormat::Pretty)]
    format: OutputFormat,
    #[arg(long, default_value_t = FailOnArg::Error)]
    fail_on: FailOnArg,
    #[arg(long, alias = "tip", help = "Show remediation tips for diagnostics")]
    tips: bool,
    #[arg(
        long,
        conflicts_with_all = ["format", "tips"],
        help = "Show an LLM-optimized grouped diagnostic report"
    )]
    llm: bool,
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

    match run_check(&cwd, args.config.as_deref(), args.fail_on.into()) {
        Ok(report) => {
            if args.llm {
                print!("{}", render_llm(&report));
            } else {
                match args.format {
                    OutputFormat::Pretty => {
                        print!("{}", render_pretty(&report, args.tips));
                    }
                    OutputFormat::Json => match serde_json::to_string_pretty(&report) {
                        Ok(json) => println!("{json}"),
                        Err(error) => {
                            eprintln!("error: could not render JSON output: {error}");
                            return ExitCode::from(2);
                        }
                    },
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
