pub mod classify;
pub mod fixtures;
pub mod model;
pub mod parse;
mod provider_payload;
pub mod recommend;
pub mod report;

pub use classify::{classify_failure, infer_bounce_type};
pub use fixtures::{built_in_fixtures, find_built_in_fixture, BuiltInFixture, FixtureExpectation};
pub use model::{
    BounceType, Confidence, ConfidenceLevel, FailureCategory, FailureReport, InputSource,
    ParseInput, ParsedFailure, RecommendedAction, Signal, SignalKind,
};
pub use parse::parse_failure;
pub use recommend::recommend_action;
pub use report::explain;
