use email_failure_core::{
    explain, BounceType, FailureCategory, FailureReport, InputSource, ParseInput, RecommendedAction,
};

struct ProviderCase {
    name: &'static str,
    path: &'static str,
    raw: &'static str,
    category: FailureCategory,
    bounce_type: BounceType,
    recommended_action: RecommendedAction,
}

#[test]
fn classifies_sanitized_resend_style_provider_fixtures() {
    let cases = [
        ProviderCase {
            name: "email.bounced invalid recipient",
            path: "fixtures/providers/resend/email-bounced-invalid-recipient.json",
            raw: include_str!("../fixtures/providers/resend/email-bounced-invalid-recipient.json"),
            category: FailureCategory::InvalidRecipient,
            bounce_type: BounceType::Hard,
            recommended_action: RecommendedAction::SuppressRecipient,
        },
        ProviderCase {
            name: "email.bounced authentication failure",
            path: "fixtures/providers/resend/email-bounced-authentication-failure.json",
            raw: include_str!(
                "../fixtures/providers/resend/email-bounced-authentication-failure.json"
            ),
            category: FailureCategory::AuthenticationFailure,
            bounce_type: BounceType::Hard,
            recommended_action: RecommendedAction::FixDomainAuthentication,
        },
        ProviderCase {
            name: "email.failed reached daily quota",
            path: "fixtures/providers/resend/email-failed-reached-daily-quota.json",
            raw: include_str!("../fixtures/providers/resend/email-failed-reached-daily-quota.json"),
            category: FailureCategory::RateLimited,
            bounce_type: BounceType::Soft,
            recommended_action: RecommendedAction::ReduceSendingRate,
        },
        ProviderCase {
            name: "Resend-like email.bounced temporary failure",
            path: "fixtures/providers/resend/email-bounced-temporary-failure-resend-like.json",
            raw: include_str!(
                "../fixtures/providers/resend/email-bounced-temporary-failure-resend-like.json"
            ),
            category: FailureCategory::TemporaryFailure,
            bounce_type: BounceType::Soft,
            recommended_action: RecommendedAction::RetryLater,
        },
    ];

    for case in cases {
        let report = explain(ParseInput {
            raw: case.raw,
            source: InputSource::File {
                path: case.path.to_owned(),
            },
        });

        assert_eq!(report.schema_version, "0.1", "{} schema", case.name);
        assert_eq!(report.category, case.category, "{} category", case.name);
        assert_eq!(
            report.bounce_type, case.bounce_type,
            "{} bounce type",
            case.name
        );
        assert_eq!(
            report.recommended_action, case.recommended_action,
            "{} action",
            case.name
        );
    }
}

#[test]
fn valid_unsupported_event_ignores_misleading_metadata() {
    let report =
        explain_inline(r#"{"type":"email.delivered","data":{"subject":"550 5.1.1 User unknown"}}"#);

    assert_unknown_report(&report);
}

#[test]
fn valid_non_object_json_is_unknown() {
    let report = explain_inline(r#"["550 5.1.1 User unknown"]"#);

    assert_unknown_report(&report);
}

#[test]
fn supported_event_missing_failure_subtree_is_unknown() {
    let report =
        explain_inline(r#"{"type":"email.bounced","data":{"subject":"550 5.1.1 User unknown"}}"#);

    assert_unknown_report(&report);
}

#[test]
fn malformed_json_falls_back_to_plain_text_classification() {
    let report = explain_inline(r#"{"type":"email.bounced", 550 5.1.1 User unknown"#);

    assert_eq!(report.schema_version, "0.1");
    assert_eq!(report.category, FailureCategory::InvalidRecipient);
    assert_eq!(
        report.recommended_action,
        RecommendedAction::SuppressRecipient
    );
}

fn explain_inline(raw: &str) -> FailureReport {
    explain(ParseInput {
        raw,
        source: InputSource::Inline,
    })
}

fn assert_unknown_report(report: &FailureReport) {
    assert_eq!(report.schema_version, "0.1");
    assert_eq!(report.category, FailureCategory::Unknown);
    assert_eq!(report.bounce_type, BounceType::Unknown);
    assert_eq!(report.recommended_action, RecommendedAction::Unknown);
    assert!(report.signals.is_empty());
}
