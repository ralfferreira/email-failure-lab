use std::collections::HashMap;

use crate::model::{BounceType, FailureCategory, ParsedFailure, SignalKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhraseRule {
    pub phrase: &'static str,
    pub category: FailureCategory,
    pub strong: bool,
    pub priority: u8,
}

impl PhraseRule {
    #[must_use]
    pub fn all() -> &'static [Self] {
        PHRASE_RULES
    }

    const fn strong(phrase: &'static str, category: FailureCategory, priority: u8) -> Self {
        Self {
            phrase,
            category,
            strong: true,
            priority,
        }
    }

    const fn weak(phrase: &'static str, category: FailureCategory, priority: u8) -> Self {
        Self {
            phrase,
            category,
            strong: false,
            priority,
        }
    }
}

static PHRASE_RULES: &[PhraseRule] = &[
    PhraseRule::strong(
        "message rejected as spam",
        FailureCategory::ContentRejected,
        100,
    ),
    PhraseRule::strong("classified as spam", FailureCategory::ContentRejected, 100),
    PhraseRule::strong("content rejected", FailureCategory::ContentRejected, 100),
    PhraseRule::strong("spam content", FailureCategory::ContentRejected, 100),
    PhraseRule::strong("spam detected", FailureCategory::ContentRejected, 100),
    PhraseRule::strong(
        "unauthenticated email",
        FailureCategory::AuthenticationFailure,
        95,
    ),
    PhraseRule::strong("spf fail", FailureCategory::AuthenticationFailure, 95),
    PhraseRule::strong("dkim fail", FailureCategory::AuthenticationFailure, 95),
    PhraseRule::strong("dmarc fail", FailureCategory::AuthenticationFailure, 95),
    PhraseRule::strong("user unknown", FailureCategory::InvalidRecipient, 90),
    PhraseRule::strong(
        "recipient address rejected",
        FailureCategory::InvalidRecipient,
        90,
    ),
    PhraseRule::strong("mailbox unavailable", FailureCategory::InvalidRecipient, 90),
    PhraseRule::strong("no such user", FailureCategory::InvalidRecipient, 90),
    PhraseRule::strong("mailbox full", FailureCategory::MailboxFull, 90),
    PhraseRule::strong("quota exceeded", FailureCategory::MailboxFull, 90),
    PhraseRule::strong("over quota", FailureCategory::MailboxFull, 90),
    PhraseRule::strong("rate limited", FailureCategory::RateLimited, 90),
    PhraseRule::strong("too many messages", FailureCategory::RateLimited, 90),
    PhraseRule::strong("throttled", FailureCategory::RateLimited, 90),
    PhraseRule::strong("temporary failure", FailureCategory::TemporaryFailure, 80),
    PhraseRule::strong(
        "temporary system problem",
        FailureCategory::TemporaryFailure,
        80,
    ),
    PhraseRule::strong(
        "temporarily unavailable",
        FailureCategory::TemporaryFailure,
        80,
    ),
    PhraseRule::strong("try again later", FailureCategory::TemporaryFailure, 80),
    PhraseRule::strong("provider error", FailureCategory::ProviderError, 80),
    PhraseRule::strong("internal error", FailureCategory::ProviderError, 80),
    PhraseRule::strong("upstream error", FailureCategory::ProviderError, 80),
    PhraseRule::weak("rejected by policy", FailureCategory::PolicyRejection, 70),
    PhraseRule::weak("not accepted", FailureCategory::PolicyRejection, 60),
    PhraseRule::weak("blocked", FailureCategory::PolicyRejection, 60),
    PhraseRule::weak("message rejected", FailureCategory::PolicyRejection, 50),
];

#[must_use]
pub fn classify_failure(parsed: &ParsedFailure) -> FailureCategory {
    if let Some(category) = strongest_strong_phrase_category(parsed) {
        return category;
    }

    signal_categories(parsed)
        .into_iter()
        .max_by_key(|(_, score)| *score)
        .map_or(FailureCategory::Unknown, |(category, _)| category)
}

#[must_use]
pub fn infer_bounce_type(_parsed: &ParsedFailure, category: &FailureCategory) -> BounceType {
    match category {
        FailureCategory::InvalidRecipient
        | FailureCategory::AuthenticationFailure
        | FailureCategory::PolicyRejection
        | FailureCategory::ContentRejected => BounceType::Hard,
        FailureCategory::MailboxFull
        | FailureCategory::RateLimited
        | FailureCategory::TemporaryFailure => BounceType::Soft,
        FailureCategory::ProviderError | FailureCategory::Unknown => BounceType::Unknown,
    }
}

#[must_use]
pub fn category_for_signal(kind: SignalKind, value: &str) -> Option<FailureCategory> {
    match kind {
        SignalKind::SmtpCode => category_for_smtp_code(value),
        SignalKind::EnhancedStatusCode => category_for_enhanced_status(value),
        SignalKind::MatchedPhrase => category_for_phrase(value),
    }
}

#[must_use]
pub fn category_for_smtp_code(code: &str) -> Option<FailureCategory> {
    match code {
        "421" | "451" => Some(FailureCategory::TemporaryFailure),
        _ => None,
    }
}

#[must_use]
pub fn category_for_enhanced_status(code: &str) -> Option<FailureCategory> {
    match code {
        "5.1.1" => Some(FailureCategory::InvalidRecipient),
        "5.2.2" => Some(FailureCategory::MailboxFull),
        "5.7.1" | "5.7.26" => Some(FailureCategory::PolicyRejection),
        "4.7.0" => Some(FailureCategory::TemporaryFailure),
        "4.3.0" => Some(FailureCategory::TemporaryFailure),
        _ => None,
    }
}

#[must_use]
pub fn category_for_phrase(phrase: &str) -> Option<FailureCategory> {
    PhraseRule::all()
        .iter()
        .find(|rule| rule.phrase == phrase)
        .map(|rule| rule.category)
}

#[must_use]
pub fn signal_categories(parsed: &ParsedFailure) -> HashMap<FailureCategory, u16> {
    let mut scores = HashMap::new();

    for signal in &parsed.signals {
        if let Some(category) = category_for_signal(signal.kind, &signal.value) {
            let specificity = signal_specificity(signal.kind, &signal.value);
            *scores.entry(category).or_insert(0) += u16::from(signal.weight) + specificity;
        }
    }

    scores
}

fn signal_specificity(kind: SignalKind, value: &str) -> u16 {
    match kind {
        SignalKind::MatchedPhrase => PhraseRule::all()
            .iter()
            .find(|rule| rule.phrase == value)
            .map_or(40, |rule| {
                if rule.strong {
                    u16::from(rule.priority)
                } else {
                    10
                }
            }),
        SignalKind::EnhancedStatusCode => 30,
        SignalKind::SmtpCode => 10,
    }
}

fn strongest_strong_phrase_category(parsed: &ParsedFailure) -> Option<FailureCategory> {
    parsed
        .signals
        .iter()
        .filter(|signal| signal.kind == SignalKind::MatchedPhrase)
        .filter_map(|signal| {
            PhraseRule::all()
                .iter()
                .find(|rule| rule.phrase == signal.value && rule.strong)
        })
        .max_by_key(|rule| rule.priority)
        .map(|rule| rule.category)
}

#[cfg(test)]
mod tests {
    use crate::model::{FailureCategory, InputSource, ParseInput};
    use crate::parse::parse_failure;

    use super::classify_failure;

    fn category(raw: &str) -> FailureCategory {
        let parsed = parse_failure(ParseInput {
            raw,
            source: InputSource::Inline,
        });
        classify_failure(&parsed)
    }

    #[test]
    fn classifies_required_priority_cases() {
        assert_eq!(
            category("550 5.7.1 Unauthenticated email from example.com is not accepted"),
            FailureCategory::AuthenticationFailure
        );
        assert_eq!(
            category("550 5.7.1 Message rejected"),
            FailureCategory::PolicyRejection
        );
        assert_eq!(
            category("550 5.7.1 Message rejected as spam"),
            FailureCategory::ContentRejected
        );
        assert_eq!(
            category("550 5.1.1 Message rejected"),
            FailureCategory::InvalidRecipient
        );
    }

    #[test]
    fn generic_smtp_550_is_unknown() {
        assert_eq!(category("550"), FailureCategory::Unknown);
    }
}
