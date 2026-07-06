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
    about = "Explain transactional email failures from SMTP and bounce-like text."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Explain an SMTP error, bounce-like string, or plain text file.
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
