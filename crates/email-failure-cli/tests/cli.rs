use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

const RESEND_BOUNCED_INVALID_RECIPIENT: &str = r#"{
  "type": "email.bounced",
  "data": {
    "bounce": {
      "message": "550 5.1.1 User unknown",
      "type": "Permanent",
      "subType": "General"
    }
  }
}"#;

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
fn root_help_mentions_provider_webhook_json() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("provider webhook JSON"));
}

#[test]
fn explain_help_mentions_provider_webhook_json() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("provider webhook JSON"));
}

#[test]
fn explains_inline_provider_webhook_json() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", RESEND_BOUNCED_INVALID_RECIPIENT, "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""schemaVersion": "0.1""#))
        .stdout(predicate::str::contains(
            r#""category": "invalid_recipient""#,
        ));
}

#[test]
fn inline_provider_json_with_url_is_not_treated_as_a_path() {
    let payload = r#"{
      "type": "email.bounced",
      "data": {
        "documentationUrl": "https://example.com/failures/123",
        "bounce": {
          "message": "550 5.1.1 User unknown",
          "type": "Permanent",
          "subType": "General"
        }
      }
    }"#;
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", payload, "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            r#""category": "invalid_recipient""#,
        ));
}

#[test]
fn unsupported_inline_provider_json_returns_unknown_report() {
    let payload = r#"{"type":"email.delivered","data":{"subject":"550 5.1.1 User unknown"}}"#;
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", payload, "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""category": "unknown""#))
        .stdout(predicate::str::contains(r#""signals": []"#));
}

#[test]
fn inline_json_array_with_url_is_not_treated_as_a_path() {
    let payload = r#"[{"url":"https://example.com/failures/123"}]"#;
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", payload, "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""category": "unknown""#));
}

#[test]
fn inline_json_string_with_url_returns_unknown_report() {
    let payload = r#""https://example.com/failures/550""#;
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", payload, "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""category": "unknown""#))
        .stdout(predicate::str::contains(r#""signals": []"#));
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
fn explains_provider_webhook_json_file() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let path = temp_dir.path().join("resend-bounced.json");
    fs::write(&path, RESEND_BOUNCED_INVALID_RECIPIENT).expect("write JSON fixture");

    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", path.to_str().expect("path is UTF-8"), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            r#""category": "invalid_recipient""#,
        ));
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
fn explains_provider_webhook_json_from_stdin() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", "-", "--json"])
        .write_stdin(RESEND_BOUNCED_INVALID_RECIPIENT)
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
fn missing_json_input_is_an_error() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", "missing-resend-webhook.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no file exists"))
        .stderr(predicate::str::contains("missing-resend-webhook.json"));
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
        .stderr(predicate::str::contains("bounce.txt"));
}

#[test]
fn non_utf8_stdin_input_is_a_clear_error() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["explain", "-"])
        .write_stdin([0xff, 0xfe, 0xfd])
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not read stdin text"));
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
