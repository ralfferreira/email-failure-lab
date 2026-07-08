use crate::classify::PhraseRule;
use crate::model::{ParseInput, ParsedFailure, Signal, SignalKind};

pub const SMTP_CODE_WEIGHT: u8 = 20;
pub const ENHANCED_STATUS_CODE_WEIGHT: u8 = 35;
pub const STRONG_PHRASE_WEIGHT: u8 = 35;
pub const WEAK_PHRASE_WEIGHT: u8 = 20;

pub fn parse_failure(input: ParseInput<'_>) -> ParsedFailure {
    let normalized_text = normalize_text(input.raw);
    let mut signals = Vec::new();

    signals.extend(parse_smtp_codes(&normalized_text));
    signals.extend(parse_enhanced_status_codes(&normalized_text));
    signals.extend(parse_phrase_signals(&normalized_text));

    ParsedFailure {
        normalized_text,
        signals,
    }
}

pub fn normalize_text(input: &str) -> String {
    input
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_smtp_codes(input: &str) -> Vec<Signal> {
    input
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '.')
        .filter_map(|token| {
            let token = token.trim_matches('.');

            if token.len() == 3
                && token.starts_with(['4', '5'])
                && token.chars().all(|character| character.is_ascii_digit())
            {
                Some(Signal {
                    kind: SignalKind::SmtpCode,
                    value: token.to_owned(),
                    weight: SMTP_CODE_WEIGHT,
                })
            } else {
                None
            }
        })
        .collect()
}

fn parse_enhanced_status_codes(input: &str) -> Vec<Signal> {
    input
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '.')
        .filter_map(|token| {
            let token = token.trim_matches('.');

            if is_enhanced_status_code(token) {
                Some(Signal {
                    kind: SignalKind::EnhancedStatusCode,
                    value: token.to_owned(),
                    weight: ENHANCED_STATUS_CODE_WEIGHT,
                })
            } else {
                None
            }
        })
        .collect()
}

fn is_enhanced_status_code(token: &str) -> bool {
    let parts = token.split('.').collect::<Vec<_>>();

    if parts.len() != 3 {
        return false;
    }

    matches!(parts[0], "2" | "4" | "5")
        && parts[1].len() <= 3
        && parts[2].len() <= 3
        && parts[1].chars().all(|character| character.is_ascii_digit())
        && parts[2].chars().all(|character| character.is_ascii_digit())
}

fn parse_phrase_signals(input: &str) -> Vec<Signal> {
    PhraseRule::all()
        .iter()
        .filter(|rule| phrase_matches(input, rule.phrase))
        .map(|rule| Signal {
            kind: SignalKind::MatchedPhrase,
            value: rule.phrase.to_owned(),
            weight: if rule.strong {
                STRONG_PHRASE_WEIGHT
            } else {
                WEAK_PHRASE_WEIGHT
            },
        })
        .collect()
}

fn phrase_matches(input: &str, phrase: &str) -> bool {
    let mut search_start = 0;

    while let Some(relative_start) = input[search_start..].find(phrase) {
        let start = search_start + relative_start;
        let end = start + phrase.len();

        if has_phrase_boundaries(input, start, end) && !is_negated(input, start) {
            return true;
        }

        search_start = end;
    }

    false
}

fn has_phrase_boundaries(input: &str, start: usize, end: usize) -> bool {
    let starts_cleanly = input[..start]
        .chars()
        .next_back()
        .is_none_or(|character| !character.is_ascii_alphanumeric());
    let ends_cleanly = input[end..]
        .chars()
        .next()
        .is_none_or(|character| !character.is_ascii_alphanumeric());

    starts_cleanly && ends_cleanly
}

fn is_negated(input: &str, phrase_start: usize) -> bool {
    input[..phrase_start]
        .split_whitespace()
        .last()
        .is_some_and(|word| matches!(word, "not" | "never"))
}

#[cfg(test)]
mod tests {
    use crate::model::{InputSource, ParseInput, SignalKind};

    use super::{normalize_text, parse_failure};

    fn parse(raw: &str) -> Vec<(SignalKind, String)> {
        parse_failure(ParseInput {
            raw,
            source: InputSource::Inline,
        })
        .signals
        .into_iter()
        .map(|signal| (signal.kind, signal.value))
        .collect()
    }

    #[test]
    fn normalizes_whitespace_and_case() {
        assert_eq!(normalize_text("  USER \n\t Unknown  "), "user unknown");
    }

    #[test]
    fn matches_phrases_across_normalized_line_breaks() {
        let signals = parse("Remote said:\nUser\nunknown");

        assert!(signals.contains(&(SignalKind::MatchedPhrase, "user unknown".to_owned())));
    }

    #[test]
    fn parses_smtp_code_without_parsing_enhanced_status_as_smtp() {
        let signals = parse("550 5.1.1 User unknown");

        assert!(signals.contains(&(SignalKind::SmtpCode, "550".to_owned())));
        assert!(signals.contains(&(SignalKind::EnhancedStatusCode, "5.1.1".to_owned())));
        assert!(!signals.contains(&(SignalKind::SmtpCode, "5.1".to_owned())));
        assert!(!signals.contains(&(SignalKind::SmtpCode, "511".to_owned())));
    }

    #[test]
    fn parses_enhanced_status_without_lookaround() {
        let signals = parse("status=5.7.26 unauthenticated email");

        assert!(signals.contains(&(SignalKind::EnhancedStatusCode, "5.7.26".to_owned())));
    }

    #[test]
    fn parses_adjacent_enhanced_status_codes() {
        let signals = parse("5.1.1 5.2.2");

        assert!(signals.contains(&(SignalKind::EnhancedStatusCode, "5.1.1".to_owned())));
        assert!(signals.contains(&(SignalKind::EnhancedStatusCode, "5.2.2".to_owned())));
    }

    #[test]
    fn does_not_treat_date_or_id_as_smtp_code() {
        let signals = parse("event 2026-07-02 id abc550xyz");

        assert!(!signals
            .iter()
            .any(|(kind, value)| *kind == SignalKind::SmtpCode && value == "550"));
    }

    #[test]
    fn matches_specific_spam_phrases_only() {
        let not_spam = parse("this message is not spam policy information");
        let rejected_as_spam = parse("Message rejected as spam");

        assert!(!not_spam
            .iter()
            .any(|(kind, value)| *kind == SignalKind::MatchedPhrase && value.contains("spam")));
        assert!(rejected_as_spam.contains(&(
            SignalKind::MatchedPhrase,
            "message rejected as spam".to_owned()
        )));
    }

    #[test]
    fn does_not_match_negated_weak_phrases() {
        let signals = parse("this message is not blocked");

        assert!(!signals
            .iter()
            .any(|(kind, value)| *kind == SignalKind::MatchedPhrase && value == "blocked"));
    }

    #[test]
    fn matches_phrase_boundaries() {
        let signals = parse("this message was unblocked by policy");

        assert!(!signals
            .iter()
            .any(|(kind, value)| *kind == SignalKind::MatchedPhrase && value == "blocked"));
    }
}
