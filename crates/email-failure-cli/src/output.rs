use clap::ValueEnum;
use email_failure_core::classify::rule_id_for_signal;
use email_failure_core::{
    BounceType, ConfidenceLevel, FailureCategory, FailureReport, RecommendedAction, SignalKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

pub fn format_text(report: &FailureReport, verbose: bool) -> String {
    let mut output = String::new();

    push_line(
        &mut output,
        format!("Failure: {}", display_category(report.category)),
    );
    push_line(
        &mut output,
        format!("Bounce type: {}", display_bounce_type(report.bounce_type)),
    );
    push_line(
        &mut output,
        format!(
            "Recommended action: {}",
            display_action(report.recommended_action)
        ),
    );
    push_line(
        &mut output,
        format!(
            "Confidence: {} ({}%)",
            display_confidence(report.confidence.level),
            report.confidence.score
        ),
    );
    output.push('\n');

    push_line(&mut output, "Why:");
    push_line(&mut output, &report.explanation);
    output.push('\n');

    push_line(&mut output, "What your app should do:");
    for guidance in &report.app_guidance {
        push_line(&mut output, format!("- {guidance}"));
    }
    output.push('\n');

    push_line(&mut output, "Signals:");
    if report.signals.is_empty() {
        push_line(&mut output, "- none");
    } else {
        for signal in &report.signals {
            if verbose {
                let metadata = rule_id_for_signal(signal.kind, &signal.value).map_or_else(
                    || format!("weight: {}", signal.weight),
                    |rule_id| format!("weight: {}, rule_id: {rule_id}", signal.weight),
                );
                push_line(
                    &mut output,
                    format!(
                        "- {}: {} ({metadata})",
                        display_signal_kind(signal.kind),
                        signal.value
                    ),
                );
            } else {
                push_line(
                    &mut output,
                    format!("- {}: {}", display_signal_kind(signal.kind), signal.value),
                );
            }
        }
    }

    output
}

pub fn format_json(report: &FailureReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(report)
}

fn push_line(output: &mut String, line: impl AsRef<str>) {
    output.push_str(line.as_ref());
    output.push('\n');
}

fn display_category(category: FailureCategory) -> &'static str {
    match category {
        FailureCategory::InvalidRecipient => "Invalid recipient",
        FailureCategory::MailboxFull => "Mailbox full",
        FailureCategory::AuthenticationFailure => "Authentication failure",
        FailureCategory::PolicyRejection => "Policy rejection",
        FailureCategory::RateLimited => "Rate limited",
        FailureCategory::TemporaryFailure => "Temporary failure",
        FailureCategory::ContentRejected => "Content rejected",
        FailureCategory::ProviderError => "Provider error",
        FailureCategory::Unknown => "Unknown",
    }
}

fn display_bounce_type(bounce_type: BounceType) -> &'static str {
    match bounce_type {
        BounceType::Hard => "Hard bounce",
        BounceType::Soft => "Soft bounce",
        BounceType::Unknown => "Unknown",
    }
}

fn display_action(action: RecommendedAction) -> &'static str {
    match action {
        RecommendedAction::SuppressRecipient => "Suppress recipient",
        RecommendedAction::RetryLater => "Retry later",
        RecommendedAction::FixDomainAuthentication => "Fix domain authentication",
        RecommendedAction::ReduceSendingRate => "Reduce sending rate",
        RecommendedAction::ReviewContent => "Review content",
        RecommendedAction::InvestigateProvider => "Investigate provider",
        RecommendedAction::ContactRecipient => "Contact recipient",
        RecommendedAction::NoActionRequired => "No action required",
        RecommendedAction::Unknown => "Unknown",
    }
}

fn display_confidence(level: ConfidenceLevel) -> &'static str {
    match level {
        ConfidenceLevel::Low => "Low",
        ConfidenceLevel::Medium => "Medium",
        ConfidenceLevel::High => "High",
    }
}

fn display_signal_kind(kind: SignalKind) -> &'static str {
    match kind {
        SignalKind::SmtpCode => "smtp_code",
        SignalKind::EnhancedStatusCode => "enhanced_status_code",
        SignalKind::MatchedPhrase => "matched_phrase",
    }
}

#[cfg(test)]
mod tests {
    use email_failure_core::{explain, InputSource, ParseInput};

    use super::format_text;

    #[test]
    fn text_output_matches_golden_shape() {
        let report = explain(ParseInput {
            raw: "550 5.1.1 User unknown",
            source: InputSource::Inline,
        });

        assert_eq!(
            format_text(&report, false),
            concat!(
                "Failure: Invalid recipient\n",
                "Bounce type: Hard bounce\n",
                "Recommended action: Suppress recipient\n",
                "Confidence: High (99%)\n",
                "\n",
                "Why:\n",
                "The recipient address appears not to exist or cannot receive mail.\n",
                "\n",
                "What your app should do:\n",
                "- Stop sending to this address.\n",
                "- Mark the email as invalid.\n",
                "- Ask the user to update their email address.\n",
                "\n",
                "Signals:\n",
                "- smtp_code: 550\n",
                "- enhanced_status_code: 5.1.1\n",
                "- matched_phrase: user unknown\n",
            )
        );
    }

    #[test]
    fn verbose_text_includes_rule_ids_without_adding_one_to_smtp_codes() {
        let report = explain(ParseInput {
            raw: "550 5.1.1 User unknown",
            source: InputSource::Inline,
        });
        let output = format_text(&report, true);

        assert!(output.contains("- smtp_code: 550 (weight: 20)"));
        assert!(output.contains(concat!(
            "- enhanced_status_code: 5.1.1 ",
            "(weight: 35, rule_id: enhanced_status.invalid_recipient.5_1_1)"
        )));
        assert!(output.contains(concat!(
            "- matched_phrase: user unknown ",
            "(weight: 35, rule_id: phrase.invalid_recipient.user_unknown)"
        )));
    }
}
