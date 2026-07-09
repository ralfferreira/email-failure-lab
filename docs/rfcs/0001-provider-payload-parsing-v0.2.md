# RFC 0001: Provider Payload Parsing for v0.2

- **Status:** Proposed
- **Issue:** #18
- **Unblocks:** #19, #23
- **Created:** 2026-07-09
- **External refs verified:** 2026-07-09

## Problem statement

v0.1 of Email Failure Lab classifies plain-text SMTP errors and bounce-like snippets. That foundation is correct, but many developers first see delivery failures as **provider webhook JSON**, not as raw SMTP strings.

Without a designed normalization path, provider support risks becoming:

- provider-locked (for example, Resend-only APIs or event enums in the public model),
- a parallel classifier that duplicates SMTP/phrase rules,
- or a schema-breaking change to `explain(ParseInput)` / `FailureReport`.

This RFC defines a **doc-only** v0.2 design: detect a minimal provider-agnostic JSON shape, extract failure-relevant fields into deterministic text, and hand that text to the existing classification pipeline. Implementation, fixtures, and CLI behavior changes belong to follow-up issues (#19, #23), not this document.

## Goals

1. Define a **provider-agnostic** minimal JSON payload shape for v0.2.
2. Keep classification **deterministic** and compatible with the current text pipeline.
3. Keep the public core interface stable: `explain(ParseInput) -> FailureReport`.
4. Place initial provider normalization in **`email-failure-core`** as pure logic before the current parser.
5. Use Resend-style `email.bounced` / `email.failed` as the **reference case**, without locking the architecture to one provider.
6. Answer the five design questions from #18 explicitly (see [Answers to issue #18 questions](#answers-to-issue-18-questions)).
7. Unblock fixture work (#19) and explain-path support (#23) without resolving those issues here.

## Non-goals

This RFC does **not**:

- Implement Rust code, add crates, or change CLI commands.
- Add a runtime `serde_json` dependency as part of this issue (dependency choice is deferred to implementation PRs).
- Change the public `FailureReport` JSON schema or `schemaVersion`.
- Add real fixtures, fixture files, or tests (see #19).
- Call provider APIs, verify webhooks, authenticate, or store events.
- Parse full MIME, DSN, or `.eml` structures beyond treating input as text/JSON text.
- Introduce provider-specific categories, signal kinds, or reason enums in the public model for v0.2.
- Support every Resend (or other provider) event type in v0.2.
- Resolve #19 or #23; those remain separate implementation issues.

## Current state

Today the pipeline is text-only and lives in `email-failure-core`:

```txt
ParseInput { raw, source }
  -> parse_failure (normalize whitespace/case, extract SMTP / enhanced status / phrases)
  -> classify_failure
  -> infer_bounce_type
  -> recommend_action
  -> calculate_confidence
  -> FailureReport
```

Public entry point:

```rust
pub fn explain(input: ParseInput<'_>) -> FailureReport;
```

Relevant facts for this design:

- Inputs are already-loaded strings (`Inline` or `File`); the core does not read files or the network.
- Multiline SMTP snippets are normalized before classification (`docs/failure-categories.md`).
- Categories and confidence remain rule-based; there is no provider event model in v0.1.
- The CLI (`email-lab explain`) loads text and formats the report; it should stay an effect boundary.

## Proposed v0.2 data flow

Keep `explain(ParseInput)` and `FailureReport` stable. Insert a **provider normalization** step inside core **before** the existing text parser:

```txt
ParseInput.raw
  -> try_normalize_provider_payload(raw)
       |
       +-- not JSON / malformed JSON
       |     -> treat raw as plain text (current behavior)
       |
       +-- valid JSON, unsupported shape or event type
       |     -> short-circuit to FailureReport { category: unknown, ... }
       |        (no CLI hard error; still a normal report)
       |
       +-- supported payload
             -> deterministic failure text
             -> existing parse_failure / classify / report pipeline
```

### Interface decision (v0.2)

| Surface | Decision |
| --- | --- |
| `explain(ParseInput)` | Unchanged signature |
| `FailureReport` | Unchanged public fields and `schemaVersion` for this milestone |
| Provider detection | Inside core, opportunistic on `raw` |
| Separate crate | **Not** in v0.2; keep logic in `email-failure-core` |
| CLI | No new commands; existing `explain` path consumes normalized text once implemented |

Optional later (out of scope here): a private or crate-internal helper such as `normalize_provider_payload(raw) -> Option<ProviderNormalization>` for tests. Public API growth is not required for the first cut.

### Why normalize to text instead of classifying JSON directly

- Reuses SMTP codes, enhanced status codes, and phrase rules already proven in v0.1.
- Keeps the core provider-agnostic: providers differ in envelope shape, but failure meaning still collapses to classifiable text.
- Avoids baking Resend bounce subtypes or failed-reason strings into public enums.

## Minimal payload shape

v0.2 supports a JSON **object** with at least:

```json
{
  "type": "<event-type-string>",
  "data": { }
}
```

### Reference event support

Resend-style events are the reference case ([event types](https://resend.com/docs/webhooks/event-types), verified 2026-07-09):

| `type` | Failure-relevant fields | Notes |
| --- | --- | --- |
| `email.bounced` | `data.bounce.message`, `data.bounce.type`, `data.bounce.subType` | See [email.bounced](https://resend.com/docs/webhooks/emails/bounced) |
| `email.failed` | `data.failed.reason` | See [email.failed](https://resend.com/docs/webhooks/emails/failed) |

Other top-level or nested fields (`created_at`, `email_id`, `from`, `to`, `subject`, `tags`, `message_id`, `diagnosticCode`, etc.) may appear in real webhooks. For v0.2 classification they are **ignored** (see [Unknown fields](#unknown-fields)).

### Provider-agnostic reading

The design keys off:

1. a string `type`, and
2. a `data` object containing a small, documented failure subtree.

A future provider can map into the same pattern (event type + failure text/reason fields) without changing `FailureReport`. v0.2 only **implements** the Resend-like field paths above; other providers are future work.

### Sanitized Resend-like example

No real account, message, or recipient data. Values are placeholders only:

```json
{
  "type": "email.bounced",
  "created_at": "2026-07-09T12:00:00.000Z",
  "data": {
    "email_id": "00000000-0000-4000-8000-000000000001",
    "from": "Example Sender <sender@example.com>",
    "to": ["recipient@example.com"],
    "subject": "Example message",
    "bounce": {
      "message": "550 5.1.1 The email account that you tried to reach does not exist.",
      "type": "Permanent",
      "subType": "General"
    },
    "tags": {
      "campaign": "example"
    }
  }
}
```

After normalization, the existing pipeline should see deterministic text derived from the bounce fields (for example a single line combining message and bounce type/subtype tokens), then classify via current rules—typically toward `invalid_recipient` for this SMTP-bearing message.

Failed-event sketch (also sanitized):

```json
{
  "type": "email.failed",
  "data": {
    "failed": {
      "reason": "reached_daily_quota"
    }
  }
}
```

Normalization turns provider reason tokens into readable, deterministic text so existing phrase rules can apply. Token-splitting `reached_daily_quota` alone (`reached daily quota`) does **not** match current `rate_limited` strong phrases in `docs/failure-categories.md`; see rule 3 for the expected emission for this reference case.

## Normalization rules

1. **Detect JSON opportunistically.** If `raw` is not valid JSON (or is malformed), keep today's plain-text path. If it is valid JSON but not an object with `type` (string) and `data` (object)—or has an unsupported `type`—short-circuit to `FailureReport` with `category: unknown` (same as the failure-behavior table). Only when the minimal shape and a supported `type` are present, enter provider field extraction.
2. **Extract only failure-relevant fields** for supported types:
   - `email.bounced`: `data.bounce.message`, `data.bounce.type`, `data.bounce.subType`
   - `email.failed`: `data.failed.reason`
3. **Build deterministic text.** Concatenate extracted fields in a fixed order with stable separators (implementation detail), then feed that string into `parse_failure` as if the user had pasted SMTP/bounce text. Non-normative reference hint: for `email.failed` with `reason: "reached_daily_quota"`, emit normalized text that includes the existing strong phrase `rate limit exceeded` (for example `reached daily quota: rate limit exceeded`) so current rules classify it as `rate_limited` without expanding the phrase list in this RFC.
4. **Do not map provider enums directly to `FailureCategory`.** Bounce `type` / `subType` and failed `reason` become text tokens; SMTP/enhanced status/phrase rules remain the classifier.
5. **Ignore non-failure metadata** for classification (addresses, subjects, ids, tags, timestamps).
6. **Preserve determinism.** Same payload bytes → same normalized text → same `FailureReport`.

### Mapping intuition (non-normative)

| Provider hint | Likely text signal | Likely category (via existing rules) |
| --- | --- | --- |
| Bounce message containing `550 5.1.1` / user unknown | SMTP + enhanced + phrase | `invalid_recipient` |
| Bounce `Temporary` + mailbox-full language | phrases / soft signals | `mailbox_full` or `temporary_failure` |
| Failed `reached_daily_quota` (reference) | emit text containing `rate limit exceeded` | `rate_limited` via existing phrases |
| Unsupported or empty failure subtree | none | `unknown` |

Exact string templates are left to the implementation PR so fixtures (#19) can lock them down.

## Failure behavior

| Input | Behavior |
| --- | --- |
| Plain SMTP / bounce text | Unchanged text pipeline |
| Malformed JSON (not valid JSON) | **Treat as plain text** until an explicit input-format flag or media type exists |
| Valid JSON without `type`+`data`, or unsupported `type` | Return `FailureReport` with `category: unknown` (and usual unknown guidance); **not** a CLI usage error |
| Supported type but missing failure subtree / empty useful fields | `unknown` report |
| Supported type with classifiable extracted text | Normal classified `FailureReport` |

Rationale:

- Malformed JSON that looks like log noise should not become a hard CLI failure; v0.1 already returns `unknown` for unrecognized text.
- Valid but unsupported provider JSON is intentionally distinguishable as “we understood JSON shape but not this event,” still via `unknown` rather than exit-code errors, so automation keeps a stable report contract. Issue #23 may document CLI messaging; this RFC chooses **report-level unknown**, not CLI error, for unsupported JSON objects.

## Fixtures

This RFC does **not** add fixture files. Initial fixture expectations for follow-ups:

1. Sanitized `email.bounced` payloads covering at least:
   - invalid recipient (SMTP-bearing bounce message),
   - temporary / soft failure language,
   - authentication-related bounce message.
2. Sanitized `email.failed` payload(s) with provider reason strings.
3. Negative cases: unsupported `type`, valid JSON non-object, malformed JSON-as-text.
4. Store under the existing fixture layout once #19 lands; keep examples free of real emails, ids, or secrets.

#19 owns fixture files and tests. #23 owns wiring `email-lab explain` to accept these payloads end-to-end.

## Future work

- Implement normalization in `email-failure-core` and wire through `explain` (#23).
- Add sanitized fixtures and tests (#19); optionally consider phrase-list expansion (`daily quota` / `reached.*quota`) later so raw token-split reasons can match without emission hints.
- Optionally extract a dedicated provider crate **after** the in-core path proves stable.
- Additional event types (`email.delivery_delayed`, complaints, suppressions) only when they map cleanly to existing categories.
- Explicit `--format json|text` (or similar) if opportunistic detection proves ambiguous.
- Additional providers mapped to the same `type` + `data` failure-field pattern.
- Consider whether unknown/unsupported JSON should later expose a dedicated signal or explanation string—without changing categories in the first implementation.

## Answers to issue #18 questions

1. **What minimal provider payload shape should v0.2 support?**  
   A JSON object with string `type` and object `data`. Reference support: `email.bounced` (`data.bounce.message` / `type` / `subType`) and `email.failed` (`data.failed.reason`).

2. **Should provider parsing live in the core crate or a separate crate?**  
   Initially in **`email-failure-core`** as pure normalization before the current parser. Do **not** create a separate crate in v0.2.

3. **How should provider-specific reasons normalize into current categories?**  
   Extract failure-relevant fields into deterministic text; leave classification to existing SMTP / enhanced status / phrase rules. Do not add provider-specific category enums in v0.2.

4. **How should unknown provider fields be preserved or ignored?**  
   **Ignore** for classification. Do **not** preserve them in public `FailureReport` JSON for now.

5. **What fixtures are needed first?**  
   Sanitized Resend-like bounced (and failed) payloads for invalid recipient, temporary failure, and authentication failure, plus negative/unsupported JSON cases—owned by #19 after this RFC is accepted.

## External references

Verified 2026-07-09:

- [Resend webhook event types](https://resend.com/docs/webhooks/event-types)
- [Resend `email.bounced`](https://resend.com/docs/webhooks/emails/bounced)
- [Resend `email.failed`](https://resend.com/docs/webhooks/emails/failed)
