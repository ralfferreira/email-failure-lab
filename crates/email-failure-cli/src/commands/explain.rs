use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use anstream::{AutoStream, ColorChoice};
use clap::Args;
use email_failure_core::{explain, FailureReport, InputSource, ParseInput};

use crate::error::CliError;
use crate::output::{format_json, format_text, OutputFormat, TextStyle};

#[derive(Debug, Args)]
pub struct ExplainArgs {
    /// SMTP error, bounce-like string, provider webhook JSON, or path to an input file.
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

pub fn run_explain(args: ExplainArgs, no_color: bool) -> Result<(), CliError> {
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

    match format {
        OutputFormat::Text => write_text_report(&report, args.verbose, no_color),
        OutputFormat::Json => write_json_report(&report),
    }
}

fn write_text_report(
    report: &FailureReport,
    verbose: bool,
    no_color: bool,
) -> Result<(), CliError> {
    let stdout = io::stdout();
    let color_choice = if no_color {
        ColorChoice::Never
    } else {
        AutoStream::choice(&stdout)
    };
    let text_style = if color_choice == ColorChoice::Never {
        TextStyle::Plain
    } else {
        TextStyle::Color
    };
    let rendered = format_text(report, verbose, text_style);
    let mut stdout = AutoStream::new(stdout, color_choice).lock();

    writeln!(stdout, "{rendered}").map_err(CliError::WriteOutput)
}

fn write_json_report(report: &FailureReport) -> Result<(), CliError> {
    let rendered = format_json(report)?;
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    writeln!(stdout, "{rendered}").map_err(CliError::WriteOutput)
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
    if looks_like_inline_json(input) {
        return false;
    }

    input.starts_with("./")
        || input.starts_with("../")
        || input.starts_with('/')
        || input.starts_with("~/")
        || input.contains('/')
        || input.contains('\\')
        || has_supported_input_file_extension(input)
}

fn looks_like_inline_json(input: &str) -> bool {
    matches!(input.trim_start().chars().next(), Some('{' | '[' | '"'))
}

fn has_supported_input_file_extension(input: &str) -> bool {
    Path::new(input)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "txt" | "log" | "eml" | "json"
            )
        })
}

#[cfg(test)]
mod tests {
    use super::{is_path_like, looks_like_inline_json};

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
        assert!(is_path_like("resend-bounced.json"));
        assert!(!is_path_like("-"));
        assert!(!is_path_like("mailbox.full"));
    }

    #[test]
    fn json_payloads_are_not_treated_as_paths() {
        let object = r#"{"type":"email.bounced","url":"https://example.com/events/123"}"#;
        let array = r#"[{"url":"https://example.com/events/123"}]"#;
        let scalar = r#""https://example.com/events/123""#;

        assert!(looks_like_inline_json(object));
        assert!(looks_like_inline_json(array));
        assert!(looks_like_inline_json(scalar));
        assert!(looks_like_inline_json(&format!("  {object}")));
        assert!(!is_path_like(object));
        assert!(!is_path_like(array));
        assert!(!is_path_like(scalar));
    }
}
