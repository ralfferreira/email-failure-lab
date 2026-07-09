use email_failure_core::{explain, InputSource, ParseInput};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct FixtureCase {
    name: String,
    input: String,
}

#[test]
fn fixture_reports_match_failure_report_v0_1_schema() {
    let schema = failure_report_schema();
    let validator = jsonschema::validator_for(&schema).expect("failure report schema compiles");
    let fixtures: Vec<FixtureCase> =
        serde_json::from_str(include_str!("../fixtures/cases.json")).expect("valid fixtures");

    for fixture in fixtures {
        let report = explain(ParseInput {
            raw: &fixture.input,
            source: InputSource::Inline,
        });
        let report_json = serde_json::to_value(report).expect("report serializes");

        if let Err(error) = validator.validate(&report_json) {
            panic!(
                "fixture report should match schema for {}: {error}",
                fixture.name
            );
        }
    }
}

#[test]
fn provider_payload_reports_match_failure_report_v0_1_schema() {
    let schema = failure_report_schema();
    let validator = jsonschema::validator_for(&schema).expect("failure report schema compiles");
    let fixtures = [
        (
            "email.bounced invalid recipient",
            include_str!("../fixtures/providers/resend/email-bounced-invalid-recipient.json"),
        ),
        (
            "email.bounced authentication failure",
            include_str!("../fixtures/providers/resend/email-bounced-authentication-failure.json"),
        ),
        (
            "email.failed reached daily quota",
            include_str!("../fixtures/providers/resend/email-failed-reached-daily-quota.json"),
        ),
        (
            "Resend-like email.bounced temporary failure",
            include_str!(
                "../fixtures/providers/resend/email-bounced-temporary-failure-resend-like.json"
            ),
        ),
    ];

    for (name, raw) in fixtures {
        let report = explain(ParseInput {
            raw,
            source: InputSource::Inline,
        });
        let report_json = serde_json::to_value(report).expect("report serializes");

        if let Err(error) = validator.validate(&report_json) {
            panic!("provider payload report should match schema for {name}: {error}");
        }
    }
}

#[test]
fn schema_rejects_unexpected_report_fields() {
    let schema = failure_report_schema();
    let validator = jsonschema::validator_for(&schema).expect("failure report schema compiles");
    let report = explain(ParseInput {
        raw: "550 5.1.1 User unknown",
        source: InputSource::Inline,
    });
    let mut report_json = serde_json::to_value(report).expect("report serializes");

    report_json["unexpected"] = Value::String("field".to_owned());

    assert!(
        validator.validate(&report_json).is_err(),
        "schema should reject unknown top-level fields"
    );
}

#[test]
fn schema_rejects_inconsistent_confidence_level_and_score() {
    let schema = failure_report_schema();
    let validator = jsonschema::validator_for(&schema).expect("failure report schema compiles");
    let report = explain(ParseInput {
        raw: "550 5.1.1 User unknown",
        source: InputSource::Inline,
    });
    let mut report_json = serde_json::to_value(report).expect("report serializes");

    report_json["confidence"]["score"] = Value::from(59);

    assert!(
        validator.validate(&report_json).is_err(),
        "schema should reject high confidence with a low confidence score"
    );
}

fn failure_report_schema() -> Value {
    serde_json::from_str(include_str!("../../../schemas/failure-report.v0.1.json"))
        .expect("valid JSON schema")
}
