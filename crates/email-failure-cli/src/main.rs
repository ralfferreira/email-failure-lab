mod commands;
mod error;
mod output;

use clap::{Parser, Subcommand};

use crate::commands::explain::{run_explain, ExplainArgs};
use crate::error::CliError;

#[derive(Debug, Parser)]
#[command(
    name = "email-lab",
    version,
    about = "Explain transactional email failures from SMTP, bounce-like text, and provider webhook JSON."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Explain SMTP text, bounce-like input, or provider webhook JSON.
    Explain(ExplainArgs),
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
        Command::Explain(args) => run_explain(args),
    }
}
