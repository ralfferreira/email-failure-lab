use assert_cmd::Command;
use email_failure_core::find_built_in_fixture;
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

const EXPECTED_BUILT_IN_FIXTURES: &[(&str, &str)] = &[
    ("auth-failure", "authentication_failure"),
    ("invalid-recipient", "invalid_recipient"),
    ("mailbox-full", "mailbox_full"),
    ("plain-bounce", "invalid_recipient"),
    ("resend-authentication-failure", "authentication_failure"),
    ("resend-daily-quota", "rate_limited"),
    ("resend-invalid-recipient", "invalid_recipient"),
    ("resend-temporary-failure", "temporary_failure"),
];

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
fn root_help_lists_fixture_discovery() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "fixtures  Discover and inspect built-in failure fixtures",
        ));
}

#[test]
fn fixtures_help_lists_list_and_show_subcommands() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["fixtures", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Discover and inspect built-in failure fixtures",
        ))
        .stdout(predicate::str::contains(
            "list  List the available built-in fixtures",
        ))
        .stdout(predicate::str::contains(
            "show  Show a fixture's input and expected classification",
        ));
}

#[test]
fn lists_all_built_in_fixtures_in_sorted_order_from_any_working_directory() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .current_dir(temp_dir.path())
        .args(["fixtures", "list"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::eq(expected_fixture_list_output()));
}

#[test]
fn shows_invalid_recipient_input_and_expected_metadata_from_any_working_directory() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .current_dir(temp_dir.path())
        .args(["fixtures", "show", "invalid-recipient"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::eq(expected_fixture_show_output(
            "invalid-recipient",
            built_in_fixture_input("invalid-recipient"),
            "invalid_recipient",
            "hard",
            "suppress_recipient",
            "high",
        )));
}

#[test]
fn shows_provider_fixture_input_and_expected_metadata() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["fixtures", "show", "resend-daily-quota"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::eq(expected_fixture_show_output(
            "resend-daily-quota",
            built_in_fixture_input("resend-daily-quota"),
            "rate_limited",
            "soft",
            "reduce_sending_rate",
            "medium",
        )));
}

#[test]
fn unknown_fixture_is_a_clear_error_with_a_list_hint() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .current_dir(temp_dir.path())
        .args(["fixtures", "show", "does-not-exist"])
        .assert()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::eq(
            "unknown fixture 'does-not-exist'; run 'email-lab fixtures list' to see available fixtures\n",
        ));
}

#[test]
fn fixture_show_requires_a_name() {
    let mut command = Command::cargo_bin("email-lab").expect("binary exists");

    command
        .args(["fixtures", "show"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("<NAME>"));
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

fn expected_fixture_list_output() -> String {
    let name_width = EXPECTED_BUILT_IN_FIXTURES
        .iter()
        .map(|(name, _)| name.len())
        .max()
        .expect("fixture list is not empty");
    let mut output = format!(
        "Built-in fixtures ({}):\n",
        EXPECTED_BUILT_IN_FIXTURES.len()
    );

    for (name, category) in EXPECTED_BUILT_IN_FIXTURES {
        output.push_str(&format!("  {name:name_width$}  {category}\n"));
    }

    output.push_str("\nUse 'email-lab fixtures show <name>' to inspect a fixture.\n");
    output
}

fn built_in_fixture_input(name: &str) -> &'static str {
    find_built_in_fixture(name)
        .unwrap_or_else(|| panic!("expected built-in fixture '{name}'"))
        .input
}

fn expected_fixture_show_output(
    name: &str,
    input: &str,
    category: &str,
    bounce_type: &str,
    recommended_action: &str,
    confidence_level: &str,
) -> String {
    let mut output = format!("Fixture: {name}\n\nInput:\n");
    output.push_str(input);
    if !input.ends_with('\n') {
        output.push('\n');
    }
    output.push_str("\nExpected classification:\n");
    output.push_str(&format!("  category: {category}\n"));
    output.push_str(&format!("  bounce_type: {bounce_type}\n"));
    output.push_str(&format!("  recommended_action: {recommended_action}\n"));
    output.push_str(&format!("  confidence_level: {confidence_level}\n"));
    output
}
