# Roadmap

Email Failure Lab evolves in small, deterministic milestones. Version labels describe product direction, not delivery dates. Scope may change through focused issues and accepted RFCs.

## v0.1 - Classification foundation

**Status:** Implemented on `main`. No Git tag or GitHub release has been published yet.

- Pure Rust core library and the `email-lab` CLI.
- Input from inline text, UTF-8 files, and stdin.
- Deterministic classification from SMTP codes, enhanced status codes, and known phrases.
- Readable text output and stable `FailureReport` JSON v0.1.
- Fixture, schema, CLI, regression, and benchmark coverage.
- Diagnostic rule IDs in verbose text without changing the JSON v0.1 contract.

## v0.2 - Provider payloads and fixture library

**Status:** Current. The core implementation is on `main`; provider documentation and release preparation continue.

Delivered:

- Local normalization of supported Resend-style `email.bounced` and `email.failed` payloads.
- Sanitized provider fixtures for common failure paths.
- `fixtures list` and `fixtures show` commands for built-in examples.
- The existing deterministic classifier and JSON v0.1 contract remain the source of truth.
- No network calls, provider credentials, webhook verification, or event storage.

Next:

- Add practical provider and Resend integration guidance in [#24](https://github.com/ralfferreira/email-failure-lab/issues/24).
- Expand app-handling examples for each category in [#25](https://github.com/ralfferreira/email-failure-lab/issues/25).
- Treat additional providers and event types as separately accepted scope.

## v0.3 - Webhook Scenario Simulator

**Status:** Planned. Design comes first in [#21](https://github.com/ralfferreira/email-failure-lab/issues/21).

- Define the smallest useful scenario set.
- Model duplicate and out-of-order events.
- Specify the CLI surface and safe local delivery behavior.
- Begin implementation only after the RFC fixes scope and non-goals.

## v0.4 - DNS Doctor

**Status:** Planned. Design comes first in [#22](https://github.com/ralfferreira/email-failure-lab/issues/22).

- Define initial SPF, DKIM, and DMARC checks.
- Model findings and actionable fix plans.
- Specify timeout, network failure, and provider-specific behavior before implementation.

## v0.5 - Node and application examples

**Status:** Directional.

- Explore a Node package or bindings for the deterministic core.
- Add Next.js and Resend-oriented application examples.
- Define the package boundary and public API in a proposal before implementation.

## Exploratory work

These directions are useful, but are not assigned to a numbered release:

- An agent-friendly MCP wrapper in [#28](https://github.com/ralfferreira/email-failure-lab/issues/28).
- A lightweight static documentation site in [#29](https://github.com/ralfferreira/email-failure-lab/issues/29).

## Scope boundaries

Unless a future RFC explicitly changes direction, Email Failure Lab will not include:

- Email sending, provider API authentication, or provider account management.
- A hosted dashboard, persistent event store, database, telemetry, or background service.
- LLM-based classification; deterministic behavior remains the source of truth.
- Full MIME, attachment, or DSN parsing; `.eml` remains plain UTF-8 input.
- Provider-specific public categories that destabilize `FailureReport` JSON v0.1.
