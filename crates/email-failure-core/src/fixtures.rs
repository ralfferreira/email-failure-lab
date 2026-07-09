use crate::{BounceType, ConfidenceLevel, FailureCategory, RecommendedAction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct BuiltInFixture {
    pub name: &'static str,
    pub input: &'static str,
    pub expected: FixtureExpectation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct FixtureExpectation {
    pub category: FailureCategory,
    pub bounce_type: BounceType,
    pub recommended_action: RecommendedAction,
    pub confidence_level: ConfidenceLevel,
}

const BUILT_IN_FIXTURES: &[BuiltInFixture] = &[
    BuiltInFixture {
        name: "auth-failure",
        input: include_str!("../fixtures/raw/auth-failure.txt"),
        expected: FixtureExpectation {
            category: FailureCategory::AuthenticationFailure,
            bounce_type: BounceType::Hard,
            recommended_action: RecommendedAction::FixDomainAuthentication,
            confidence_level: ConfidenceLevel::High,
        },
    },
    BuiltInFixture {
        name: "invalid-recipient",
        input: include_str!("../fixtures/raw/invalid-recipient.txt"),
        expected: FixtureExpectation {
            category: FailureCategory::InvalidRecipient,
            bounce_type: BounceType::Hard,
            recommended_action: RecommendedAction::SuppressRecipient,
            confidence_level: ConfidenceLevel::High,
        },
    },
    BuiltInFixture {
        name: "mailbox-full",
        input: include_str!("../fixtures/raw/mailbox-full.txt"),
        expected: FixtureExpectation {
            category: FailureCategory::MailboxFull,
            bounce_type: BounceType::Soft,
            recommended_action: RecommendedAction::RetryLater,
            confidence_level: ConfidenceLevel::High,
        },
    },
    BuiltInFixture {
        name: "plain-bounce",
        input: include_str!("../fixtures/raw/plain-bounce.eml"),
        expected: FixtureExpectation {
            category: FailureCategory::InvalidRecipient,
            bounce_type: BounceType::Hard,
            recommended_action: RecommendedAction::SuppressRecipient,
            confidence_level: ConfidenceLevel::High,
        },
    },
    BuiltInFixture {
        name: "resend-authentication-failure",
        input: include_str!(
            "../fixtures/providers/resend/email-bounced-authentication-failure.json"
        ),
        expected: FixtureExpectation {
            category: FailureCategory::AuthenticationFailure,
            bounce_type: BounceType::Hard,
            recommended_action: RecommendedAction::FixDomainAuthentication,
            confidence_level: ConfidenceLevel::High,
        },
    },
    BuiltInFixture {
        name: "resend-daily-quota",
        input: include_str!("../fixtures/providers/resend/email-failed-reached-daily-quota.json"),
        expected: FixtureExpectation {
            category: FailureCategory::RateLimited,
            bounce_type: BounceType::Soft,
            recommended_action: RecommendedAction::ReduceSendingRate,
            confidence_level: ConfidenceLevel::Medium,
        },
    },
    BuiltInFixture {
        name: "resend-invalid-recipient",
        input: include_str!("../fixtures/providers/resend/email-bounced-invalid-recipient.json"),
        expected: FixtureExpectation {
            category: FailureCategory::InvalidRecipient,
            bounce_type: BounceType::Hard,
            recommended_action: RecommendedAction::SuppressRecipient,
            confidence_level: ConfidenceLevel::Medium,
        },
    },
    BuiltInFixture {
        name: "resend-temporary-failure",
        input: include_str!(
            "../fixtures/providers/resend/email-bounced-temporary-failure-resend-like.json"
        ),
        expected: FixtureExpectation {
            category: FailureCategory::TemporaryFailure,
            bounce_type: BounceType::Soft,
            recommended_action: RecommendedAction::RetryLater,
            confidence_level: ConfidenceLevel::High,
        },
    },
];

pub fn built_in_fixtures() -> &'static [BuiltInFixture] {
    BUILT_IN_FIXTURES
}

pub fn find_built_in_fixture(name: &str) -> Option<&'static BuiltInFixture> {
    BUILT_IN_FIXTURES
        .iter()
        .find(|fixture| fixture.name == name)
}
