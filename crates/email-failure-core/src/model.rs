use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureCategory {
    InvalidRecipient,
    MailboxFull,
    AuthenticationFailure,
    PolicyRejection,
    RateLimited,
    TemporaryFailure,
    ContentRejected,
    ProviderError,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BounceType {
    Hard,
    Soft,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendedAction {
    SuppressRecipient,
    RetryLater,
    FixDomainAuthentication,
    ReduceSendingRate,
    ReviewContent,
    InvestigateProvider,
    ContactRecipient,
    NoActionRequired,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Confidence {
    pub level: ConfidenceLevel,
    pub score: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Signal {
    pub kind: SignalKind,
    pub value: String,
    pub weight: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalKind {
    SmtpCode,
    EnhancedStatusCode,
    MatchedPhrase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseInput<'a> {
    pub raw: &'a str,
    pub source: InputSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputSource {
    Inline,
    File { path: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFailure {
    pub normalized_text: String,
    pub signals: Vec<Signal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailureReport {
    pub schema_version: String,
    pub category: FailureCategory,
    pub bounce_type: BounceType,
    pub recommended_action: RecommendedAction,
    pub confidence: Confidence,
    pub explanation: String,
    pub app_guidance: Vec<String>,
    pub signals: Vec<Signal>,
}
