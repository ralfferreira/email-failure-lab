use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;

#[test]
fn explains_inline_text() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", "550 5.1.1 User unknown"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Failure: Invalid recipient"))
        .stdout(predicate::str::contains(
            "Recommended action: Suppress recipient",
        ))
        .stdout(predicate::str::contains("matched_phrase: user unknown"));
}

#[test]
fn explains_inline_text_as_json() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", "550 5.1.1 User unknown", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""schemaVersion": "0.1""#))
        .stdout(predicate::str::contains(
            r#""category": "invalid_recipient""#,
        ))
        .stdout(predicate::str::contains(r#""bounceType": "hard""#));
}

#[test]
fn explains_file_input() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");

    command
        .current_dir(workspace_root)
        .args([
            "explain",
            "./crates/email-failure-core/fixtures/raw/invalid-recipient.txt",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Failure: Invalid recipient"));
}

#[test]
fn missing_path_like_input_is_an_error() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", "./missing-bounce.txt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no file exists"));
}

#[test]
fn empty_input_is_an_error() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", "   "])
        .assert()
        .failure()
        .stderr(predicate::str::contains("input cannot be empty"));
}

#[test]
fn enhanced_status_dots_are_treated_as_inline_input() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", "5.1.1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Failure: Invalid recipient"));
}
