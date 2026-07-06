# Contributing

Thanks for helping improve Email Failure Lab.

This project is early, so the best contributions are small, tested, and focused on making `email-lab explain` more useful and reliable.

## Project scope

The v0.1 scope is intentionally narrow:

- parse SMTP errors and bounce-like text
- classify deterministic failure categories
- produce polished text output
- produce stable JSON output
- maintain realistic fixtures and tests

Please do not add DNS checks, webhook simulation, provider API integrations, Node bindings, web apps, telemetry, databases, or full MIME parsing in v0.1 unless an issue explicitly accepts that scope.

## Development setup

Install Rust stable, then run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

On Windows, the MSVC toolchain requires Visual Studio Build Tools with the C++ linker installed. If that is not available, the GNU toolchain is also supported for local validation:

```bash
rustup toolchain install stable-x86_64-pc-windows-gnu
cargo +stable-x86_64-pc-windows-gnu test --workspace
```

## Adding classification rules

When adding or changing a rule:

- keep the rule deterministic
- prefer specific phrases over broad substring matching
- avoid matching negated phrases such as `not blocked`
- add or update fixtures in `crates/email-failure-core/fixtures/cases.json`
- add unit tests when the behavior is subtle
- keep JSON enum names stable unless a breaking change is intentional

## Pull request checklist

Before opening a PR:

- run `cargo fmt --check`
- run `cargo clippy --workspace --all-targets -- -D warnings`
- run `cargo test --workspace`
- update README/docs when behavior changes
- add fixtures for new failure patterns
- keep the PR focused on one behavior or milestone

## Commit and PR style

Use clear, product-facing names. Good examples:

- `Add mailbox full fixture coverage`
- `Improve policy rejection classification`
- `Document failure categories`

Avoid names that describe the tool or process used to make the change.

## Code of conduct

Be direct, kind, and specific. Assume good intent, explain tradeoffs clearly, and keep discussions focused on improving the project.

