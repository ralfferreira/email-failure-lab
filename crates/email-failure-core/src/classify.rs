use std::collections::HashMap;

use crate::model::{BounceType, FailureCategory, ParsedFailure, SignalKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhraseRule {
    pub id: &'static str,
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

    const fn strong(
        id: &'static str,
        phrase: &'static str,
        category: FailureCategory,
        priority: u8,
    ) -> Self {
        Self {
            id,
            phrase,
            category,
            strong: true,
            priority,
        }
    }

    const fn weak(
        id: &'static str,
        phrase: &'static str,
        category: FailureCategory,
        priority: u8,
    ) -> Self {
        Self {
            id,
            phrase,
            category,
            strong: false,
            priority,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EnhancedStatusRule {
    id: &'static str,
    code: &'static str,
    category: FailureCategory,
}

impl EnhancedStatusRule {
    fn all() -> &'static [Self] {
        ENHANCED_STATUS_RULES
    }

    const fn new(id: &'static str, code: &'static str, category: FailureCategory) -> Self {
        Self { id, code, category }
    }
}

static PHRASE_RULES: &[PhraseRule] = &[
    PhraseRule::strong(
        "phrase.content_rejected.message_rejected_as_spam",
        "message rejected as spam",
        FailureCategory::ContentRejected,
        100,
    ),
    PhraseRule::strong(
        "phrase.content_rejected.classified_as_spam",
        "classified as spam",
        FailureCategory::ContentRejected,
        100,
    ),
    PhraseRule::strong(
        "phrase.content_rejected.content_rejected",
        "content rejected",
        FailureCategory::ContentRejected,
        100,
    ),
    PhraseRule::strong(
        "phrase.content_rejected.spam_content",
        "spam content",
        FailureCategory::ContentRejected,
        100,
    ),
    PhraseRule::strong(
        "phrase.content_rejected.spam_detected",
        "spam detected",
        FailureCategory::ContentRejected,
        100,
    ),
    PhraseRule::strong(
        "phrase.content_rejected.identified_as_spam",
        "identified as spam",
        FailureCategory::ContentRejected,
        100,
    ),
    PhraseRule::strong(
        "phrase.authentication_failure.unauthenticated_email",
        "unauthenticated email",
        FailureCategory::AuthenticationFailure,
        95,
    ),
    PhraseRule::strong(
        "phrase.authentication_failure.this_mail_is_unauthenticated",
        "this mail is unauthenticated",
        FailureCategory::AuthenticationFailure,
        95,
    ),
    PhraseRule::strong(
        "phrase.authentication_failure.spf_fail",
        "spf fail",
        FailureCategory::AuthenticationFailure,
        95,
    ),
    PhraseRule::strong(
        "phrase.authentication_failure.dkim_fail",
        "dkim fail",
        FailureCategory::AuthenticationFailure,
        95,
    ),
    PhraseRule::strong(
        "phrase.authentication_failure.dmarc_fail",
        "dmarc fail",
        FailureCategory::AuthenticationFailure,
        95,
    ),
    PhraseRule::strong(
        "phrase.invalid_recipient.user_unknown",
        "user unknown",
        FailureCategory::InvalidRecipient,
        90,
    ),
    PhraseRule::strong(
        "phrase.invalid_recipient.recipient_address_rejected",
        "recipient address rejected",
        FailureCategory::InvalidRecipient,
        90,
    ),
    PhraseRule::strong(
        "phrase.invalid_recipient.mailbox_unavailable",
        "mailbox unavailable",
        FailureCategory::InvalidRecipient,
        90,
    ),
    PhraseRule::strong(
        "phrase.invalid_recipient.mailbox_disabled",
        "mailbox disabled",
        FailureCategory::InvalidRecipient,
        90,
    ),
    PhraseRule::strong(
        "phrase.invalid_recipient.no_such_user",
        "no such user",
        FailureCategory::InvalidRecipient,
        90,
    ),
    PhraseRule::strong(
        "phrase.mailbox_full.mailbox_full",
        "mailbox full",
        FailureCategory::MailboxFull,
        90,
    ),
    PhraseRule::strong(
        "phrase.mailbox_full.quota_exceeded",
        "quota exceeded",
        FailureCategory::MailboxFull,
        90,
    ),
    PhraseRule::strong(
        "phrase.mailbox_full.over_quota",
        "over quota",
        FailureCategory::MailboxFull,
        90,
    ),
    PhraseRule::strong(
        "phrase.rate_limited.rate_limited",
        "rate limited",
        FailureCategory::RateLimited,
        90,
    ),
    PhraseRule::strong(
        "phrase.rate_limited.rate_limit_exceeded",
        "rate limit exceeded",
        FailureCategory::RateLimited,
        90,
    ),
    PhraseRule::strong(
        "phrase.rate_limited.too_many_messages",
        "too many messages",
        FailureCategory::RateLimited,
        90,
    ),
    PhraseRule::strong(
        "phrase.rate_limited.throttled",
        "throttled",
        FailureCategory::RateLimited,
        90,
    ),
    PhraseRule::strong(
        "phrase.temporary_failure.temporary_failure",
        "temporary failure",
        FailureCategory::TemporaryFailure,
        80,
    ),
    PhraseRule::strong(
        "phrase.temporary_failure.temporary_system_problem",
        "temporary system problem",
        FailureCategory::TemporaryFailure,
        80,
    ),
    PhraseRule::strong(
        "phrase.temporary_failure.temporarily_unavailable",
        "temporarily unavailable",
        FailureCategory::TemporaryFailure,
        80,
    ),
    PhraseRule::strong(
        "phrase.temporary_failure.temporarily_deferred",
        "temporarily deferred",
        FailureCategory::TemporaryFailure,
        80,
    ),
    PhraseRule::strong(
        "phrase.temporary_failure.try_again_later",
        "try again later",
        FailureCategory::TemporaryFailure,
        80,
    ),
    PhraseRule::strong(
        "phrase.provider_error.provider_error",
        "provider error",
        FailureCategory::ProviderError,
        80,
    ),
    PhraseRule::strong(
        "phrase.provider_error.internal_error",
        "internal error",
        FailureCategory::ProviderError,
        80,
    ),
    PhraseRule::strong(
        "phrase.provider_error.upstream_error",
        "upstream error",
        FailureCategory::ProviderError,
        80,
    ),
    PhraseRule::weak(
        "phrase.policy_rejection.rejected_by_policy",
        "rejected by policy",
        FailureCategory::PolicyRejection,
        70,
    ),
    PhraseRule::weak(
        "phrase.policy_rejection.access_denied",
        "access denied",
        FailureCategory::PolicyRejection,
        70,
    ),
    PhraseRule::weak(
        "phrase.policy_rejection.block_list",
        "block list",
        FailureCategory::PolicyRejection,
        70,
    ),
    PhraseRule::weak(
        "phrase.policy_rejection.not_accepted",
        "not accepted",
        FailureCategory::PolicyRejection,
        60,
    ),
    PhraseRule::weak(
        "phrase.policy_rejection.blocked",
        "blocked",
        FailureCategory::PolicyRejection,
        60,
    ),
    PhraseRule::weak(
        "phrase.policy_rejection.message_rejected",
        "message rejected",
        FailureCategory::PolicyRejection,
        50,
    ),
];

static ENHANCED_STATUS_RULES: &[EnhancedStatusRule] = &[
    EnhancedStatusRule::new(
        "enhanced_status.invalid_recipient.5_1_1",
        "5.1.1",
        FailureCategory::InvalidRecipient,
    ),
    EnhancedStatusRule::new(
        "enhanced_status.invalid_recipient.5_2_1",
        "5.2.1",
        FailureCategory::InvalidRecipient,
    ),
    EnhancedStatusRule::new(
        "enhanced_status.mailbox_full.5_2_2",
        "5.2.2",
        FailureCategory::MailboxFull,
    ),
    EnhancedStatusRule::new(
        "enhanced_status.policy_rejection.5_7_1",
        "5.7.1",
        FailureCategory::PolicyRejection,
    ),
    EnhancedStatusRule::new(
        "enhanced_status.authentication_failure.5_7_26",
        "5.7.26",
        FailureCategory::AuthenticationFailure,
    ),
    EnhancedStatusRule::new(
        "enhanced_status.temporary_failure.4_7_0",
        "4.7.0",
        FailureCategory::TemporaryFailure,
    ),
    EnhancedStatusRule::new(
        "enhanced_status.temporary_failure.4_3_0",
        "4.3.0",
        FailureCategory::TemporaryFailure,
    ),
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
    enhanced_status_rule_for(code).map(|rule| rule.category)
}

#[must_use]
pub fn category_for_phrase(phrase: &str) -> Option<FailureCategory> {
    phrase_rule_for(phrase).map(|rule| rule.category)
}

#[must_use]
pub fn rule_id_for_signal(kind: SignalKind, value: &str) -> Option<&'static str> {
    match kind {
        SignalKind::SmtpCode => None,
        SignalKind::EnhancedStatusCode => enhanced_status_rule_for(value).map(|rule| rule.id),
        SignalKind::MatchedPhrase => phrase_rule_for(value).map(|rule| rule.id),
    }
}

fn phrase_rule_for(phrase: &str) -> Option<&'static PhraseRule> {
    PhraseRule::all().iter().find(|rule| rule.phrase == phrase)
}

fn enhanced_status_rule_for(code: &str) -> Option<&'static EnhancedStatusRule> {
    EnhancedStatusRule::all()
        .iter()
        .find(|rule| rule.code == code)
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
        SignalKind::MatchedPhrase => phrase_rule_for(value).map_or(40, |rule| {
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
        .filter_map(|signal| phrase_rule_for(&signal.value).filter(|rule| rule.strong))
        .max_by_key(|rule| rule.priority)
        .map(|rule| rule.category)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::model::{FailureCategory, InputSource, ParseInput, SignalKind};
    use crate::parse::parse_failure;

    use super::{classify_failure, rule_id_for_signal, EnhancedStatusRule, PhraseRule};

    fn category(raw: &str) -> FailureCategory {
        let parsed = parse_failure(ParseInput {
            raw,
            source: InputSource::Inline,
        });
        classify_failure(&parsed)
    }

    fn assert_rule_ids<'a>(namespace: &str, ids: impl IntoIterator<Item = &'a str>) {
        let mut unique_ids = HashSet::new();

        for id in ids {
            assert!(!id.is_empty(), "rule ID must not be empty");
            assert!(
                id.starts_with(namespace),
                "unexpected rule ID namespace: {id}"
            );
            assert!(unique_ids.insert(id), "duplicate rule ID: {id}");
        }
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

    #[test]
    fn phrase_rule_ids_are_non_empty_unique_and_namespaced() {
        assert_rule_ids("phrase.", PhraseRule::all().iter().map(|rule| rule.id));
    }

    #[test]
    fn enhanced_status_rule_ids_are_non_empty_unique_and_namespaced() {
        assert_rule_ids(
            "enhanced_status.",
            EnhancedStatusRule::all().iter().map(|rule| rule.id),
        );
    }

    #[test]
    fn representative_signal_values_resolve_to_stable_rule_ids() {
        assert_eq!(
            rule_id_for_signal(SignalKind::MatchedPhrase, "user unknown"),
            Some("phrase.invalid_recipient.user_unknown")
        );
        assert_eq!(
            rule_id_for_signal(SignalKind::EnhancedStatusCode, "5.7.26"),
            Some("enhanced_status.authentication_failure.5_7_26")
        );
        assert_eq!(rule_id_for_signal(SignalKind::SmtpCode, "550"), None);
    }
}
