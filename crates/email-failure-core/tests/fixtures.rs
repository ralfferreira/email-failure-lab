use email_failure_core::{
    explain, BounceType, ConfidenceLevel, FailureCategory, InputSource, ParseInput,
    RecommendedAction, SignalKind,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FixtureCase {
    name: String,
    input: String,
    category: FailureCategory,
    bounce_type: BounceType,
    recommended_action: RecommendedAction,
    confidence_level: ConfidenceLevel,
    required_signal_kinds: Vec<SignalKind>,
}

#[test]
fn fixtures_match_expected_reports() {
    let fixtures: Vec<FixtureCase> =
        serde_json::from_str(include_str!("../fixtures/cases.json")).expect("valid fixtures");

    assert!(
        fixtures.len() >= 20,
        "expected at least 20 fixtures, got {}",
        fixtures.len()
    );

    for fixture in fixtures {
        let report = explain(ParseInput {
            raw: &fixture.input,
            source: InputSource::Inline,
        });

        assert_eq!(report.category, fixture.category, "{}", fixture.name);
        assert_eq!(report.bounce_type, fixture.bounce_type, "{}", fixture.name);
        assert_eq!(
            report.recommended_action, fixture.recommended_action,
            "{}",
            fixture.name
        );
        assert_eq!(
            report.confidence.level, fixture.confidence_level,
            "{}",
            fixture.name
        );

        for required_kind in fixture.required_signal_kinds {
            assert!(
                report
                    .signals
                    .iter()
                    .any(|signal| signal.kind == required_kind),
                "{} missing required signal kind {required_kind:?}",
                fixture.name
            );
        }
    }
}
