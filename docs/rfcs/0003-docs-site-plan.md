# RFC 0003: Lightweight documentation site plan

- **Status:** Proposed
- **Issue:** #29
- **Created:** 2026-08-07

## Problem statement

Issue #29 asks whether Email Failure Lab needs a public documentation site. This RFC answers that question and records a plan for later, so the decision does not get relitigated in every release discussion.

The short answer is not yet. All documentation already lives as markdown in the repository, and the audience today is contributors and early adopters who arrive through GitHub. A site would duplicate that content without reaching anyone new, and maintaining it would compete with the v0.2 polish and the v0.3 simulator design for the same limited time.

## Decision

Defer the site until after v0.3 lands or until external contributors ask for one. Until then, the documentation surface is:

- `README.md` for the quickstart and CLI examples.
- `docs/*.md` for guides.
- `schemas/failure-report.v0.1.json` for the JSON contract.
- `ROADMAP.md` for direction and scope boundaries.

When a site becomes justified, build it as a static site generated from the existing markdown, with **mdBook** published to **GitHub Pages**. mdBook fits a Rust repository, installs with `cargo install mdbook`, and keeps the in-repo markdown as the single source of truth. The site becomes a rendering of the repository, not a second content tree to keep in sync.

## Alternatives considered

- **A custom React or Vite app.** Rejected. It adds a frontend toolchain to a Rust repository for content that is plain markdown.
- **Docusaurus.** Rejected for a v1 site. It handles versioned docs well, but that weight is not needed for a repository this size, and it pulls in a Node build.
- **Interactive playgrounds.** Rejected. Issue #29 names them an explicit non-goal, and they would break the no-network, no-service scope boundary in `ROADMAP.md`.

## Information architecture

The site maps existing files. It does not introduce new content:

| Site section | Source |
| --- | --- |
| Quickstart | `README.md` |
| CLI examples | `README.md` |
| JSON schema and reference | `schemas/failure-report.v0.1.json`, plus a short how-to page written later |
| Failure categories | `docs/failure-categories.md` |
| Provider payloads | `docs/provider-webhooks.md` |
| Roadmap | `ROADMAP.md` |
| RFCs | `docs/rfcs/` |

## Non-goals

- No frontend implementation in this issue.
- No playgrounds or interactive demos.
- No second content tree. If a page cannot be generated from an existing file, it waits until someone writes that file in-repo first.
- No hosted service beyond static GitHub Pages.

## Acceptance

Issue #29 accepts either a short proposal under `docs/rfcs/` or a roadmap update. This RFC does both:

- [x] Proposal recorded as `docs/rfcs/0003-docs-site-plan.md`.
- [x] `ROADMAP.md` Exploratory bullet for #29 links this RFC.
