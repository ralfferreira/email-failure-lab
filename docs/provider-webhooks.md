# Use provider webhooks in your app

Email Failure Lab can classify supported Resend-style failure webhooks from local JSON. This guide shows how to classify a saved payload, record the result, and map `recommendedAction` to an app decision. The workflow runs locally and needs no Resend API token.

## Understand the supported provider boundary

Version 0.2 recognizes the documented Resend-style `email.bounced` and `email.failed` shapes:

- For `email.bounced`, it normalizes `data.bounce.message`, `data.bounce.type`, and `data.bounce.subType`
- For `email.failed`, it normalizes `data.failed.reason`
- It ignores recipient addresses, message identifiers, subjects, tags, and other provider metadata
- It returns an `unknown` report for valid JSON outside the supported shapes

Email Failure Lab does not receive webhooks, verify signatures, call provider APIs, send email, or update suppression lists. Verify production webhook requests against the raw request body before classification, as described in Resend's [webhook verification guide](https://resend.com/docs/webhooks/verify-webhooks-requests). A webhook signing secret is separate from a provider API key. Local classification of saved fixtures needs neither credential.

See Resend's [`email.bounced`](https://resend.com/docs/webhooks/emails/bounced) and [`email.failed`](https://resend.com/docs/webhooks/emails/failed) references for the provider-owned payload contract.

Inspect `category` and `recommendedAction` instead of treating a successful process exit as a recognized failure. Valid but unsupported JSON produces an `unknown` report, while malformed JSON continues through the plain-text classifier. Confidence represents deterministic rule strength, not probability.

## Classify a saved bounce payload

Save this synthetic payload as `resend-bounce.json`:

```json
{
  "type": "email.bounced",
  "created_at": "2026-07-09T12:00:00.000Z",
  "data": {
    "email_id": "00000000-0000-4000-8000-000000000101",
    "from": "Example Sender <sender@example.com>",
    "to": ["invalid-recipient@example.com"],
    "bounce": {
      "message": "550 5.1.1 The email account does not exist.",
      "type": "Permanent",
      "subType": "General"
    }
  }
}
```

Run the classifier from the repository root:

```bash
cargo run --quiet -p email-failure-cli -- \
  explain ./resend-bounce.json --json
```

The report includes `category: "invalid_recipient"` and `recommendedAction: "suppress_recipient"`. You can also run the repository's sanitized fixture from its crate directory:

```bash
cd crates/email-failure-core
cargo run --quiet -p email-failure-cli -- \
  explain fixtures/providers/resend/email-bounced-invalid-recipient.json --json
```

These commands read local files and write the classification to standard output. They do not need network access or provider credentials.

## Write decisions to an automation log

Use `jq` to select stable report fields and append one JSON object per line:

```bash
cargo run --quiet -p email-failure-cli -- \
  explain ./resend-bounce.json --json |
  jq -c \
    '{schemaVersion, category, recommendedAction, confidence}' \
  >> failure-decisions.jsonl
```

PowerShell can produce the same JSON Lines record without `jq`:

```powershell
cargo run --quiet -p email-failure-cli -- `
  explain ./resend-bounce.json --json |
  ConvertFrom-Json |
  Select-Object schemaVersion, category, recommendedAction, confidence |
  ConvertTo-Json -Compress -Depth 3 |
  Add-Content -Path failure-decisions.jsonl
```

The `FailureReport` JSON v0.1 schema is stable for automation. Store the verified webhook identifier separately so retries or replays do not apply the same action twice.

## Map the report to an app decision in TypeScript

Keep provider metadata outside the report and route only on the stable action value:

```typescript
type FailureReport = {
  schemaVersion: "0.1";
  recommendedAction: string;
};

type AppDecision =
  | { type: "suppress"; recipientId: string }
  | { type: "retry"; recipientId: string; delaySeconds: number }
  | { type: "manual_review"; recipientId: string };

export function decide(
  report: FailureReport,
  recipientId: string,
): AppDecision {
  switch (report.recommendedAction) {
    case "suppress_recipient":
      return { type: "suppress", recipientId };
    case "retry_later":
      return { type: "retry", recipientId, delaySeconds: 900 };
    default:
      return { type: "manual_review", recipientId };
  }
}
```

Pass a recipient identifier from your verified webhook record, not from `FailureReport`. Apply the returned decision through your app's suppression, scheduling, or review workflow. Keep retries bounded and make action handling idempotent.

See [Failure categories](failure-categories.md) for the current category, bounce type, and recommended action mappings.

## Classify a payload through the Rust library

Rust applications can call the deterministic core without spawning the command-line interface:

```rust
use email_failure_core::{
    explain, InputSource, ParseInput, RecommendedAction,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "resend-bounce.json";
    let payload = std::fs::read_to_string(path)?;
    let report = explain(ParseInput {
        raw: &payload,
        source: InputSource::File { path: path.into() },
    });

    match report.recommended_action {
        RecommendedAction::SuppressRecipient => println!("suppress"),
        RecommendedAction::RetryLater => println!("retry later"),
        action => println!("manual review: {action:?}"),
    }

    Ok(())
}
```

The core normalizes the supported provider fields, then uses the same deterministic classifier as plain text input. It performs no file input/output or network access itself; the calling application supplies the payload.
