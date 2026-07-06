use crate::model::{BounceType, FailureCategory, RecommendedAction};

#[must_use]
pub fn recommend_action(
    category: &FailureCategory,
    _bounce_type: &BounceType,
) -> RecommendedAction {
    match category {
        FailureCategory::InvalidRecipient => RecommendedAction::SuppressRecipient,
        FailureCategory::MailboxFull | FailureCategory::TemporaryFailure => {
            RecommendedAction::RetryLater
        }
        FailureCategory::AuthenticationFailure => RecommendedAction::FixDomainAuthentication,
        FailureCategory::PolicyRejection | FailureCategory::ContentRejected => {
            RecommendedAction::ReviewContent
        }
        FailureCategory::RateLimited => RecommendedAction::ReduceSendingRate,
        FailureCategory::ProviderError => RecommendedAction::InvestigateProvider,
        FailureCategory::Unknown => RecommendedAction::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{BounceType, FailureCategory, RecommendedAction};

    use super::recommend_action;

    #[test]
    fn maps_core_categories_to_actions() {
        assert_eq!(
            recommend_action(&FailureCategory::InvalidRecipient, &BounceType::Hard),
            RecommendedAction::SuppressRecipient
        );
        assert_eq!(
            recommend_action(&FailureCategory::AuthenticationFailure, &BounceType::Hard),
            RecommendedAction::FixDomainAuthentication
        );
        assert_eq!(
            recommend_action(&FailureCategory::RateLimited, &BounceType::Soft),
            RecommendedAction::ReduceSendingRate
        );
        assert_eq!(
            recommend_action(&FailureCategory::Unknown, &BounceType::Unknown),
            RecommendedAction::Unknown
        );
    }
}
