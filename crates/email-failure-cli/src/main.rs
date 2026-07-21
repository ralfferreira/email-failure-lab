mod commands;
mod error;
mod output;

use clap::{Parser, Subcommand};

use crate::commands::explain::{run_explain, ExplainArgs};
use crate::commands::fixtures::{run_fixtures, FixturesArgs};
use crate::error::CliError;

#[derive(Debug, Parser)]
#[command(
    name = "email-lab",
    version,
    about = "Explain transactional email failures from SMTP, bounce-like text, and provider webhook JSON."
)]
struct Cli {
    /// Disable color in text output.
    #[arg(long, global = true)]
    no_color: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Explain SMTP text, bounce-like input, or provider webhook JSON.
    Explain(ExplainArgs),
    /// Discover and inspect built-in failure fixtures.
    Fixtures(FixturesArgs),
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), CliError> {
    let cli = Cli::parse();

    match cli.command {
        Command::Explain(args) => run_explain(args, cli.no_color),
        Command::Fixtures(args) => run_fixtures(args),
    }
}
