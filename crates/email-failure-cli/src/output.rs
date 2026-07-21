use anstyle::{AnsiColor, Style};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextStyle {
    Plain,
    Color,
}

const LABEL_STYLE: Style = Style::new().bold();
const HEADING_STYLE: Style = AnsiColor::BrightBlue.on_default().bold();
const CATEGORY_STYLE: Style = AnsiColor::BrightMagenta.on_default();
const ACTION_STYLE: Style = AnsiColor::BrightCyan.on_default();

pub fn format_text(report: &FailureReport, verbose: bool, text_style: TextStyle) -> String {
    let mut output = String::new();

    push_field(
        &mut output,
        "Failure",
        display_category(report.category),
        CATEGORY_STYLE,
        text_style,
    );
    push_field(
        &mut output,
        "Bounce type",
        display_bounce_type(report.bounce_type),
        bounce_type_style(report.bounce_type),
        text_style,
    );
    push_field(
        &mut output,
        "Recommended action",
        display_action(report.recommended_action),
        ACTION_STYLE,
        text_style,
    );
    push_field(
        &mut output,
        "Confidence",
        &format!(
            "{} ({}%)",
            display_confidence(report.confidence.level),
            report.confidence.score
        ),
        confidence_style(report.confidence.level),
        text_style,
    );
    output.push('\n');

    push_line(&mut output, paint("Why:", HEADING_STYLE, text_style));
    push_line(&mut output, &report.explanation);
    output.push('\n');

    push_line(
        &mut output,
        paint("What your app should do:", HEADING_STYLE, text_style),
    );
    for guidance in &report.app_guidance {
        push_line(&mut output, format!("- {guidance}"));
    }
    output.push('\n');

    push_line(&mut output, paint("Signals:", HEADING_STYLE, text_style));
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

fn push_field(
    output: &mut String,
    label: &str,
    value: &str,
    value_style: Style,
    text_style: TextStyle,
) {
    push_line(
        output,
        format!(
            "{}: {}",
            paint(label, LABEL_STYLE, text_style),
            paint(value, value_style, text_style)
        ),
    );
}

fn paint(value: &str, style: Style, text_style: TextStyle) -> String {
    match text_style {
        TextStyle::Plain => value.to_owned(),
        TextStyle::Color => format!("{style}{value}{style:#}"),
    }
}

fn bounce_type_style(bounce_type: BounceType) -> Style {
    match bounce_type {
        BounceType::Hard => AnsiColor::BrightRed.on_default(),
        BounceType::Soft => AnsiColor::BrightYellow.on_default(),
        BounceType::Unknown => AnsiColor::BrightBlack.on_default(),
    }
}

fn confidence_style(level: ConfidenceLevel) -> Style {
    match level {
        ConfidenceLevel::Low => AnsiColor::BrightRed.on_default(),
        ConfidenceLevel::Medium => AnsiColor::BrightYellow.on_default(),
        ConfidenceLevel::High => AnsiColor::BrightGreen.on_default(),
    }
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
    use anstyle::AnsiColor;
    use email_failure_core::{explain, BounceType, ConfidenceLevel, InputSource, ParseInput};

    use super::{bounce_type_style, confidence_style, format_text, TextStyle};

    #[test]
    fn text_output_matches_golden_shape() {
        let report = explain(ParseInput {
            raw: "550 5.1.1 User unknown",
            source: InputSource::Inline,
        });

        assert_eq!(
            format_text(&report, false, TextStyle::Plain),
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
    fn color_output_styles_key_fields_without_changing_text() {
        let report = explain(ParseInput {
            raw: "550 5.1.1 User unknown",
            source: InputSource::Inline,
        });
        let plain = format_text(&report, false, TextStyle::Plain);
        let color = format_text(&report, false, TextStyle::Color);

        assert_eq!(anstream::adapter::strip_str(&color).to_string(), plain);
        assert!(color.contains("\x1b[1mFailure\x1b[0m: \x1b[95mInvalid recipient\x1b[0m"));
        assert!(color.contains("\x1b[1mBounce type\x1b[0m: \x1b[91mHard bounce\x1b[0m"));
        assert!(color.contains(concat!(
            "\x1b[1mRecommended action\x1b[0m: ",
            "\x1b[96mSuppress recipient\x1b[0m"
        )));
        assert!(color.contains("\x1b[1mConfidence\x1b[0m: \x1b[92mHigh (99%)\x1b[0m"));
        assert!(color.contains("\x1b[1m\x1b[94mWhy:\x1b[0m"));
    }

    #[test]
    fn bounce_and_confidence_styles_cover_every_result_level() {
        assert_eq!(
            bounce_type_style(BounceType::Hard),
            AnsiColor::BrightRed.on_default()
        );
        assert_eq!(
            bounce_type_style(BounceType::Soft),
            AnsiColor::BrightYellow.on_default()
        );
        assert_eq!(
            bounce_type_style(BounceType::Unknown),
            AnsiColor::BrightBlack.on_default()
        );
        assert_eq!(
            confidence_style(ConfidenceLevel::Low),
            AnsiColor::BrightRed.on_default()
        );
        assert_eq!(
            confidence_style(ConfidenceLevel::Medium),
            AnsiColor::BrightYellow.on_default()
        );
        assert_eq!(
            confidence_style(ConfidenceLevel::High),
            AnsiColor::BrightGreen.on_default()
        );
    }
}
