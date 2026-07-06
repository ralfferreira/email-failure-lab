# PLAN.md - Email Failure Lab v0.1 Technical Plan

## 1. Project Goal

Build Email Failure Lab as a small, polished Rust-powered CLI and core library that turns cryptic transactional email failures into actionable explanations.

The first milestone is `email-lab explain`: given an SMTP error, bounce-like string, or simple text file, it should produce a deterministic report with category, bounce type, recommended action, confidence, explanation, and matched signals.

Positioning:

```txt
From cryptic email failures to actionable fixes in seconds.
```

## 2. v0.1 Scope

- Create a Rust workspace with a pure core crate and a CLI crate.
- Support `email-lab explain` for inline strings and plain text files.
- Treat `.eml` files as plain UTF-8 text only.
- Parse SMTP status codes, enhanced status codes, and known failure phrases.
- Classify only these implemented and tested categories:
  - `invalid_recipient`
  - `mailbox_full`
  - `authentication_failure`
  - `policy_rejection`
  - `rate_limited`
  - `temporary_failure`
  - `content_rejected`
  - `provider_error`
  - `unknown`
- Output polished text by default and stable JSON with `--json`.
- Include fixtures, unit tests, CLI integration tests, README examples, and failure-category docs.

## 3. Explicit Non-Goals For v0.1

- No DNS Doctor.
- No webhook simulator.
- No Node bindings, TypeScript package, Next.js demo, or web playground.
- No real provider API integration.
- No MIME parsing, attachment parsing, DSN parsing, or full `.eml` parsing.
- No spam complaint category.
- No domain misconfiguration category.
- No provider event categories or provider reason signals.
- No LLM-based classification.
- No database, telemetry, background services, or persistent storage.

## 4. Proposed Repository Structure

The repository is currently a fresh Git repo with `.gitignore` and `email-failure-lab-context.md`, so v0.1 introduces a clean Rust workspace:

```txt
email-failure-lab/
  Cargo.toml
  README.md
  LICENSE
  PLAN.md
  crates/
    email-failure-core/
      Cargo.toml
      src/
        lib.rs
        model.rs
        parse.rs
        classify.rs
        recommend.rs
        report.rs
      fixtures/
        cases.json
        raw/
          invalid-recipient.txt
          mailbox-full.txt
          auth-failure.txt
    email-failure-cli/
      Cargo.toml
      src/
        main.rs
        commands/
          mod.rs
          explain.rs
        output.rs
        error.rs
  docs/
    failure-categories.md
```

Future DNS, provider, scenario, and Node modules should be added later as separate crates/packages.

## 5. Rust Crate Boundaries

- `email-failure-core`: pure deterministic library. It accepts already-loaded text, parses signals, classifies failures, recommends actions, and returns `FailureReport`. It must not read files, inspect terminals, print output, access environment variables, call the network, or depend on CLI details.
- `email-failure-cli`: effect boundary. It parses CLI args, decides whether input is inline text or a file path, reads files, calls the core library, formats text/JSON, and maps errors to exit codes.

Minimal public core API:

```rust
pub fn explain(input: ParseInput<'_>) -> FailureReport;
pub fn parse_failure(input: ParseInput<'_>) -> ParsedFailure;
pub fn classify_failure(parsed: &ParsedFailure) -> FailureCategory;
pub fn infer_bounce_type(parsed: &ParsedFailure, category: &FailureCategory) -> BounceType;
pub fn recommend_action(category: &FailureCategory, bounce_type: &BounceType) -> RecommendedAction;
```

## 6. Core Domain Model

Use explicit enums with stable serialized names. Use snake_case enum values and camelCase report fields.

Do not add `SpamComplaint`, `DomainMisconfigured`, provider event signals, or provider reason signals in v0.1.

## 7. Classification Pipeline

Implement v0.1 as deterministic rules:

```txt
input text
-> normalize text
-> parse signals
-> classify category
-> infer bounce type
-> recommend action
-> calculate confidence
-> build report
```

Parsing rules:

- Normalize by lowercasing, trimming, and collapsing whitespace.
- Use Rust-compatible regexes only. The Rust `regex` crate does not support lookaround.
- Extract SMTP codes with tokenization so enhanced status codes like `5.1.1` are not treated as SMTP codes.
- Extract enhanced status codes with a regex for values like `5.1.1`, `4.7.0`, `5.7.26`.
- Match known phrases from a small in-code rule table.
- Specific phrase rules beat generic enhanced-code or SMTP-code rules.

Confidence scoring is deterministic rule strength, not statistical probability:

```txt
smtp_code: +20
enhanced_status_code: +35
strong phrase: +35
weak phrase: +20
category agreement bonus: +10
conflicting signal penalty: -20
cap score between 1 and 99

high: 90-99
medium: 60-89
low: 1-59
```

## 8. CLI Interface

Use `clap` with one v0.1 command:

```bash
email-lab explain "550 5.1.1 User unknown"
email-lab explain ./path/to/file.txt
email-lab explain ./path/to/file.txt --json
```

Path-like input detection:

- Treat input as path-like if it starts with `./`, `../`, `/`, or `~/`.
- Treat input as path-like if it contains `/` or `\`.
- Treat input as path-like if it ends with `.txt`, `.log`, or `.eml`.
- Do not treat dots alone as path-like because enhanced SMTP status codes like `5.1.1` contain dots.

## 9. Text Output Design

Default output should be readable, copyable, and useful without color:

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

## 10. JSON Output Schema

Use camelCase field names and snake_case enum values. `schemaVersion` must be present from v0.1.

## 11. Fixture Strategy

- Store table-driven expected cases in `crates/email-failure-core/fixtures/cases.json`.
- Store raw text files under `crates/email-failure-core/fixtures/raw/` for CLI file-input tests.
- Include at least 20 fixture cases covering common SMTP strings, casing differences, partial signals, ambiguous cases, and unknown input.

## 12. Test Strategy

- Parser unit tests for SMTP codes, enhanced status codes, and phrase matches.
- Classifier unit tests for priority and ambiguous `5.7.1` cases.
- Recommendation tests.
- Confidence scoring tests.
- Fixture table tests from `cases.json`.
- CLI integration tests with `assert_cmd`.
- JSON schema stability tests.
- Golden test for default text output.

Acceptance commands:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## 13. Error Handling Strategy

- Core classification is total for non-empty strings: it returns `FailureReport`, including `Unknown`, instead of failing.
- CLI errors use `Result` and a small `CliError` enum.
- File read errors include the path and a human-readable reason.
- Exit code `0` for successful reports, `1` for usage/input/output errors.

## 14. Documentation/README Plan

README v0.1 includes positioning, source build instructions, quickstart examples, example output, supported categories, non-goals, development commands, and the pure-core/CLI-boundary architecture note.

`docs/failure-categories.md` documents each implemented category, bounce type behavior, and recommended app handling.

## 15. First 10 Implementation Tasks

1. Create the Rust workspace with `email-failure-core` and `email-failure-cli`, root `Cargo.toml`, basic README, MIT license, and `PLAN.md`.
2. Add core domain models with serde support and stable enum serialization.
3. Implement input normalization and signal parsing for SMTP codes, enhanced status codes, and phrase matches.
4. Implement deterministic classification, bounce type inference, recommendation mapping, and confidence scoring.
5. Add the `explain` report builder that wires parse, classify, recommend, confidence, explanation, and app guidance together.
6. Add JSON serialization and schema stability tests for `FailureReport`.
7. Implement `email-lab explain <INPUT>` for inline strings with default text output.
8. Add explicit path-like detection, UTF-8 file reading, `--json`, `--format`, `--verbose`, and clear CLI errors.
9. Add fixtures plus unit, fixture, CLI integration, and text golden tests.
10. Polish README examples, add `docs/failure-categories.md`, and ensure all acceptance commands pass.

## 16. Risks And Tradeoffs

- Deterministic phrase rules are simple and testable, but can misclassify ambiguous provider wording. Mitigation: expose matched signals and confidence clearly.
- Path-versus-inline detection can surprise users. Mitigation: use explicit path-like rules and never treat dots alone as path-like.
- Regex mistakes can create false positives. Mitigation: avoid lookaround and ensure `5.1.1` is never parsed as SMTP code.
- Confidence scores can imply more precision than exists. Mitigation: document them as deterministic rule strength, not probability.

## 17. Future Milestones

- v0.2: Provider payloads and fixture library.
- v0.3: Webhook Scenario Simulator.
- v0.4: DNS Doctor.
- v0.5: Node package and Next.js/Resend examples.

## 18. Open Questions

None blocking for v0.1.
