# Changelog

All notable changes to Email Failure Lab will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and releases will follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html). The project has not published its first tagged release, so the current project history is collected under `Unreleased`.

## [Unreleased]

### Added

- A pure Rust classification core and the `email-lab` command-line interface.
- Deterministic failure reports for inline text, UTF-8 files, and stdin.
- Human-readable text output and stable `FailureReport` JSON v0.1.
- Local normalization for supported Resend-style `email.bounced` and `email.failed` payloads.
- Built-in raw and provider fixtures with `fixtures list` and `fixtures show` commands.
- Fixture, schema, CLI, regression, and Criterion benchmark coverage.
- Diagnostic rule identifiers in verbose text output without changing JSON v0.1.
- Contributor guidance, dual MIT/Apache-2.0 licensing, and continuous integration on Linux and Windows.
- A public roadmap, Code of Conduct, security policy, and release checklist.

### Changed

- Expanded deterministic coverage for multiline and provider-style SMTP failures while preserving existing text behavior and the JSON v0.1 schema.
