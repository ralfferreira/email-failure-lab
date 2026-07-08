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
    #[serde(default)]
    required_signals: Vec<ExpectedSignal>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpectedSignal {
    kind: SignalKind,
    value: String,
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

        for required_signal in fixture.required_signals {
            assert!(
                report.signals.iter().any(|signal| {
                    signal.kind == required_signal.kind && signal.value == required_signal.value
                }),
                "{} missing required signal {:?}: {}",
                fixture.name,
                required_signal.kind,
                required_signal.value
            );
        }
    }
}

#[test]
fn multiline_fixtures_cover_split_signals() {
    let fixtures: Vec<FixtureCase> =
        serde_json::from_str(include_str!("../fixtures/cases.json")).expect("valid fixtures");

    let multiline_fixtures = fixtures
        .iter()
        .filter(|fixture| fixture.input.lines().count() > 1)
        .collect::<Vec<_>>();

    assert!(
        multiline_fixtures.len() >= 5,
        "expected at least 5 multiline fixtures, got {}",
        multiline_fixtures.len()
    );

    for fixture in multiline_fixtures {
        assert!(
            !fixture.required_signals.is_empty(),
            "{} should pin concrete signals detected from multiline input",
            fixture.name
        );

        let normalized_input = normalize_for_fixture_assertion(&fixture.input);
        let has_signal_split_across_lines =
            fixture.required_signals.iter().any(|required_signal| {
                normalized_input.contains(&required_signal.value)
                    && !fixture.input.lines().any(|line| {
                        normalize_for_fixture_assertion(line).contains(&required_signal.value)
                    })
            });

        assert!(
            has_signal_split_across_lines,
            "{} should include at least one required signal split across lines",
            fixture.name
        );
    }
}

fn normalize_for_fixture_assertion(input: &str) -> String {
    input
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
