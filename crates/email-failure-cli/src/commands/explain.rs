use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use clap::Args;
use email_failure_core::{explain, InputSource, ParseInput};

use crate::error::CliError;
use crate::output::{format_json, format_text, OutputFormat};

#[derive(Debug, Args)]
pub struct ExplainArgs {
    /// SMTP error, bounce-like string, or path to a plain text file.
    pub input: String,
    /// Emit stable JSON output.
    #[arg(long)]
    pub json: bool,
    /// Select output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,
    /// Include additional signal details in text output.
    #[arg(long)]
    pub verbose: bool,
}

pub fn run_explain(args: ExplainArgs) -> Result<(), CliError> {
    let format = if args.json {
        OutputFormat::Json
    } else {
        args.format
    };

    let resolved_input = resolve_input(&args.input)?;
    let report = explain(ParseInput {
        raw: &resolved_input.raw,
        source: resolved_input.source,
    });

    let rendered = match format {
        OutputFormat::Text => format_text(&report, args.verbose),
        OutputFormat::Json => format_json(&report)?,
    };

    println!("{rendered}");
    Ok(())
}

#[derive(Debug)]
struct ResolvedInput {
    raw: String,
    source: InputSource,
}

fn resolve_input(input: &str) -> Result<ResolvedInput, CliError> {
    if input == "-" {
        return read_stdin();
    }

    if input.trim().is_empty() {
        return Err(CliError::EmptyInput);
    }

    let path = PathBuf::from(input);

    if path.is_file() {
        let raw = fs::read_to_string(&path).map_err(|source| CliError::ReadFile {
            path: path.clone(),
            source,
        })?;
        if raw.trim().is_empty() {
            return Err(CliError::EmptyInput);
        }

        return Ok(ResolvedInput {
            raw,
            source: InputSource::File {
                path: path.display().to_string(),
            },
        });
    }

    if is_path_like(input) {
        return Err(CliError::MissingFile(path));
    }

    Ok(ResolvedInput {
        raw: input.to_owned(),
        source: InputSource::Inline,
    })
}

fn read_stdin() -> Result<ResolvedInput, CliError> {
    let mut raw = String::new();
    io::stdin()
        .read_to_string(&mut raw)
        .map_err(CliError::ReadStdin)?;

    if raw.trim().is_empty() {
        return Err(CliError::EmptyInput);
    }

    Ok(ResolvedInput {
        raw,
        source: InputSource::Inline,
    })
}

fn is_path_like(input: &str) -> bool {
    input.starts_with("./")
        || input.starts_with("../")
        || input.starts_with('/')
        || input.starts_with("~/")
        || input.contains('/')
        || input.contains('\\')
        || has_plain_text_file_extension(input)
}

fn has_plain_text_file_extension(input: &str) -> bool {
    Path::new(input)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "txt" | "log" | "eml"
            )
        })
}

#[cfg(test)]
mod tests {
    use super::is_path_like;

    #[test]
    fn dots_alone_do_not_make_input_path_like() {
        assert!(!is_path_like("550 5.1.1 User unknown"));
        assert!(!is_path_like("5.1.1"));
    }

    #[test]
    fn detects_only_explicit_path_like_inputs() {
        assert!(is_path_like("./bounce.txt"));
        assert!(is_path_like("../bounce.txt"));
        assert!(is_path_like("/tmp/bounce.txt"));
        assert!(is_path_like("~/bounce.txt"));
        assert!(is_path_like("fixtures\\bounce.txt"));
        assert!(is_path_like("bounce.eml"));
        assert!(is_path_like("bounce.log"));
        assert!(!is_path_like("-"));
        assert!(!is_path_like("mailbox.full"));
    }
}
