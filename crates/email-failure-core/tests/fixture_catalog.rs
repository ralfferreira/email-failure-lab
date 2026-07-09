use std::collections::HashSet;

use email_failure_core::{
    built_in_fixtures, explain, find_built_in_fixture, InputSource, ParseInput,
};

const EXPECTED_NAMES: &[&str] = &[
    "auth-failure",
    "invalid-recipient",
    "mailbox-full",
    "plain-bounce",
    "resend-authentication-failure",
    "resend-daily-quota",
    "resend-invalid-recipient",
    "resend-temporary-failure",
];

#[test]
fn built_in_fixture_names_are_stable_unique_sorted_kebab_case() {
    let fixtures = built_in_fixtures();
    let names = fixtures
        .iter()
        .map(|fixture| fixture.name)
        .collect::<Vec<_>>();

    assert_eq!(names, EXPECTED_NAMES);
    assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(
        names.iter().copied().collect::<HashSet<_>>().len(),
        names.len()
    );

    for name in names {
        assert!(
            is_kebab_case(name),
            "fixture name is not kebab-case: {name}"
        );
    }
}

#[test]
fn built_in_fixture_lookup_finds_every_fixture_by_exact_name() {
    for fixture in built_in_fixtures() {
        assert_eq!(find_built_in_fixture(fixture.name), Some(fixture));
    }

    assert_eq!(find_built_in_fixture("missing-fixture"), None);
    assert_eq!(find_built_in_fixture("INVALID-RECIPIENT"), None);
}

#[test]
fn built_in_fixture_expectations_match_classifier_outputs() {
    for fixture in built_in_fixtures() {
        let report = explain(ParseInput {
            raw: fixture.input,
            source: InputSource::Inline,
        });

        assert_eq!(
            report.category, fixture.expected.category,
            "{} category drifted",
            fixture.name
        );
        assert_eq!(
            report.bounce_type, fixture.expected.bounce_type,
            "{} bounce type drifted",
            fixture.name
        );
        assert_eq!(
            report.recommended_action, fixture.expected.recommended_action,
            "{} recommended action drifted",
            fixture.name
        );
        assert_eq!(
            report.confidence.level, fixture.expected.confidence_level,
            "{} confidence level drifted",
            fixture.name
        );
    }
}

fn is_kebab_case(name: &str) -> bool {
    !name.is_empty()
        && name.split('-').all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}
