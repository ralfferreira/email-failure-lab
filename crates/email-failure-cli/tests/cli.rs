use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
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
fn explains_inline_text_with_format_json() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", "550 5.1.1 User unknown", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""schemaVersion": "0.1""#))
        .stdout(predicate::str::contains(
            r#""category": "invalid_recipient""#,
        ));
}

#[test]
fn verbose_output_includes_signal_weights() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", "550 5.1.1 User unknown", "--verbose"])
        .assert()
        .success()
        .stdout(predicate::str::contains("- smtp_code: 550 (weight: 20)"))
        .stdout(predicate::str::contains(
            "- enhanced_status_code: 5.1.1 (weight: 35)",
        ))
        .stdout(predicate::str::contains(
            "- matched_phrase: user unknown (weight: 35)",
        ));
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
fn explains_plain_eml_file_as_text() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");

    command
        .current_dir(workspace_root)
        .args([
            "explain",
            "./crates/email-failure-core/fixtures/raw/plain-bounce.eml",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Failure: Invalid recipient"));
}

#[test]
fn explains_stdin_input() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", "-"])
        .write_stdin("550 5.1.1 User unknown")
        .assert()
        .success()
        .stdout(predicate::str::contains("Failure: Invalid recipient"));
}

#[test]
fn explains_stdin_input_as_json() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", "-", "--json"])
        .write_stdin("550 5.1.1 User unknown")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            r#""category": "invalid_recipient""#,
        ));
}

#[test]
fn empty_stdin_input_is_an_error() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", "-"])
        .write_stdin("   ")
        .assert()
        .failure()
        .stderr(predicate::str::contains("input cannot be empty"));
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
fn non_utf8_file_input_is_a_clear_error() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let path = temp_dir.path().join("bounce.txt");
    fs::write(&path, [0xff, 0xfe, 0xfd]).expect("write invalid UTF-8 fixture");

    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", path.to_str().expect("path is UTF-8")])
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not read input file"))
        .stderr(predicate::str::contains("bounce.txt"))
        .stderr(predicate::str::contains("UTF-8 text"));
}

#[test]
fn non_utf8_stdin_input_is_a_clear_error() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", "-"])
        .write_stdin([0xff, 0xfe, 0xfd])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "could not read stdin as UTF-8 text",
        ));
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
