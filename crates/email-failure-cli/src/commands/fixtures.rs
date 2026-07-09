use clap::{Args, Subcommand};
use email_failure_core::{
    built_in_fixtures, find_built_in_fixture, BounceType, ConfidenceLevel, FailureCategory,
    RecommendedAction,
};

use crate::error::CliError;

#[derive(Debug, Args)]
pub struct FixturesArgs {
    #[command(subcommand)]
    command: FixturesCommand,
}

#[derive(Debug, Subcommand)]
enum FixturesCommand {
    /// List the available built-in fixtures.
    List,
    /// Show a fixture's input and expected classification.
    Show(ShowFixtureArgs),
}

#[derive(Debug, Args)]
struct ShowFixtureArgs {
    /// Fixture name reported by `email-lab fixtures list`.
    name: String,
}

pub fn run_fixtures(args: FixturesArgs) -> Result<(), CliError> {
    match args.command {
        FixturesCommand::List => {
            print_fixture_list();
            Ok(())
        }
        FixturesCommand::Show(args) => show_fixture(args.name),
    }
}

fn print_fixture_list() {
    let mut fixtures = built_in_fixtures().iter().collect::<Vec<_>>();
    fixtures.sort_unstable_by_key(|fixture| fixture.name);

    let name_width = fixtures
        .iter()
        .map(|fixture| fixture.name.len())
        .max()
        .unwrap_or_default();
    let mut output = format!("Built-in fixtures ({}):\n", fixtures.len());

    for fixture in fixtures {
        output.push_str(&format!(
            "  {:name_width$}  {}\n",
            fixture.name,
            category_name(fixture.expected.category)
        ));
    }

    output.push_str("\nUse 'email-lab fixtures show <name>' to inspect a fixture.");
    println!("{output}");
}

fn show_fixture(name: String) -> Result<(), CliError> {
    let Some(fixture) = find_built_in_fixture(&name) else {
        return Err(CliError::UnknownFixture(name));
    };

    let mut output = format!("Fixture: {}\n\nInput:\n", fixture.name);
    output.push_str(fixture.input);
    if !fixture.input.ends_with('\n') {
        output.push('\n');
    }
    output.push_str("\nExpected classification:\n");
    output.push_str(&format!(
        "  category: {}\n",
        category_name(fixture.expected.category)
    ));
    output.push_str(&format!(
        "  bounce_type: {}\n",
        bounce_type_name(fixture.expected.bounce_type)
    ));
    output.push_str(&format!(
        "  recommended_action: {}\n",
        action_name(fixture.expected.recommended_action)
    ));
    output.push_str(&format!(
        "  confidence_level: {}",
        confidence_level_name(fixture.expected.confidence_level)
    ));

    println!("{output}");
    Ok(())
}

fn category_name(category: FailureCategory) -> &'static str {
    match category {
        FailureCategory::InvalidRecipient => "invalid_recipient",
        FailureCategory::MailboxFull => "mailbox_full",
        FailureCategory::AuthenticationFailure => "authentication_failure",
        FailureCategory::PolicyRejection => "policy_rejection",
        FailureCategory::RateLimited => "rate_limited",
        FailureCategory::TemporaryFailure => "temporary_failure",
        FailureCategory::ContentRejected => "content_rejected",
        FailureCategory::ProviderError => "provider_error",
        FailureCategory::Unknown => "unknown",
    }
}

fn bounce_type_name(bounce_type: BounceType) -> &'static str {
    match bounce_type {
        BounceType::Hard => "hard",
        BounceType::Soft => "soft",
        BounceType::Unknown => "unknown",
    }
}

fn action_name(action: RecommendedAction) -> &'static str {
    match action {
        RecommendedAction::SuppressRecipient => "suppress_recipient",
        RecommendedAction::RetryLater => "retry_later",
        RecommendedAction::FixDomainAuthentication => "fix_domain_authentication",
        RecommendedAction::ReduceSendingRate => "reduce_sending_rate",
        RecommendedAction::ReviewContent => "review_content",
        RecommendedAction::InvestigateProvider => "investigate_provider",
        RecommendedAction::ContactRecipient => "contact_recipient",
        RecommendedAction::NoActionRequired => "no_action_required",
        RecommendedAction::Unknown => "unknown",
    }
}

fn confidence_level_name(level: ConfidenceLevel) -> &'static str {
    match level {
        ConfidenceLevel::Low => "low",
        ConfidenceLevel::Medium => "medium",
        ConfidenceLevel::High => "high",
    }
}
