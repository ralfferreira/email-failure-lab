use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProviderNormalization {
    PlainText,
    UnsupportedJson,
    Normalized(String),
}

pub(crate) fn normalize_provider_payload(raw: &str) -> ProviderNormalization {
    if is_standalone_smtp_code(raw.trim()) {
        return ProviderNormalization::PlainText;
    }

    let payload = match serde_json::from_str::<Value>(raw) {
        Ok(Value::Object(payload)) => payload,
        Ok(_) => return ProviderNormalization::UnsupportedJson,
        Err(_) => return ProviderNormalization::PlainText,
    };

    let Some(event_type) = payload.get("type").and_then(Value::as_str) else {
        return ProviderNormalization::UnsupportedJson;
    };
    let Some(data) = payload.get("data").and_then(Value::as_object) else {
        return ProviderNormalization::UnsupportedJson;
    };

    match event_type {
        "email.bounced" => ProviderNormalization::Normalized(normalize_bounced(data)),
        "email.failed" => ProviderNormalization::Normalized(normalize_failed(data)),
        _ => ProviderNormalization::UnsupportedJson,
    }
}

fn is_standalone_smtp_code(input: &str) -> bool {
    input.len() == 3
        && input.starts_with(['4', '5'])
        && input.chars().all(|character| character.is_ascii_digit())
}

fn normalize_bounced(data: &Map<String, Value>) -> String {
    let Some(bounce) = data.get("bounce").and_then(Value::as_object) else {
        return String::new();
    };

    ["message", "type", "subType"]
        .into_iter()
        .filter_map(|field| bounce.get(field).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalize_failed(data: &Map<String, Value>) -> String {
    let Some(reason) = data
        .get("failed")
        .and_then(Value::as_object)
        .and_then(|failed| failed.get("reason"))
        .and_then(Value::as_str)
    else {
        return String::new();
    };

    let readable_reason = reason
        .trim()
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if reason.trim() == "reached_daily_quota" {
        format!("{readable_reason}: rate limit exceeded")
    } else {
        readable_reason
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_provider_payload, ProviderNormalization};

    #[test]
    fn treats_non_json_and_malformed_json_as_plain_text() {
        assert_eq!(
            normalize_provider_payload("550 5.1.1 User unknown"),
            ProviderNormalization::PlainText
        );
        assert_eq!(
            normalize_provider_payload(r#"{"type":"email.bounced""#),
            ProviderNormalization::PlainText
        );
    }

    #[test]
    fn preserves_standalone_smtp_codes_as_plain_text() {
        for code in ["421", "451", "550"] {
            assert_eq!(
                normalize_provider_payload(code),
                ProviderNormalization::PlainText,
                "{code} should remain on the SMTP text path"
            );
        }
    }

    #[test]
    fn rejects_valid_json_without_the_supported_shape() {
        assert_eq!(
            normalize_provider_payload(r#"["550 5.1.1 User unknown"]"#),
            ProviderNormalization::UnsupportedJson
        );
        assert_eq!(
            normalize_provider_payload(
                r#"{"type":"email.delivered","data":{"subject":"550 user unknown"}}"#
            ),
            ProviderNormalization::UnsupportedJson
        );
        assert_eq!(
            normalize_provider_payload(r#"{"type":"email.bounced","data":null}"#),
            ProviderNormalization::UnsupportedJson
        );
    }

    #[test]
    fn normalizes_bounced_fields_in_a_fixed_order() {
        let payload = r#"{
            "type": "email.bounced",
            "data": {
                "subject": "ignored 421 rate limit exceeded",
                "bounce": {
                    "subType": "General",
                    "type": "Permanent",
                    "message": "550 5.1.1 User unknown"
                }
            }
        }"#;

        assert_eq!(
            normalize_provider_payload(payload),
            ProviderNormalization::Normalized(
                "550 5.1.1 User unknown\nPermanent\nGeneral".to_owned()
            )
        );
    }

    #[test]
    fn normalizes_failed_reason_and_maps_daily_quota() {
        assert_eq!(
            normalize_provider_payload(
                r#"{"type":"email.failed","data":{"failed":{"reason":"internal_error"}}}"#
            ),
            ProviderNormalization::Normalized("internal error".to_owned())
        );
        assert_eq!(
            normalize_provider_payload(
                r#"{"type":"email.failed","data":{"failed":{"reason":"reached_daily_quota"}}}"#
            ),
            ProviderNormalization::Normalized(
                "reached daily quota: rate limit exceeded".to_owned()
            )
        );
    }

    #[test]
    fn supported_payload_without_useful_fields_normalizes_to_empty_text() {
        assert_eq!(
            normalize_provider_payload(
                r#"{"type":"email.bounced","data":{"bounce":{"message":null}}}"#
            ),
            ProviderNormalization::Normalized(String::new())
        );
    }
}
