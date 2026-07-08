# Failure Categories

Email Failure Lab v0.1 intentionally keeps categories broad and deterministic. Confidence is rule strength, not a statistical probability.

Inputs are treated as plain text. Multiline SMTP snippets are normalized before classification, so a code, enhanced status code, and matching phrase may appear on separate lines and still produce the same category. Full `.eml`, MIME, attachment, and DSN parsing are intentionally outside the v0.1 category model.

## Confidence scoring

Confidence is a deterministic score for how strongly the current rules support the report. It is not a statistical probability, a delivery-rate prediction, or a claim that the remote mailbox state is known.

Signals contribute the current weights:

- SMTP code: 20
- Enhanced status code: 35
- Strong matched phrase: 35
- Weak matched phrase: 20

After adding signal weights, the scorer applies a few simple adjustments:

- If at least two recognized signals point to the same category, add 10.
- If recognized signals point to conflicting categories, subtract 20.
- If any category is recognized and the input includes an enhanced status code or strong matched phrase, keep the score at 60 or higher, even when that strong signal is not itself category-specific.
- If the only signals are generic SMTP codes, cap the score at 59.
- Clamp the final score to the range 1-99.

The 60-point floor is applied after conflict penalties. That means a mixed or partial input can still end at medium confidence when it includes a strong signal.

Confidence levels are derived from the final score:

- `high`: 90-99
- `medium`: 60-89
- `low`: 1-59

For example, `550 5.1.1 User unknown` is high confidence because the enhanced status code and phrase both point to `invalid_recipient`. A bare `550` remains low confidence because the SMTP code alone does not identify the cause. `421 5.4.1` is medium confidence because `421` points to `temporary_failure` and the enhanced status code triggers the medium-confidence floor, even though `5.4.1` does not map to a category.

## invalid_recipient

- Bounce type: `hard`
- Recommended action: `suppress_recipient`
- Typical signals: `5.1.1`, `user unknown`, `recipient address rejected`, `no such user`
- App handling: stop sending to the address, mark it invalid, and ask the user to update it.

## mailbox_full

- Bounce type: `soft`
- Recommended action: `retry_later`
- Typical signals: `5.2.2`, `mailbox full`, `quota exceeded`, `over quota`
- App handling: retry later with backoff and consider asking the recipient to clear space.

## authentication_failure

- Bounce type: `hard`
- Recommended action: `fix_domain_authentication`
- Typical signals: `spf fail`, `dkim fail`, `dmarc fail`, `unauthenticated email`
- App handling: check sending domain authentication before retrying at volume.

## policy_rejection

- Bounce type: `hard`
- Recommended action: `review_content`
- Typical signals: `5.7.1`, `rejected by policy`, `message rejected`, `blocked`
- App handling: review policy requirements, sending reputation, recipient rules, and message content.

## rate_limited

- Bounce type: `soft`
- Recommended action: `reduce_sending_rate`
- Typical signals: `rate limited`, `too many messages`, `throttled`
- App handling: slow down delivery, apply exponential backoff, and avoid retry storms.

## temporary_failure

- Bounce type: `soft`
- Recommended action: `retry_later`
- Typical signals: `421`, `451`, `temporary failure`, `try again later`
- App handling: retry later with backoff and preserve the original failure context.

## content_rejected

- Bounce type: `hard`
- Recommended action: `review_content`
- Typical signals: `message rejected as spam`, `classified as spam`, `content rejected`, `spam detected`
- App handling: inspect message content, links, headers, and sending patterns before retrying.

## provider_error

- Bounce type: `unknown`
- Recommended action: `investigate_provider`
- Typical signals: `provider error`, `internal error`, `upstream error`
- App handling: inspect provider status and logs before deciding whether to retry.

## unknown

- Bounce type: `unknown`
- Recommended action: `unknown`
- Typical signals: no strong recognized category signal
- App handling: keep the raw error, investigate manually, and add a fixture if it becomes common.

