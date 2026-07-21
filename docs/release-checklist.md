# How to release Email Failure Lab

This checklist creates a versioned Git tag and GitHub release from a tested `main` commit.

Publishing packages to crates.io is not part of this process. Add that step only after package ownership, dependency versioning, and publication order have been designed and accepted.

## Prerequisites

- Maintainer access to `ralfferreira/email-failure-lab`.
- An authenticated GitHub CLI session: `gh auth status`.
- Rust 1.85 with `rustfmt` and `clippy`; `rust-toolchain.toml` pins the baseline toolchain.
- A clean checkout synchronized with `origin/main`.
- A semantic version in the form `X.Y.Z`.

## 1. Prepare a release pull request

Create a short-lived branch from the latest `main`:

```bash
git switch main
git pull --ff-only origin main
git switch -c release/vX.Y.Z
```

Update `version` under `[workspace.package]` in `Cargo.toml`. Then refresh `Cargo.lock` without changing dependency versions:

```bash
cargo check --workspace
```

In `CHANGELOG.md`:

1. Rename `Unreleased` to `[X.Y.Z] - YYYY-MM-DD`.
2. Keep only changes included in this release under that heading.
3. Add a new empty `Unreleased` section above it.
4. After the first tag exists, add comparison links for that release and for `Unreleased`.

The crate version and `FailureReport` schema version are independent. Do not rename `schemas/failure-report.v0.1.json` or change `schemaVersion` only because the workspace version changes.

Review the version and lockfile changes:

```bash
cargo metadata --no-deps --format-version 1
git diff -- Cargo.toml Cargo.lock CHANGELOG.md
```

Open a release pull request and merge it only after the checks below pass.

## 2. Run the release checks

Run the same checks required by CI:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Compile the benchmark target without measuring the current machine:

```bash
cargo bench --workspace --no-run
```

On Windows, install the GNU toolchain if the pinned MSVC toolchain or linker is unavailable:

```bash
rustup toolchain install 1.85.0-x86_64-pc-windows-gnu
```

Then replace `cargo` at the start of every Cargo command in this guide with `cargo +1.85.0-x86_64-pc-windows-gnu`. This applies to the version refresh, formatting, Clippy, tests, benchmark compilation, and CLI verification. For example:

```bash
cargo +1.85.0-x86_64-pc-windows-gnu fmt --check
cargo +1.85.0-x86_64-pc-windows-gnu clippy --workspace --all-targets --locked -- -D warnings
cargo +1.85.0-x86_64-pc-windows-gnu test --workspace --locked
cargo +1.85.0-x86_64-pc-windows-gnu bench --workspace --no-run
cargo +1.85.0-x86_64-pc-windows-gnu run --locked -p email-failure-cli -- --version
```

## 3. Verify the CLI

Confirm that the binary reports the intended version:

```bash
cargo run --locked -p email-failure-cli -- --version
```

Exercise the main input and output paths:

```bash
cargo run --locked -p email-failure-cli -- explain "550 5.1.1 User unknown"
cargo run --locked -p email-failure-cli -- explain "550 5.1.1 User unknown" --json
cargo run --locked -p email-failure-cli -- explain ./crates/email-failure-core/fixtures/providers/resend/email-bounced-invalid-recipient.json --json
cargo run --locked -p email-failure-cli -- fixtures list
cargo run --locked -p email-failure-cli -- fixtures show resend-daily-quota
```

Verify that text remains readable, JSON still reports `schemaVersion: "0.1"`, provider input classifies successfully, and the built-in fixture commands work.

## 4. Tag the merged commit

After the release pull request is merged, update local `main` and confirm that the release commit is checked out:

```bash
git switch main
git pull --ff-only origin main
git status --short
git log -1 --oneline
```

Create and push an annotated tag:

```bash
git tag -a vX.Y.Z -m "Email Failure Lab vX.Y.Z"
git push origin vX.Y.Z
```

Do not move or reuse a published version tag. If a released version is wrong, fix it in a new patch release.

## 5. Create the GitHub release

Copy the matching version section from `CHANGELOG.md` into a release notes file, then run:

```bash
gh release create vX.Y.Z --verify-tag --title "Email Failure Lab vX.Y.Z" --notes-file release-notes.md
```

Verify the published metadata:

```bash
git ls-remote --tags origin vX.Y.Z
gh release view vX.Y.Z --json tagName,name,isDraft,isPrerelease,publishedAt,url
```

Remove the local release notes file after publication if it is not intended for the repository.

## 6. Verify the published source

Use a separate clean checkout or worktree at the tag and rerun the tests:

```bash
release_check_dir="../email-failure-lab-release-check"
git worktree add "$release_check_dir" vX.Y.Z
(
  cd "$release_check_dir"
  cargo test --workspace --locked
  cargo run --locked -p email-failure-cli -- --version
)
```

Confirm that the GitHub release page links to the expected tag and that the README examples match the released CLI.

Confirm that the version command from the tagged worktree prints `email-lab X.Y.Z`.

Remove the verification worktree after the subshell exits:

```bash
git worktree remove ../email-failure-lab-release-check
```

## Troubleshooting

- **MSVC reports a missing linker:** install Visual Studio Build Tools with the C++ workload or use the documented GNU Rust toolchain.
- **CI changes `Cargo.lock`:** run `cargo check --workspace`, commit the refreshed lockfile, and rerun all locked checks.
- **A tag was created locally on the wrong commit:** delete the local tag before pushing it with `git tag -d vX.Y.Z`, check out the correct merged commit, and recreate it.
- **A pushed release contains an error:** do not move the published tag. Correct release notes in GitHub for documentation-only mistakes, or publish a new patch version for code or package changes.
