# Failure Categories

Email Failure Lab v0.1 intentionally keeps categories broad and deterministic. Confidence is rule strength, not a statistical probability. Treat `confidence.level` and `confidence.score` as how strongly the current rules support the report, not as a chance that delivery will succeed or that the remote mailbox state is known.

Inputs are treated as plain text. Multiline SMTP snippets are normalized before classification, so a code, enhanced status code, and matching phrase may appear on separate lines and still produce the same category. Full `.eml`, MIME, attachment, and DSN parsing are intentionally outside the v0.1 category model.

Key off `category`, `bounceType`, and `recommendedAction` in the JSON report. `appGuidance` follows `recommendedAction`. Prefer switching on `recommendedAction` in app code so handlers stay action-centric when several categories share one action.

The `RecommendedAction` contract also includes `contact_recipient` and `no_action_required`. The v0.1 category mapping never returns those values for the categories below.

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

The recipient address appears not to exist or cannot receive mail.

- Default bounce type: `hard`
- Recommended action: `suppress_recipient`
- Typical signals: `5.1.1`, `5.2.1`, `user unknown`, `recipient address rejected`, `mailbox disabled`, `no such user`
- Recommended app behavior: Stop sending to this address. Mark the email as invalid. Ask the user to update their email address.

```typescript
switch (report.recommendedAction) {
  case "suppress_recipient":
    await suppressRecipient(recipientId);
    break;
}
```

## mailbox_full

The recipient mailbox appears to be full or over quota.

- Default bounce type: `soft`
- Recommended action: `retry_later`
- Typical signals: `5.2.2`, `mailbox full`, `quota exceeded`, `over quota`
- Recommended app behavior: Retry later with exponential backoff. Keep the original failure context for debugging. Avoid retrying indefinitely.

```typescript
switch (report.recommendedAction) {
  case "retry_later":
    await scheduleRetry(recipientId, { backoff: "exponential" });
    break;
}
```

## authentication_failure

The receiving server rejected the message because sender authentication appears to be failing.

- Default bounce type: `hard`
- Recommended action: `fix_domain_authentication`
- Typical signals: `5.7.26`, `spf fail`, `dkim fail`, `dmarc fail`, `unauthenticated email`, `this mail is unauthenticated`
- Recommended app behavior: Check SPF, DKIM, and DMARC for the sending domain. Verify that the provider is authorized to send for this domain. Retry only after authentication is fixed.

```typescript
switch (report.recommendedAction) {
  case "fix_domain_authentication":
    await openAuthReview(sendingDomain);
    break;
}
```

## policy_rejection

The receiving server rejected the message because of a policy decision.

- Default bounce type: `hard`
- Recommended action: `review_content`
- Typical signals: `5.7.1`, `rejected by policy`, `message rejected`, `blocked`, `access denied`, `block list`
- Recommended app behavior: Review message content, links, headers, and sending patterns. Check recipient or provider policy requirements. Retry only after changing the likely cause.

```typescript
switch (report.recommendedAction) {
  case "review_content":
    await queueContentReview(messageId);
    break;
}
```

## rate_limited

The receiving server or provider is asking you to slow down sending.

- Default bounce type: `soft`
- Recommended action: `reduce_sending_rate`
- Typical signals: `rate limited`, `rate limit exceeded`, `too many messages`, `throttled`
- Recommended app behavior: Reduce sending rate for this destination. Use backoff before retrying. Avoid retry storms that can worsen throttling.

```typescript
switch (report.recommendedAction) {
  case "reduce_sending_rate":
    await throttleDestination(destinationId);
    break;
}
```

## temporary_failure

The failure appears temporary, so a later retry may succeed.

- Default bounce type: `soft`
- Recommended action: `retry_later`
- Typical signals: `421`, `451`, `temporary failure`, `temporarily deferred`, `try again later`
- Recommended app behavior: Retry later with exponential backoff. Keep the original failure context for debugging. Avoid retrying indefinitely.

```typescript
switch (report.recommendedAction) {
  case "retry_later":
    await scheduleRetry(recipientId, { backoff: "exponential" });
    break;
}
```

## content_rejected

The receiving system appears to have rejected the message content.

- Default bounce type: `hard`
- Recommended action: `review_content`
- Typical signals: `message rejected as spam`, `classified as spam`, `content rejected`, `identified as spam`, `spam detected`
- Recommended app behavior: Review message content, links, headers, and sending patterns. Check recipient or provider policy requirements. Retry only after changing the likely cause.

```typescript
switch (report.recommendedAction) {
  case "review_content":
    await queueContentReview(messageId);
    break;
}
```

## provider_error

The failure appears related to the email provider or an upstream service.

- Default bounce type: `unknown`
- Recommended action: `investigate_provider`
- Typical signals: `provider error`, `internal error`, `upstream error`
- Recommended app behavior: Check provider status and logs. Keep the raw error for support or incident review. Retry only if the provider indicates the issue is transient.

```typescript
switch (report.recommendedAction) {
  case "investigate_provider":
    await openProviderIncident(rawFailure);
    break;
}
```

## unknown

Email Failure Lab could not confidently classify this failure yet.

- Default bounce type: `unknown`
- Recommended action: `unknown`
- Typical signals: no strong recognized category signal
- Recommended app behavior: Keep the raw failure for manual investigation. Add a fixture if this failure becomes common.

```typescript
switch (report.recommendedAction) {
  case "unknown":
    await keepForManualReview(rawFailure);
    break;
}
```
