# Email Failure Lab

From cryptic email failures to actionable fixes in seconds.

Email Failure Lab is a small Rust-powered toolkit for explaining transactional email failures. The v0.1 milestone focuses on one polished command: `email-lab explain`.

```bash
cargo run -p email-failure-cli -- explain "550 5.1.1 User unknown"
```

```txt
Failure: Invalid recipient
Bounce type: Hard bounce
Recommended action: Suppress recipient
Confidence: High (99%)

Why:
The recipient address appears not to exist or cannot receive mail.

What your app should do:
- Stop sending to this address.
- Mark the email as invalid.
- Ask the user to update their email address.

Signals:
- smtp_code: 550
- enhanced_status_code: 5.1.1
- matched_phrase: user unknown
```

## Quickstart

Build and run from source:

```bash
cargo run -p email-failure-cli -- explain "550 5.1.1 User unknown"
```

Explain a plain text file:

```bash
cargo run -p email-failure-cli -- explain ./crates/email-failure-core/fixtures/raw/invalid-recipient.txt
```

Emit stable JSON:

```bash
cargo run -p email-failure-cli -- explain "550 5.1.1 User unknown" --json
```

```json
{
  "schemaVersion": "0.1",
  "category": "invalid_recipient",
  "bounceType": "hard",
  "recommendedAction": "suppress_recipient",
  "confidence": {
    "level": "high",
    "score": 99
  },
  "explanation": "The recipient address appears not to exist or cannot receive mail.",
  "appGuidance": [
    "Stop sending to this address.",
    "Mark the email as invalid.",
    "Ask the user to update their email address."
  ],
  "signals": [
    {
      "kind": "smtp_code",
      "value": "550",
      "weight": 20
    },
    {
      "kind": "enhanced_status_code",
      "value": "5.1.1",
      "weight": 35
    },
    {
      "kind": "matched_phrase",
      "value": "user unknown",
      "weight": 35
    }
  ]
}
```

## Supported v0.1 categories

- `invalid_recipient`
- `mailbox_full`
- `authentication_failure`
- `policy_rejection`
- `rate_limited`
- `temporary_failure`
- `content_rejected`
- `provider_error`
- `unknown`

See [docs/failure-categories.md](docs/failure-categories.md) for category behavior and recommended app handling.

## Architecture

The workspace has two crates:

- `email-failure-core`: pure deterministic rules. It performs no file I/O, terminal output, network access, environment access, or CLI-specific work.
- `email-failure-cli`: effect boundary. It parses arguments, reads files, formats output, and maps errors to exit codes.

The v0.1 pipeline is:

```txt
input -> parse signals -> classify category -> infer bounce type -> recommend action -> build report
```

## Development

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## v0.1 non-goals

Email Failure Lab v0.1 does not include DNS Doctor, webhook simulation, provider API integration, Node bindings, TypeScript packages, Next.js examples, telemetry, databases, or full `.eml`/MIME parsing.
