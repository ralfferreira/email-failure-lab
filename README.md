# Email Failure Lab

[![CI](https://github.com/ralfferreira/email-failure-lab/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/ralfferreira/email-failure-lab/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

From cryptic email failures to actionable fixes in seconds.

Email Failure Lab is a Rust-powered CLI and core library for developers who need to understand why transactional emails failed and what their application should do next.

The project turns SMTP errors and bounce-like text into a structured report with a failure category, bounce type, confidence level, recommended app behavior, and the exact signals that drove the decision.

The first milestone focuses on one polished command: `email-lab explain`.

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

## What it is

Email Failure Lab is not an email provider, dashboard, or deliverability black box. It is a developer tool for debugging transactional email failures locally and deterministically.

It is designed around a small pure Rust core and a CLI boundary:

- The core library parses signals and builds stable reports.
- The CLI handles arguments, file input, and output formatting.
- The JSON output is intended for automation and future integrations.

## Project status

Email Failure Lab has a focused v0.1 text-classification foundation. The v0.2 explain path adds local, deterministic normalization for supported Resend-style `email.bounced` and `email.failed` JSON payloads while preserving the existing report schema. Webhook simulation, provider API access, DNS checks, Node bindings, and web demos remain future milestones.

## Quickstart

Clone the repository and run from source:

```bash
cargo run -p email-failure-cli -- explain "550 5.1.1 User unknown"
```

Explain a plain text file:

```bash
cargo run -p email-failure-cli -- explain ./crates/email-failure-core/fixtures/raw/invalid-recipient.txt
```

Explain a sanitized Resend-style webhook payload:

```bash
cargo run -p email-failure-cli -- explain ./crates/email-failure-core/fixtures/providers/resend/email-bounced-invalid-recipient.json --json
```

Provider JSON support is intentionally narrow: v0.2 recognizes the documented `email.bounced` and `email.failed` failure fields, ignores unrelated metadata, and makes no API calls. Valid but unsupported JSON returns an `unknown` report; malformed JSON continues through the existing plain-text classifier.

Pipe one-line or multiline failure text from stdin:

```bash
echo "550 5.1.1 User unknown" | cargo run -p email-failure-cli -- explain -
```

```bash
printf "550\n5.1.1\nUser unknown\n" | cargo run -p email-failure-cli -- explain -
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

The JSON output contract is documented in [schemas/failure-report.v0.1.json](schemas/failure-report.v0.1.json).

Multiline input is normalized as plain text before classification, so signals can appear on different lines. Full `.eml`, MIME, attachment, and DSN parsing remain out of scope for v0.1; `.eml` files are treated as plain UTF-8 text.

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

See [docs/failure-categories.md](docs/failure-categories.md) for category behavior, confidence scoring, and recommended app handling.

## Architecture

The workspace has two crates:

- `email-failure-core`: pure deterministic rules. It performs no file I/O, terminal output, network access, environment access, or CLI-specific work.
- `email-failure-cli`: effect boundary. It parses arguments, reads files, formats output, and maps errors to exit codes.

The explain pipeline is:

```txt
input -> normalize supported provider JSON -> parse signals -> classify category -> infer bounce type -> recommend action -> build report
```

## Repository layout

```txt
crates/
  email-failure-core/  # pure parsing, classification, recommendations, reports
  email-failure-cli/   # CLI args, file input, text/JSON output
docs/
  failure-categories.md
schemas/
  failure-report.v0.1.json
```

## Development

Email Failure Lab currently supports Rust 1.85 or newer. The repository includes `rust-toolchain.toml` so local development and CI use the same baseline toolchain.

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

On Windows, the default MSVC toolchain requires Visual Studio Build Tools with the C++ linker installed. The project also validates with the GNU Rust toolchain:

```bash
cargo +1.85.0-x86_64-pc-windows-gnu test --workspace
```

## Contributing

Contributions are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md) for setup, scope, testing expectations, and PR guidelines.

## v0.1 non-goals

Email Failure Lab v0.1 does not include DNS Doctor, webhook simulation, provider API integration, Node bindings, TypeScript packages, Next.js examples, telemetry, databases, or full `.eml`/MIME parsing.

## License

Email Failure Lab is licensed under either of:

- Apache License, Version 2.0
- MIT license

at your option.
