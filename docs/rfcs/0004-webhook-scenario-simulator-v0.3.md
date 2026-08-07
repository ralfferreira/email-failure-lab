# RFC 0004: Webhook scenario simulator for v0.3

- **Status:** Proposed
- **Issue:** #21
- **Created:** 2026-08-07

## Problem statement

Email Failure Lab ships sanitized webhook fixtures. Developers can inspect and classify them, but they cannot replay a sequence against a local webhook handler.

A simulator can grow into a second product. A general scenario language would need event generators, clocks, variables, provider lifecycle rules, and a durable result format. v0.3 does not need those parts. It needs a safe way to replay known JSON bytes, repeat them, and change their delivery order.

This RFC defines that narrow simulator. It records design only. Implementation belongs in later issues after this proposal is accepted.

## Decision

Add a `simulate` command group to `email-lab`. The command builds a named scenario pack from existing built-in fixtures or local JSON files. It supports two replay transforms.

- `Duplicate` adds an exact copy of one selected source.
- `ReverseOrder` reverses the base source order.

The command prints a dry-run plan by default. It can POST the plan only when the user supplies a numeric loopback URL and a separate consent flag.

The replay code belongs in `email-failure-cli`. It reads files, prints plans, and performs optional HTTP requests. `email-failure-core`, `explain(ParseInput)`, and `FailureReport` stay unchanged.

## Goals

1. Replay sanitized provider webhook fixtures without copying fixture files out of the binary.
2. Cover a single event, an exact duplicate, and reversed delivery order.
3. Let a developer add local JSON objects to an ad hoc named pack.
4. Make the default mode deterministic and network-free.
5. Restrict delivery to numeric loopback addresses through hard validation.
6. Mark every simulated request so a local handler can identify it.
7. Keep scenario planning out of the stable `FailureReport` JSON contract.

## Non-goals

v0.3 will not:

- Generate provider payloads or edit fixture fields.
- Define a JSON, YAML, or TOML scenario language.
- Model complete provider event lifecycles.
- Guarantee that two built-in events refer to the same email.
- Receive webhooks, expose a server, or open a listening port.
- Send email or call provider APIs.
- Add webhook signatures, provider credentials, authorization headers, or user-defined headers.
- POST to LAN, public, Unix socket, or DNS-resolved targets.
- Follow redirects or use configured HTTP proxies.
- Add delays, clocks, retries, concurrency, rate controls, or background execution.
- Persist events, plans, responses, or telemetry.
- Display or store response bodies.
- Change provider normalization, classifications, categories, or `FailureReport` JSON v0.1.
- Promise a stable machine-readable replay result in v0.3.

## Initial scenario packs

v0.3 ships three packs. They reuse the Resend-style provider fixtures already compiled into `email-failure-core`.

| Pack | Base sources | Transform | Consumer guidance |
| --- | --- | --- | --- |
| `single-bounce` | `resend-invalid-recipient` | None | Record one permanent bounce and suppress the invalid recipient. |
| `duplicate-bounce` | `resend-invalid-recipient` | Duplicate source 1 once | Apply the bounce once. Treat the second delivery as an exact redelivery. |
| `out-of-order-pair` | `resend-daily-quota`, then `resend-temporary-failure` | Reverse order | Process both events without assuming that webhook timestamps always increase. |

`out-of-order-pair` tests global delivery order. The transform delivers the temporary-failure fixture created at 12:15 before the daily-quota fixture created at 12:10. The fixtures have different `email_id` values, so the pack does not model one message's provider lifecycle. A developer who needs related events can supply local JSON files with matching identifiers.

No built-in pack uses the raw text or `.eml` fixtures. Replay sources must parse as top-level JSON objects.

## Provider boundary

The planner is provider-agnostic. It stores JSON bytes and an optional event type for display. It does not branch on provider fields.

The first built-in packs are Resend-style because v0.2 already ships those sanitized fixtures. This is fixture selection, not a provider abstraction. A local JSON object from another provider can join a custom pack without a new adapter.

`provider_hint` is display metadata. It does not affect validation, ordering, requests, or success.

## Fixture replay model

These names describe internal CLI types. They are not new public core types or stable JSON schemas.

### `ScenarioPack`

```text
ScenarioPack
  name: kebab-case string
  description: string
  consumer_guidance: optional string
  provider_hint: optional string
  sources: ordered list of FixtureSource
  transforms: list of ReplayTransform
```

Built-in packs are static CLI data and include one line of consumer guidance. An ad hoc pack exists only for the current process. v0.3 does not read or write pack manifests.

### `FixtureSource`

```text
FixtureSource
  source_index: positive integer
  origin:
    BuiltIn { fixture_name }
    LocalFile { path }
```

Source indexes follow command order and start at 1. `BuiltIn` resolves through the existing built-in fixture catalog. `LocalFile` is read once when the plan is built.

### `LoadedSource`

```text
LoadedSource
  source_index: positive integer
  display_name: string
  origin: BuiltIn | LocalFile
  payload_bytes: bytes
  event_type: optional string
```

The loader requires UTF-8 JSON with a top-level object. It reads at most 1 MiB per source. It parses only to validate the object and read a top-level string `type` for display. It keeps the original bytes for delivery.

### `ReplayTransform`

```text
ReplayTransform
  Duplicate { source_index }
  ReverseOrder
```

`ReverseOrder` runs first. It reverses the base `FixtureSource` list. Each `Duplicate` then adds one copy immediately after the selected source in the reversed or original order.

Repeating `--duplicate 1` adds another occurrence of source 1. All occurrences use the bytes loaded for the first occurrence. The planner does not reread the fixture or file.

### `Delivery`

```text
Delivery
  ordinal: positive integer
  source_index: positive integer
  occurrence: positive integer
  duplicate_of: optional delivery ordinal
  display_name: string
  event_type: optional string
  payload_bytes: bytes
```

The first delivery for a source has no `duplicate_of`. Every later occurrence points to that first delivery, even when the source was selected for more than one duplicate.

### `ReplayPlan`

```text
ReplayPlan
  scenario_name: string
  provider_hint: optional string
  deliveries: ordered list of Delivery
```

The same built-in pack in the same binary produces the same plan. A custom pack produces the same plan when its arguments and local file bytes are unchanged.

### `DeliveryOutcome`

```text
DeliveryOutcome
  ordinal: positive integer
  status_code: optional integer
  result: Delivered | HttpFailure | TransportFailure
```

`DeliveryOutcome` exists only during a POST run. The CLI renders it as text and does not add it to `FailureReport`.

## Transform semantics

The base list represents expected source order. `ReverseOrder` is the complete out-of-order operation for v0.3. It does not inspect or rewrite timestamps.

`Duplicate` represents provider redelivery. The copied request body is byte-identical to the first occurrence. The simulator does not create a new event ID, timestamp, or signature.

Transforms never mutate payload content. A dry-run plan reports transformed delivery order and duplicate ancestry before any request can run.

The command accepts at most 20 base sources and 100 transformed deliveries. These limits keep the command focused on handler checks instead of load generation.

## CLI surface

Add one top-level command group.

```text
email-lab simulate list
email-lab simulate show <PACK>
email-lab simulate run <PACK>
email-lab simulate run --name <NAME> \
  [--fixture <NAME>]... \
  [--file <PATH>]... \
  [--duplicate <SOURCE_INDEX>]... \
  [--out-of-order] \
  [--post-to <URL> --allow-loopback-post]
```

`simulate list` lists built-in packs. `simulate show` prints a pack's base sources, transforms, final plan, and consumer guidance.

`simulate run <PACK>` uses a built-in pack. It does not accept source or transform flags.

The custom form requires `--name` and at least one `--fixture` or `--file`. Source indexes follow the order of those source flags on the command line. The custom name must be kebab-case. A duplicate index must refer to a base source.

`--out-of-order` maps to one `ReverseOrder` transform. Each `--duplicate N` maps to one `Duplicate { source_index: N }`.

The run remains a dry run unless both `--post-to` and `--allow-loopback-post` are present. `--allow-loopback-post` without `--post-to` is an error.

## Sample commands and expected output

### List built-in packs

```bash
email-lab simulate list
```

```text
Built-in simulation packs (3):
  duplicate-bounce     2 deliveries  resend-style
  out-of-order-pair    2 deliveries  resend-style
  single-bounce        1 delivery    resend-style

Use 'email-lab simulate show <name>' to inspect a plan.
```

### Inspect duplicate representation

```bash
email-lab simulate show duplicate-bounce
```

```text
Scenario: duplicate-bounce
Provider hint: resend-style
Base sources:
  1  built-in:resend-invalid-recipient  email.bounced
Transforms:
  duplicate source 1 once
Delivery plan:
  1  source 1  occurrence 1  resend-invalid-recipient
  2  source 1  occurrence 2  resend-invalid-recipient  duplicate of delivery 1
Consumer guidance: Apply the bounce once. Treat the second delivery as an exact redelivery.
```

The consumer guidance describes the expected handler behavior. The simulator reports responses but does not inspect application state or enforce the guidance.

### Dry-run a custom pack

The local file in this example is a top-level JSON object.

```bash
email-lab simulate run \
  --name local-idempotency \
  --fixture resend-temporary-failure \
  --file ./captured-failure.json \
  --duplicate 1 \
  --out-of-order
```

```text
Scenario: local-idempotency
Mode: dry-run
Deliveries: 3
  1  source 2  occurrence 1  file:./captured-failure.json
  2  source 1  occurrence 1  built-in:resend-temporary-failure
  3  source 1  occurrence 2  built-in:resend-temporary-failure  duplicate of delivery 2

No requests sent.
```

### POST to a loopback handler

```bash
email-lab simulate run duplicate-bounce \
  --post-to http://127.0.0.1:3000/webhooks/resend \
  --allow-loopback-post
```

Expected output when the handler returns `204` twice:

```text
Scenario: duplicate-bounce
Mode: loopback POST
Target: http://127.0.0.1:3000/webhooks/resend
Deliveries: 2
  [1/2] POST resend-invalid-recipient occurrence 1 -> 204
  [2/2] POST resend-invalid-recipient occurrence 2 -> 204

Delivered: 2
Failed: 0
```

Each request includes `User-Agent: email-lab/<version>` and `X-Email-Lab-Simulation: 1`. The process exits with status 0 after a dry run or after all POST requests return a `2xx` status.

### Reject a non-loopback target

```bash
email-lab simulate run single-bounce \
  --post-to https://example.com/webhooks \
  --allow-loopback-post
```

```text
error: POST target must use http and a numeric loopback address
```

The process exits with status 1 and sends no request.

### Require explicit POST consent

```bash
email-lab simulate run single-bounce \
  --post-to http://127.0.0.1:3000/webhooks
```

```text
error: --post-to requires --allow-loopback-post
```

The process exits with status 1 and sends no request.

## Local file behavior

The CLI validates all sources and builds the complete plan before the first request.

A local source fails validation when it:

- Cannot be read.
- Exceeds 1 MiB.
- Is not UTF-8.
- Is malformed JSON.
- Is valid JSON but is not a top-level object.

The CLI does not require `type`, `data`, or a known provider shape. Replay tests transport behavior, not classification. Users can run `email-lab explain <file>` separately when they want a classification.

Dry-run output does not print payload bytes. POST output does not print payloads or response bodies. This rule prevents a captured local event from being copied into terminal logs by default.

## Loopback POST safety

Every condition below is mandatory.

1. The user must pass both `--post-to` and `--allow-loopback-post`.
2. The URL scheme must be `http`.
3. The URL host must parse as a numeric IP address whose loopback property is true. IPv4 `127.0.0.0/8` and IPv6 `::1` qualify.
4. Hostnames such as `localhost` are rejected. The CLI performs no DNS lookup.
5. The URL must include an explicit nonzero port.
6. User information, query strings, and fragments are rejected.
7. The HTTP client ignores proxy environment variables and uses no proxy.
8. Redirects are disabled. A `3xx` response is a failed delivery.
9. Requests run sequentially with one in flight.
10. Each request has a five-second total timeout.
11. The client performs no automatic or application retry.
12. The method is always `POST`.
13. Every request carries `Content-Type: application/json`.
14. Every request carries `User-Agent: email-lab/<version>`.
15. Every request carries `X-Email-Lab-Simulation: 1`.
16. The client sends the exact source bytes. It does not read credentials or headers from environment variables or config files.
17. Any transport error or non-`2xx` response stops the run. The summary reports delivered, failed, and not-attempted counts.

The simulation headers let a local handler reject simulated traffic or route it through test-only behavior. They do not authenticate the request. The simulator does not forge provider signature headers.

Validation happens before client construction and before any request. A validation error sends nothing.

There is no rollback after partial delivery. If delivery 2 fails, delivery 1 may already have reached the local handler. The failure summary must say so.

## Failure behavior

| Condition | Behavior |
| --- | --- |
| Unknown pack | Exit 1 and suggest `email-lab simulate list`. |
| Unknown built-in fixture | Exit 1 and suggest `email-lab fixtures list`. |
| Built-in fixture is not a JSON object | Exit 1 before planning. |
| Invalid source index | Exit 1 before planning. |
| Empty custom pack | Exit 1 before planning. |
| Source or delivery limit exceeded | Exit 1 before planning. |
| Unsafe target URL | Exit 1 before client construction. |
| HTTP `2xx` | Mark the delivery as delivered and continue. |
| HTTP non-`2xx` | Mark the delivery as failed, stop, and exit 1. |
| Timeout or transport error | Mark the delivery as failed, stop, and exit 1. |

Errors go to standard error. Plans, progress, and summaries go to standard output.

## Placement and change boundary

The implementation can stay inside `email-failure-cli`.

```text
email-failure-cli
  simulate command parsing
  built-in pack catalog
  source loading and JSON-object validation
  pure transform planner
  text plan renderer
  loopback URL validator
  optional HTTP sender

email-failure-core
  existing BuiltInFixture catalog
  no new effects
  no FailureReport changes
```

The CLI reuses `find_built_in_fixture`. It does not duplicate provider fixture contents. HTTP library selection belongs to the implementation issue. The selected client must support disabled redirects, disabled proxies, explicit timeouts, fixed simulation headers, and request bodies from exact bytes.

## Implementation verification

Unit tests should cover:

- Static pack names, source names, consumer guidance, and final plans.
- Reverse-before-duplicate transform order.
- Exact byte equality for duplicate deliveries.
- Stable occurrence and `duplicate_of` fields.
- Local JSON object validation and size limits.
- Source and delivery limits.
- Rejection of DNS names, public IPs, private LAN IPs, missing ports, HTTPS, user information, queries, and fragments.
- Acceptance of `127.0.0.1`, another `127/8` address, and `[::1]`.

CLI tests should cover the sample dry-run output and all validation failures without opening a socket.

Loopback integration tests may bind an ephemeral loopback port. They must confirm sequential body order, byte-identical duplicates, simulation headers, disabled redirects, stop-on-failure behavior, and no request when consent or target validation fails.

## Answers to issue #21 questions

1. **Which scenarios should v0.3 support first?**

   One sanitized bounce, one exact duplicate bounce, and one reversed pair of existing Resend-style fixtures. Custom named packs may combine built-in fixtures and local JSON objects.

2. **Should scenarios be provider-specific or provider-agnostic?**

   Replay and transforms are provider-agnostic. Initial built-in packs are Resend-style because v0.2 already ships those fixtures. v0.3 adds no provider adapter.

3. **How should duplicate and out-of-order events be represented?**

   A duplicate is another `Delivery` that reuses the same loaded bytes and points to its first delivery through `duplicate_of`. Out-of-order delivery is the explicit `ReverseOrder` transform over the base source list. Neither transform changes payload fields.

4. **What does the CLI surface look like?**

   The group has `email-lab simulate list`, `simulate show <PACK>`, and `simulate run`. The run command accepts a built-in pack or an ad hoc `--name` with repeated `--fixture` and `--file` sources. `--duplicate` and `--out-of-order` apply the two fixed transforms. `--post-to` and `--allow-loopback-post` enable local delivery together.

5. **What safety constraints apply when POSTing to local endpoints?**

   Dry-run is the default. POST requires dual consent and an `http` URL with a numeric loopback address and explicit port. The client disables DNS, proxies, redirects, retries, concurrency, credentials, and user-defined headers. It enforces source and delivery limits plus a five-second request timeout. Every request carries the fixed simulation headers.
