# RFC 0002: Agent-Friendly MCP Wrapper

- **Status:** Proposed
- **Issue:** #28
- **Unblocks:** implementation follow-up (TBD)
- **Created:** 2026-08-07

## Problem

Email Failure Lab classifies delivery failures deterministically, but agents cannot call it today without shell access and custom glue. An agent that receives a bounce snippet or a provider webhook has to know the `email-lab` CLI, its flags, and its output format.

MCP (Model Context Protocol) gives agents a standard way to discover and call tools. A thin MCP wrapper lets any MCP client classify failures and browse the built-in fixtures without learning the CLI.

This RFC is design only. It defines the first MCP surface, where the wrapper lives, and how it maps to the existing CLI. Implementation belongs to a follow-up issue.

## Goals

1. Expose failure classification to MCP clients with the smallest useful tool set.
2. Reuse the deterministic core through the existing `email-lab` CLI. No reimplementation of classification.
3. Keep `FailureReport` JSON v0.1 (`schemas/failure-report.v0.1.json`) as the tool result contract.
4. Keep the wrapper thin enough that a future switch to in-process core bindings does not change the tool surface.
5. Answer the five design questions from #28 explicitly (see [Answers to issue #28 questions](#answers-to-issue-28-questions)).

## Non-goals

This RFC does not:

- Implement the MCP server, add packages, or change CLI commands.
- Add LLM-based classification. The deterministic core remains the source of truth.
- Call provider APIs, verify webhooks, or store events.
- Add file-path or URL inputs to any tool in the first slice.
- Add a hosted playground or web UI.
- Change the public `FailureReport` schema or bump `schemaVersion`.
- Build Node bindings to the core crate. That is the documented future alternative, not v1.

## Current state

The `email-lab` CLI is the only entry point to the classifier:

- `email-lab explain` accepts inline text, a file, or stdin, and prints a report as text or JSON.
- `email-lab fixtures list` and `email-lab fixtures show` expose the built-in sanitized examples.
- Provider webhook JSON (Resend-style `email.bounced` and `email.failed`) is normalized inside the core, so `explain` already accepts it as raw input (RFC 0001).
- The core is pure. All file and network effects live at the CLI boundary.

There is no MCP server and no non-Rust API surface.

## Proposed design

Add a thin MCP server package that spawns the `email-lab` CLI per call. The server holds no state and does no classification. It validates tool arguments, runs the CLI, and returns its output.

The server prefers structured CLI output (`--json` or `--format json`, whichever the CLI provides) so tool results carry `FailureReport` JSON verbatim.

### Tools

Three tools in the first slice.

#### `email_failure_explain`

Classifies one failure input.

Input schema:

```json
{
  "type": "object",
  "properties": {
    "raw": { "type": "string", "description": "Failure text or provider webhook JSON, inline." }
  },
  "required": ["raw"]
}
```

`raw` is the only parameter. Provider JSON goes inside `raw` because the core already normalizes it. There is no file path, no URL, and no provider discriminator. Adding a discriminator would duplicate detection logic the core already owns.

The result is the `FailureReport` JSON from `email-lab explain`, unchanged.

Example request with SMTP text:

```json
{
  "name": "email_failure_explain",
  "arguments": {
    "raw": "550 5.1.1 The email account that you tried to reach does not exist."
  }
}
```

Example successful result, captured from `email-lab explain … --json` against this repository on 2026-08-07:

```json
{
  "schemaVersion": "0.1",
  "category": "invalid_recipient",
  "bounceType": "hard",
  "recommendedAction": "suppress_recipient",
  "confidence": {
    "level": "medium",
    "score": 60
  },
  "explanation": "The recipient address appears not to exist or cannot receive mail.",
  "appGuidance": [
    "Stop sending to this address.",
    "Mark the email as invalid.",
    "Ask the user to update their email address."
  ],
  "signals": [
    {
      "kind": "smtp_code",
      "value": "550",
      "weight": 20
    },
    {
      "kind": "enhanced_status_code",
      "value": "5.1.1",
      "weight": 35
    }
  ]
}
```

Example request with unsupported provider JSON. This is a successful result, not an error:

```json
{
  "name": "email_failure_explain",
  "arguments": {
    "raw": "{\"type\": \"email.opened\", \"data\": {\"email_id\": \"00000000-0000-4000-8000-000000000001\"}}"
  }
}
```

Result, also captured from the CLI on 2026-08-07:

```json
{
  "schemaVersion": "0.1",
  "category": "unknown",
  "bounceType": "unknown",
  "recommendedAction": "unknown",
  "confidence": {
    "level": "low",
    "score": 1
  },
  "explanation": "Email Failure Lab could not confidently classify this failure yet.",
  "appGuidance": [
    "Keep the raw failure for manual investigation.",
    "Add a fixture if this failure becomes common."
  ],
  "signals": []
}
```

An `unknown` category is a valid answer from a deterministic classifier. Agents need to receive it as data they can act on, not as a transport failure.

#### `email_failure_fixtures_list`

Lists the built-in sanitized fixtures. Takes no arguments.

Maps to `email-lab fixtures list`. If the CLI offers JSON output for this command, the tool returns it. If the CLI output is text only, the v1 server parses the list into an array of fixture names. Implementers may switch to structured CLI output later without changing the tool result shape.

#### `email_failure_fixtures_show`

Returns the content of one fixture for inspection. It does not run `explain` on it.

Input schema:

```json
{
  "type": "object",
  "properties": {
    "name": { "type": "string", "description": "Fixture name as returned by email_failure_fixtures_list." }
  },
  "required": ["name"]
}
```

Maps to `email-lab fixtures show <name>`. An agent that wants the fixture classified calls `email_failure_explain` with the fixture content as `raw`. Keeping show and explain separate keeps each tool doing one thing.

## Shell-out vs bind

Two candidate architectures were compared.

**Candidate A, CLI shell-out (accepted for v1).** The MCP server spawns `email-lab` per call. The CLI is already the tested effect boundary, so the wrapper inherits its input handling, its JSON output, and its determinism for free. The cost is a process spawn per call and a runtime dependency on an installed binary. For a local developer tool with human-scale call rates, that cost is acceptable.

**Candidate B, in-process core bind (documented future alternative).** Bind the `email-failure-core` crate into the server process, for example through napi-rs or a Python extension. This removes the binary dependency and the spawn cost, but it adds a native build matrix and a second public surface into the core. That packaging work is not justified until the shell-out path proves too costly in practice.

The tool surface defined above is identical under both candidates. Switching from A to B later changes only the server internals.

## Repo placement

The MCP server lives in this repository, in a directory such as `mcp/email-failure-lab/`. It is a thin Node/TS (or Python) package that spawns the CLI.

Same-repo placement keeps the server versioned with the CLI it wraps, so a CLI flag change and the server change that absorbs it land in one PR. A separate git repo adds release coordination for a package that is a few hundred lines. If the server grows its own release cadence later, extraction remains possible.

## Failure and error behavior

The dividing line is simple. If the classifier produced a report, the tool call succeeded. Errors are reserved for cases where no report exists.

| Condition | Behavior |
| --- | --- |
| Classifiable text or supported provider JSON | Successful result with the `FailureReport` |
| Unrecognized text or unsupported provider JSON | Successful result with `category: unknown` |
| Empty `raw` input | Tool error |
| Input larger than 256 KiB | Tool error before spawning the CLI |
| `email-lab` binary not found | Tool error naming the missing binary and how to install it |
| CLI exits non-zero unexpectedly | Tool error including exit code and stderr |
| Unknown fixture name in `email_failure_fixtures_show` | Tool error |

The 256 KiB cap protects the spawn boundary. Real bounce snippets and webhook payloads are a few KiB. Anything larger is almost certainly a misdirected input such as a full mailbox export.

## Future work

- Implementation follow-up issue: build the server, add tests against the real CLI, document setup for common MCP clients.
- Structured CLI output for `fixtures list` and `fixtures show` if text parsing proves brittle.
- Candidate B (in-process core bind) if shell-out packaging or binary distribution proves too costly.
- Additional tools only when an agent workflow demands them. File-path input stays out until a concrete need appears.

## Answers to issue #28 questions

1. **Should the MCP wrapper shell out to the CLI or call the Rust core through a binding?**  
   Shell out to the CLI for v1. The CLI is the existing tested effect boundary. In-process binding is the documented future alternative if shell-out costs prove high.

2. **What tool names and inputs should be exposed?**  
   Three tools: `email_failure_explain` with `{ "raw": string }`, `email_failure_fixtures_list` with no arguments, and `email_failure_fixtures_show` with `{ "name": string }`. Show is inspection only and does not classify.

3. **Should the wrapper support only raw text first, or also files and provider payloads?**  
   Inline `raw` text only in the first slice. Provider JSON may appear inside `raw` because the core already normalizes it (RFC 0001). File paths and URLs stay out. An agent with filesystem access reads the file itself and passes the contents as `raw`.

4. **How should errors be represented for agent consumers?**  
   Any produced `FailureReport` is a successful tool result, including `category: unknown`. Tool errors cover only missing binary, unexpected non-zero CLI exit, empty input, input over 256 KiB, and unknown fixture names.

5. **What parts belong in this repository versus a separate package?**  
   The MCP server lives in this repository under `mcp/email-failure-lab/` or similar, as a thin Node/TS (or Python) package that spawns the CLI. A separate git repo is not justified for v1.
