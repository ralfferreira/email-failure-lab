# Contributing

Thanks for helping improve Email Failure Lab.

This project is early, so the best contributions are small, tested, and focused on making `email-lab explain` more useful and reliable.

## Project scope

The current scope is intentionally narrow:

- parse SMTP errors and bounce-like text
- normalize the supported Resend-style provider payload fields locally
- classify deterministic failure categories
- produce polished text output
- produce stable JSON output
- maintain realistic fixtures, tests, and benchmarks

Read [ROADMAP.md](ROADMAP.md) before proposing milestone work. DNS checks, webhook simulation, provider API integrations, Node bindings, web apps, telemetry, databases, full MIME parsing, and other scope expansions require an accepted issue or RFC before implementation.

## Development setup

Install Rust 1.85 or newer. The repository includes `rust-toolchain.toml`, so `cargo` and `rustup` will use the supported baseline toolchain automatically when possible.

Then run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

On Windows, the MSVC toolchain requires Visual Studio Build Tools with the C++ linker installed. If that is not available, the GNU toolchain is also supported for local validation:

```bash
rustup toolchain install 1.85.0-x86_64-pc-windows-gnu
cargo +1.85.0-x86_64-pc-windows-gnu test --workspace --locked
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
- run `cargo clippy --workspace --all-targets --locked -- -D warnings`
- run `cargo test --workspace --locked`
- update README/docs when behavior changes
- add fixtures for new failure patterns
- keep the PR focused on one behavior or milestone

Maintainers preparing a versioned GitHub release should follow the [release checklist](docs/release-checklist.md).

## Commit and PR style

Use clear, product-facing names. Good examples:

- `Add mailbox full fixture coverage`
- `Improve policy rejection classification`
- `Document failure categories`

Avoid names that describe the tool or process used to make the change.

## Code of conduct

All contributors must follow the project [Code of Conduct](CODE_OF_CONDUCT.md).

## Security

Do not disclose suspected vulnerabilities in a public issue. Follow the private reporting instructions in [SECURITY.md](SECURITY.md).

