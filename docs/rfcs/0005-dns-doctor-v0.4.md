# RFC 0005: DNS Doctor architecture for v0.4

- **Status:** Proposed
- **Issue:** #22
- **Created:** 2026-08-07

## Problem statement

Email Failure Lab explains delivery failures after they happen. Many failures start with broken sender DNS. A missing SPF record, a revoked DKIM key, or a `p=none` DMARC policy can later appear as an authentication bounce.

The v0.4 milestone adds DNS Doctor. It inspects a sending domain's SPF, DKIM, and DMARC records. It returns typed findings and an ordered fix plan.

DNS diagnostics need network access. The current core is pure, and the CLI owns effects. This design keeps that boundary. It also leaves the stable `FailureReport` JSON v0.1 contract unchanged.

This RFC records the design only. Follow-up issues will implement it.

## Decision

DNS Doctor has a separate `DnsReport` contract. Follow-up implementation must add `schemas/dns-report.v0.1.json` with the first `DnsReport` implementation. The DNS schema versions independently from `FailureReport`.

A new `email-failure-dns` crate evaluates DNS data without performing I/O. The CLI owns DNS resolution, snapshot files, provider presets, and rendering.

`DnsSnapshot` is the boundary between those layers. The CLI records answers, missing records, timeouts, and network failures in a snapshot. The pure crate turns the snapshot into deterministic findings.

## Goals

1. Define the first SPF, DKIM, and DMARC checks for v0.4.
2. Keep DNS resolution in `email-failure-cli`.
3. Keep evaluation pure in `email-failure-dns`.
4. Define `DnsReport` as a contract separate from `FailureReport`.
5. Model findings and fixes with typed values and stable codes.
6. Report timeouts and network failures as `Inconclusive` findings.
7. Keep provider preset data out of the public library API.
8. Answer all five design questions from issue #22.

## Non-goals

This RFC does not:

- Implement Rust code, add crates, or change CLI behavior.
- Add `schemas/dns-report.v0.1.json` in this docs-only change. Follow-up implementation owns that file.
- Change `email-failure-core`, `explain(ParseInput)`, or `FailureReport` JSON v0.1.
- Add DNS types to the `email-failure-core` public API.
- Project DNS blockers into `FailureReport`. That projection is future work after v0.4.
- Publish, change, or verify DNS records at a registrar or DNS host.
- Call provider APIs, authenticate, or manage provider accounts.
- Claim provider verification from a semantically valid DNS record.
- Check MX, PTR, BIMI, MTA-STS, TLS-RPT, or DNSSEC.
- Walk the SPF include tree or enforce the RFC 7208 ten-lookup limit.
- Guess or enumerate DKIM selectors.
- Send test email or probe SMTP servers.
- Cache lookups, run a background service, or collect telemetry.
- Choose a resolver dependency. The implementation change will make that choice.

## Architecture

DNS Doctor separates collection from evaluation.

```txt
email-lab dns <domain> [--provider P] [--selector S]...
  |
  |  email-failure-cli
  |    1. validate arguments
  |    2. build the query plan
  |    3. resolve DNS or load a snapshot
  |    4. create a DnsSnapshot
  |
  |  email-failure-dns
  |    5. diagnose(&snapshot, &expectations) -> DnsReport
  |
  |  email-failure-cli
  |    6. render text or JSON
```

The pure crate never opens a socket. The same snapshot and expectations produce the same report. Live DNS can change between runs, but that change stays outside evaluation.

### Crate ownership

| Crate | v0.4 responsibility |
| --- | --- |
| `email-failure-core` | No changes and no DNS types. |
| `email-failure-dns` | Owns `DnsSnapshot`, `Expectations`, `DnsReport`, and `diagnose`. Performs pure evaluation. |
| `email-failure-cli` | Owns the `dns` subcommand, the resolver, snapshot files, provider presets, and rendering. |

The new crate keeps the dependency direction clear. The classifier explains failure text. DNS Doctor explains DNS observations. The CLI depends on both, but the libraries do not depend on each other.

## CLI contract

The proposed command is:

```console
email-lab dns <DOMAIN> \
  [--provider <PROVIDER>] \
  [--selector <SELECTOR>]... \
  [--snapshot <FILE>] \
  [--record-snapshot <FILE>] \
  [--timeout-ms <MILLISECONDS>] \
  [--resolver <IP>] \
  [--json] \
  [--verbose]
```

The caller must provide `--provider` or at least one `--selector`. The CLI rejects a request that has neither. Every v0.4 provider preset must supply at least one selector.

This rule prevents a silent DKIM pass when DNS Doctor had no selector to query. It also applies when the CLI loads a snapshot.

The CLI builds three query groups:

1. One apex TXT query for SPF.
2. One TXT query at `_dmarc.<domain>`.
3. One TXT query at `<selector>._domainkey.<domain>` for each selector.

`--provider` selects a private CLI preset. A preset supplies selectors, expected SPF includes, and a minimum DMARC policy. Provider names and preset tables do not enter the public API of `email-failure-dns`.

Each query has a 5000 ms default deadline. `--timeout-ms` changes that deadline. `--resolver` selects a resolver instead of the system resolver.

`--snapshot` reads observations from a file and performs no DNS queries. `--record-snapshot` writes the observations from a live run.

A produced report exits with status 0. Failed checks and inconclusive lookups are report data. Exit status 1 is reserved for usage failures such as an invalid domain, an unreadable snapshot, or an unknown provider.

The default text renderer redacts DKIM public-key material in the `p=` tag after the first 12 characters. `--verbose` shows the full public TXT value. JSON keeps the complete `DnsReport`, including full record values, because redaction is a presentation rule rather than report data.

## Domain models

All proposed types live in `email-failure-dns`. JSON uses camelCase field names and snake_case enum values. This matches the existing `FailureReport` conventions.

### Input types

```rust
pub struct DnsSnapshot {
    pub schema_version: String,
    pub domain: String,
    pub queries: Vec<QueryResult>,
}

pub struct QueryResult {
    pub name: String,
    pub record_type: RecordType,
    pub outcome: LookupOutcome,
}

#[non_exhaustive]
pub enum RecordType {
    Txt,
}

#[non_exhaustive]
pub enum LookupOutcome {
    Answered { records: Vec<String> },
    NoRecord,
    Timeout,
    ServerFailure,
    TransportError,
}
```

`Answered` contains one string per TXT resource record. The CLI joins chunks from the same resource record in wire order. It never joins separate TXT records.

`NoRecord` covers authoritative NXDOMAIN and NODATA responses. `ServerFailure` covers resolver responses such as SERVFAIL and REFUSED. `TransportError` covers failures before a usable DNS response arrives.

v0.4 queries TXT only. `RecordType` remains an enum so later schema versions can add record types.

```rust
pub struct Expectations {
    pub dkim_selectors: Vec<String>,
    pub expected_spf_includes: Vec<String>,
    pub minimum_dmarc_policy: Option<DmarcPolicy>,
}

#[non_exhaustive]
pub enum DmarcPolicy {
    None,
    Quarantine,
    Reject,
}
```

`Expectations` contains provider-independent data. The CLI builds it from explicit selectors and an optional provider preset.

### Evaluation entry point

```rust
pub fn diagnose(
    snapshot: &DnsSnapshot,
    expectations: &Expectations,
) -> DnsReport;
```

This function is pure. It mirrors the shape of `explain(ParseInput) -> FailureReport` without sharing either contract.

### Report types

```rust
pub struct DnsReport {
    pub schema_version: String,
    pub domain: String,
    pub checks: Vec<CheckResult>,
    pub findings: Vec<DnsFinding>,
    pub fix_plan: FixPlan,
}

pub struct CheckResult {
    pub kind: CheckKind,
    pub status: CheckStatus,
}

#[non_exhaustive]
pub enum CheckKind {
    Spf,
    Dkim,
    Dmarc,
}

pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Inconclusive,
}

pub struct DnsFinding {
    pub check: CheckKind,
    pub code: FindingCode,
    pub severity: Severity,
    pub message: String,
    pub lookup: Option<String>,
    pub record: Option<String>,
}

pub enum Severity {
    Error,
    Warning,
    Inconclusive,
}

pub struct FixPlan {
    pub steps: Vec<FixStep>,
}

pub struct FixStep {
    pub finding_code: FindingCode,
    pub title: String,
    pub change: Option<DnsChange>,
    pub note: Option<String>,
}

pub struct DnsChange {
    pub record_type: RecordType,
    pub name: String,
    pub current: Option<String>,
    pub proposed: String,
}
```

`DnsFinding.code` is stable machine-readable data. `message` is display text and may change. `lookup` identifies the query name. `record` carries the observed record when one exists.

`FixPlan` preserves step order. Each `FixStep` points back to one finding. `DnsChange` shows the current and proposed record values.

### Status derivation and finding order

Each check derives its status from its findings. `Error` takes precedence over `Inconclusive`. `Inconclusive` takes precedence over `Warning`. A check with no findings passes.

The report always orders checks as SPF, DKIM, then DMARC. It orders findings by check and then by `FindingCode` declaration order. Fix steps follow their finding order. Resolver completion order never changes the report.

### Finding codes

`FindingCode` is a `#[non_exhaustive]` enum. A code is part of the JSON contract. Implementations must not rename or reuse a published code.

| Code | Check | Severity | Condition |
| --- | --- | --- | --- |
| `spf_record_missing` | SPF | Error | No apex TXT record starts with `v=spf1`. |
| `spf_multiple_records` | SPF | Error | More than one `v=spf1` record exists. |
| `spf_syntax_error` | SPF | Error | A mechanism or qualifier fails to parse. |
| `spf_all_missing` | SPF | Warning | The policy has no terminal `all` mechanism and no `redirect`. |
| `spf_all_permissive` | SPF | Warning | The policy ends in `+all` or `?all`. |
| `spf_expected_include_missing` | SPF | Error | An include from `Expectations` is absent. |
| `dkim_selector_missing` | DKIM | Error | A supplied selector has no TXT record. |
| `dkim_record_invalid` | DKIM | Error | The record does not parse as RFC 6376 tags. |
| `dkim_public_key_missing` | DKIM | Error | The `p=` tag is absent or empty. |
| `dmarc_record_missing` | DMARC | Error | No DMARC record exists at `_dmarc.<domain>`. |
| `dmarc_multiple_records` | DMARC | Error | More than one DMARC record exists. |
| `dmarc_syntax_error` | DMARC | Error | Tag parsing fails or `p=` is absent. |
| `dmarc_policy_none` | DMARC | Warning | The record uses `p=none`. |
| `dmarc_policy_below_expected` | DMARC | Warning | The policy is weaker than the expected minimum. |
| `dmarc_rua_missing` | DMARC | Warning | The record has no `rua=` tag. |
| `lookup_timeout` | Any | Inconclusive | The query exceeds its deadline. |
| `lookup_server_failure` | Any | Inconclusive | The resolver returns SERVFAIL or REFUSED. |
| `lookup_transport_error` | Any | Inconclusive | The resolver transport fails. |

## Initial checks

v0.4 performs presence checks and shallow validation for the three records named in the roadmap.

### SPF

DNS Doctor selects apex TXT records that start with `v=spf1`. It requires exactly one. It parses mechanisms and qualifiers.

It reports syntax errors, a missing terminal `all`, permissive `+all` or `?all`, and provider-expected includes that are absent. It does not recurse through includes or calculate the full lookup budget.

### DKIM

DNS Doctor queries each supplied or preset selector at `<selector>._domainkey.<domain>`. It validates RFC 6376 tag syntax and requires a non-empty `p=` value.

The CLI argument rule guarantees at least one selector. DNS Doctor never reports DKIM as passed without performing a selector query.

A valid record proves only that the published value has the expected shape. It does not prove that a provider owns the key, recognizes the domain, or has verified the account.

### DMARC

DNS Doctor selects TXT records at `_dmarc.<domain>` that start with `v=DMARC1`. It requires exactly one record and a valid `p=` tag.

It warns on `p=none`, a policy below the expected minimum, and a missing `rua=` tag. It parses `adkim`, `aspf`, and `sp` without producing v0.4 findings for them.

## Fix plans

A fix plan is an ordered list of `FixStep` values. Each step names its finding and describes one change. A `DnsChange` provides the record name, the current value, and the proposed value when the tool can form one.

The plan uses angle-bracket placeholders when it cannot derive an account-specific value. A missing DKIM key can propose `<provider-dkim-value>`. The tool never invents a key.

Only `Error` and `Warning` findings produce fix steps. `Inconclusive` findings produce retry guidance because a failed lookup does not prove that DNS must change.

Proposed changes are conservative. For `dmarc_policy_none`, the plan proposes `p=quarantine` and tells the operator to review aggregate reports before moving to `p=reject`.

The tool does not merge multiple SPF records automatically. That operation can change qualifiers, redirects, lookup counts, and sender authorization.

## Network failures and timeouts

The CLI records every query outcome in `DnsSnapshot`. The evaluator maps incomplete lookups to findings:

| Snapshot outcome | Finding code | Check status |
| --- | --- | --- |
| `Timeout` | `lookup_timeout` | `Inconclusive` |
| `ServerFailure` | `lookup_server_failure` | `Inconclusive` |
| `TransportError` | `lookup_transport_error` | `Inconclusive` |

The evaluator continues with independent answers. It never treats a timeout or a network failure as a missing record.

A report still exits with status 0 when some or all queries are inconclusive. Automation reads the report status and finding codes. A future `--strict` mode may map report status to process exit status.

## Provider presets

Provider knowledge changes independently from DNS syntax rules. The CLI therefore owns a private preset table. Each preset maps a provider name to generic `Expectations`.

```rust
struct ProviderPreset {
    name: &'static str,
    dkim_selectors: &'static [&'static str],
    expected_spf_includes: &'static [&'static str],
    minimum_dmarc_policy: Option<DmarcPolicy>,
}
```

`email-failure-dns` receives only the resulting values. It has no provider enum and no provider names in its public API.

The implementation change must verify preset values against provider documentation and record the verification date. This RFC does not freeze a provider's current selector or SPF include.

A passing preset check means that public DNS matches the preset's documented shape. It does not mean that the provider verified the domain or that the records belong to a specific provider account.

One CLI consumes presets in v0.4, so a separate presets crate is not justified. A later frontend can trigger a mechanical extraction into a data crate.

## Snapshot format

`DnsSnapshot` has a documented JSON form. `--snapshot` reads this form. `--record-snapshot` writes it.

```json
{
  "schemaVersion": "0.1",
  "domain": "example.com",
  "queries": [
    {
      "name": "example.com",
      "recordType": "txt",
      "outcome": {
        "kind": "answered",
        "records": [
          "v=spf1 include:_spf.example.net ~all",
          "unrelated-site-verification=abc123"
        ]
      }
    },
    {
      "name": "_dmarc.example.com",
      "recordType": "txt",
      "outcome": {
        "kind": "answered",
        "records": [
          "v=DMARC1; p=none; rua=mailto:reports@example.com"
        ]
      }
    },
    {
      "name": "s1._domainkey.example.com",
      "recordType": "txt",
      "outcome": {
        "kind": "timeout"
      }
    }
  ]
}
```

Unrelated apex TXT records are normal. The SPF evaluator ignores them.

## CLI examples

### Commands

```bash
# Live check with one DKIM selector
email-lab dns example.com --selector s1

# Provider preset supplies selectors and expectations
email-lab dns example.com --provider resend

# Stable JSON for automation
email-lab dns example.com --selector s1 --json

# Offline evaluation with no network
email-lab dns example.com --selector s1 --snapshot ./snapshot.json

# Record live answers for a fixture or bug report
email-lab dns example.com --selector s1 --record-snapshot ./snapshot.json

# Change the per-query deadline and resolver
email-lab dns example.com --selector s1 --timeout-ms 10000 --resolver 1.1.1.1
```

### Text output

The snapshot above produces:

```txt
$ email-lab dns example.com --selector s1
DNS Doctor report for example.com

Checks
  spf     pass
  dkim    inconclusive
  dmarc   warn

Findings
  [inconclusive] lookup_timeout     The query for s1._domainkey.example.com
                                    timed out after 5000 ms.
  [warning]      dmarc_policy_none  _dmarc.example.com sets p=none. Receivers
                                    report failures but do not act on them.

Fix plan
  1. Raise the DMARC policy after reviewing aggregate reports.
     record:   TXT _dmarc.example.com
     current:  v=DMARC1; p=none; rua=mailto:reports@example.com
     proposed: v=DMARC1; p=quarantine; rua=mailto:reports@example.com
     note:     Move to p=reject after monitoring quarantine.

Retry inconclusive lookups with --timeout-ms or a different --resolver.
```

For a DKIM record, default text shortens the key:

```txt
record: v=DKIM1; k=rsa; p=MIIBIjANBgkq...<redacted>
```

`--verbose` prints the full `p=` value.

### JSON output

```json
{
  "schemaVersion": "0.1",
  "domain": "example.com",
  "checks": [
    { "kind": "spf", "status": "pass" },
    { "kind": "dkim", "status": "inconclusive" },
    { "kind": "dmarc", "status": "warn" }
  ],
  "findings": [
    {
      "check": "dkim",
      "code": "lookup_timeout",
      "severity": "inconclusive",
      "message": "The query for s1._domainkey.example.com timed out after 5000 ms.",
      "lookup": "s1._domainkey.example.com",
      "record": null
    },
    {
      "check": "dmarc",
      "code": "dmarc_policy_none",
      "severity": "warning",
      "message": "_dmarc.example.com sets p=none. Receivers report failures but do not act on them.",
      "lookup": "_dmarc.example.com",
      "record": "v=DMARC1; p=none; rua=mailto:reports@example.com"
    }
  ],
  "fixPlan": {
    "steps": [
      {
        "findingCode": "dmarc_policy_none",
        "title": "Raise the DMARC policy after reviewing aggregate reports.",
        "change": {
          "recordType": "txt",
          "name": "_dmarc.example.com",
          "current": "v=DMARC1; p=none; rua=mailto:reports@example.com",
          "proposed": "v=DMARC1; p=quarantine; rua=mailto:reports@example.com"
        },
        "note": "Move to p=reject after monitoring quarantine."
      }
    ]
  }
}
```

## Report contract

`DnsReport` is independent from `FailureReport`. The first implementation must add `schemas/dns-report.v0.1.json` beside `schemas/failure-report.v0.1.json`.

The contracts version independently. A DNS change does not force a `FailureReport` version change.

The `dns-report` v0.1 compatibility rules are:

- Consumers must tolerate new `FindingCode` values.
- Implementations may add optional fields without a schema version change.
- Renaming or removing a field, code, or enum value requires a schema version change.

## Determinism and tests

`diagnose` is pure. Snapshot fixtures will live in `email-failure-dns`. They will use reserved domains and sanitized values.

Every finding code needs one fixture that produces it and one fixture that does not. CLI integration tests must use snapshots or a fake resolver. CI must not depend on public DNS.

For any valid snapshot, serialization followed by parsing and diagnosis must produce the same JSON report as direct diagnosis. Finding order, fix order, and rendered output must not depend on resolver completion order.

Default text rendering tests must prove that DKIM `p=` material is redacted. Verbose rendering tests must prove that the complete value is available.

## Future work

- Walk SPF include trees and enforce the RFC 7208 ten-lookup limit.
- Add DMARC alignment and subdomain-policy findings.
- Add MX, MTA-STS, TLS-RPT, and BIMI checks.
- Add a `--strict` mode for CI exit codes.
- Extract provider presets if a second frontend needs them.
- Decide whether definite DNS blockers should project into `FailureReport`. v0.4 does not perform that projection.
- Add classifier guidance that suggests DNS Doctor for authentication failures.

## Answers to issue #22 questions

1. **Which DNS checks should v0.4 include first?**

   Check SPF presence and shallow syntax at the apex. Check DKIM validity for selectors supplied by the user or a provider preset. Check DMARC presence and policy at `_dmarc.<domain>`. Defer SPF recursion, DMARC alignment, MX, and BIMI.

2. **How should SPF, DKIM, and DMARC findings be modeled?**

   Use typed `DnsFinding` values inside a separate `DnsReport`. Each finding has a `CheckKind`, a stable `FindingCode`, a `Severity`, a message, and optional DNS evidence. Derive each `CheckStatus` from its findings. Do not project findings into `FailureReport` in v0.4.

3. **What should a fix plan look like?**

   Use an ordered `FixPlan` of `FixStep` values. Each step references its finding. When possible, a typed `DnsChange` gives the record name, the current value, and the proposed value. Inconclusive findings get retry guidance instead of DNS changes.

4. **Should provider-specific expectations live in a separate crate?**

   No. Keep a private preset table in the CLI for v0.4. Convert each preset to provider-independent `Expectations` for the pure crate. Extract a data crate only when another frontend needs the same presets.

5. **How should network failures and timeouts be reported?**

   Record each failure in `DnsSnapshot`. Convert it to an `Inconclusive` finding with a stable code. Continue evaluating independent answers and render a complete `DnsReport`. Never treat network uncertainty as a missing record.

## Acceptance mapping

- [x] Add an RFC under `docs/rfcs/`.
- [x] Include domain models and CLI examples in text and JSON.
- [x] Define explicit v0.4 non-goals.
- [x] Answer all five issue #22 questions.
- [x] Change no Rust code.

## External references

- [RFC 7208, Sender Policy Framework](https://www.rfc-editor.org/rfc/rfc7208)
- [RFC 6376, DomainKeys Identified Mail](https://www.rfc-editor.org/rfc/rfc6376)
- [RFC 7489, DMARC](https://www.rfc-editor.org/rfc/rfc7489)
