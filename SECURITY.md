# Security Policy

## Supported versions

Email Failure Lab has not published a tagged release yet.

| Version | Supported |
| --- | --- |
| Latest commit on `main` | Yes |
| Older commits, forks, and unofficial builds | No |

After tagged releases begin, this table will identify the supported release line. Until then, security fixes target `main`.

## Reporting a vulnerability

Please report suspected vulnerabilities through GitHub's [private vulnerability reporting form](https://github.com/ralfferreira/email-failure-lab/security/advisories/new).

Do not open a public issue for a vulnerability and do not include secrets, real recipient data, provider tokens, or unsanitized webhook payloads in a report.

Include as much of the following as you can:

- The affected commit, build, or environment.
- Your operating system and Rust toolchain.
- A clear description of the security impact.
- Minimal, reproducible steps using sanitized data.
- Any known mitigations or workarounds.
- Whether the issue has been disclosed anywhere else.

## Security issue or ordinary bug?

Use private reporting when a defect could affect confidentiality, integrity, availability, unsafe file or input handling, or the software supply chain.

Classifier mistakes, missing failure phrases, documentation errors, and feature requests normally belong in a [public issue](https://github.com/ralfferreira/email-failure-lab/issues/new/choose) unless they create a concrete security impact.

## Response and disclosure

This is an early, maintainer-led project. Reports will be reviewed in good faith, but the project does not promise a fixed response time or a bug bounty.

Please allow time to validate the report and prepare a fix before public disclosure. When appropriate, the project will coordinate disclosure through a GitHub security advisory and credit the reporter if they want to be named.
