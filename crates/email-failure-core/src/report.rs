use std::collections::HashSet;

use crate::classify::{category_for_signal, classify_failure, infer_bounce_type};
use crate::model::{
    Confidence, ConfidenceLevel, FailureCategory, FailureReport, ParseInput, ParsedFailure,
    RecommendedAction, SignalKind,
};
use crate::parse::parse_failure;
use crate::recommend::recommend_action;

const SCHEMA_VERSION: &str = "0.1";

#[must_use]
pub fn explain(input: ParseInput<'_>) -> FailureReport {
    let parsed = parse_failure(input);
    let category = classify_failure(&parsed);
    let bounce_type = infer_bounce_type(&parsed, &category);
    let recommended_action = recommend_action(&category, &bounce_type);
    let confidence = calculate_confidence(&parsed);

    FailureReport {
        schema_version: SCHEMA_VERSION.to_owned(),
        category,
        bounce_type,
        recommended_action,
        confidence,
        explanation: explanation_for(category).to_owned(),
        app_guidance: guidance_for(recommended_action),
        signals: parsed.signals,
    }
}

#[must_use]
pub fn calculate_confidence(parsed: &ParsedFailure) -> Confidence {
    let mut score = parsed
        .signals
        .iter()
        .map(|signal| i16::from(signal.weight))
        .sum::<i16>();

    let categories = parsed
        .signals
        .iter()
        .filter_map(|signal| category_for_signal(signal.kind, &signal.value))
        .collect::<Vec<_>>();

    let unique_categories = categories.iter().copied().collect::<HashSet<_>>();

    if categories.len() >= 2 && unique_categories.len() == 1 {
        score += 10;
    } else if unique_categories.len() > 1 {
        score -= 20;
    }

    if !unique_categories.is_empty()
        && parsed.signals.iter().any(|signal| {
            signal.kind == SignalKind::EnhancedStatusCode
                || (signal.kind == SignalKind::MatchedPhrase && signal.weight >= 35)
        })
    {
        score = score.max(60);
    }

    if parsed
        .signals
        .iter()
        .all(|signal| signal.kind == SignalKind::SmtpCode)
    {
        score = score.min(59);
    }

    let score = score.clamp(1, 99) as u8;
    let level = match score {
        90..=99 => ConfidenceLevel::High,
        60..=89 => ConfidenceLevel::Medium,
        _ => ConfidenceLevel::Low,
    };

    Confidence { level, score }
}

fn explanation_for(category: FailureCategory) -> &'static str {
    match category {
        FailureCategory::InvalidRecipient => {
            "The recipient address appears not to exist or cannot receive mail."
        }
        FailureCategory::MailboxFull => {
            "The recipient mailbox appears to be full or over quota."
        }
        FailureCategory::AuthenticationFailure => {
            "The receiving server rejected the message because sender authentication appears to be failing."
        }
        FailureCategory::PolicyRejection => {
            "The receiving server rejected the message because of a policy decision."
        }
        FailureCategory::RateLimited => {
            "The receiving server or provider is asking you to slow down sending."
        }
        FailureCategory::TemporaryFailure => {
            "The failure appears temporary, so a later retry may succeed."
        }
        FailureCategory::ContentRejected => {
            "The receiving system appears to have rejected the message content."
        }
        FailureCategory::ProviderError => {
            "The failure appears related to the email provider or an upstream service."
        }
        FailureCategory::Unknown => {
            "Email Failure Lab could not confidently classify this failure yet."
        }
    }
}

fn guidance_for(action: RecommendedAction) -> Vec<String> {
    match action {
        RecommendedAction::SuppressRecipient => vec![
            "Stop sending to this address.".to_owned(),
            "Mark the email as invalid.".to_owned(),
            "Ask the user to update their email address.".to_owned(),
        ],
        RecommendedAction::RetryLater => vec![
            "Retry later with exponential backoff.".to_owned(),
            "Keep the original failure context for debugging.".to_owned(),
            "Avoid retrying indefinitely.".to_owned(),
        ],
        RecommendedAction::FixDomainAuthentication => vec![
            "Check SPF, DKIM, and DMARC for the sending domain.".to_owned(),
            "Verify that the provider is authorized to send for this domain.".to_owned(),
            "Retry only after authentication is fixed.".to_owned(),
        ],
        RecommendedAction::ReduceSendingRate => vec![
            "Reduce sending rate for this destination.".to_owned(),
            "Use backoff before retrying.".to_owned(),
            "Avoid retry storms that can worsen throttling.".to_owned(),
        ],
        RecommendedAction::ReviewContent => vec![
            "Review message content, links, headers, and sending patterns.".to_owned(),
            "Check recipient or provider policy requirements.".to_owned(),
            "Retry only after changing the likely cause.".to_owned(),
        ],
        RecommendedAction::InvestigateProvider => vec![
            "Check provider status and logs.".to_owned(),
            "Keep the raw error for support or incident review.".to_owned(),
            "Retry only if the provider indicates the issue is transient.".to_owned(),
        ],
        RecommendedAction::ContactRecipient => vec![
            "Ask the recipient to confirm their mailbox can receive mail.".to_owned(),
            "Retry only after the recipient confirms the issue is resolved.".to_owned(),
        ],
        RecommendedAction::NoActionRequired => {
            vec!["No immediate app action appears required.".to_owned()]
        }
        RecommendedAction::Unknown => vec![
            "Keep the raw failure for manual investigation.".to_owned(),
            "Add a fixture if this failure becomes common.".to_owned(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::model::{
        ConfidenceLevel, FailureCategory, InputSource, ParseInput, RecommendedAction, SignalKind,
    };

    use super::{calculate_confidence, explain};

    #[test]
    fn builds_stable_json_schema() {
        let report = explain(ParseInput {
            raw: "550 5.1.1 User unknown",
            source: InputSource::Inline,
        });
        let value = serde_json::to_value(&report).expect("report serializes");

        assert_eq!(value["schemaVersion"], "0.1");
        assert_eq!(value["category"], "invalid_recipient");
        assert_eq!(value["bounceType"], "hard");
        assert_eq!(value["recommendedAction"], "suppress_recipient");
        assert_eq!(value["confidence"]["level"], "high");
        assert_eq!(
            value["signals"][0],
            json!({ "kind": "smtp_code", "value": "550", "weight": 20 })
        );
    }

    #[test]
    fn partial_enhanced_status_is_medium_confidence() {
        let report = explain(ParseInput {
            raw: "5.1.1",
            source: InputSource::Inline,
        });

        assert_eq!(report.category, FailureCategory::InvalidRecipient);
        assert_eq!(report.confidence.level, ConfidenceLevel::Medium);
    }

    #[test]
    fn generic_550_is_unknown_low_confidence() {
        let report = explain(ParseInput {
            raw: "550",
            source: InputSource::Inline,
        });

        assert_eq!(report.category, FailureCategory::Unknown);
        assert_eq!(report.confidence.level, ConfidenceLevel::Low);
    }

    #[test]
    fn unknown_input_still_returns_report() {
        let report = explain(ParseInput {
            raw: "some unusual remote host response",
            source: InputSource::Inline,
        });

        assert_eq!(report.category, FailureCategory::Unknown);
        assert_eq!(report.recommended_action, RecommendedAction::Unknown);
        assert!(report.signals.is_empty());
    }

    #[test]
    fn confidence_penalizes_conflicting_strong_signals() {
        let parsed = crate::parse::parse_failure(ParseInput {
            raw: "550 5.1.1 mailbox full",
            source: InputSource::Inline,
        });
        let confidence = calculate_confidence(&parsed);

        assert!(confidence.score < 90);
    }

    #[test]
    fn report_contains_expected_signal_kinds() {
        let report = explain(ParseInput {
            raw: "550 5.1.1 User unknown",
            source: InputSource::Inline,
        });
        let kinds = report
            .signals
            .iter()
            .map(|signal| signal.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            kinds,
            vec![
                SignalKind::SmtpCode,
                SignalKind::EnhancedStatusCode,
                SignalKind::MatchedPhrase
            ]
        );
    }
}
