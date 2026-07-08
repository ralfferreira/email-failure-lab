use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("input cannot be empty")]
    EmptyInput,
    #[error("could not read stdin text: {0}")]
    ReadStdin(#[source] std::io::Error),
    #[error("could not read input file '{path}': {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("input looks like a file path, but no file exists at '{0}'")]
    MissingFile(PathBuf),
    #[error("could not serialize report as JSON: {0}")]
    Json(#[from] serde_json::Error),
}
