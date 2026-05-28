# Manual Review Runbook

Use this runbook for spec-completion work orders classified as `manual-review`.
Manual evidence must come from a named human reviewer using a current clean
checkout or a packaged build from the current commit.

## Preconditions

- Confirm the checkout is clean with `git status --short`.
- Run the automated prerequisite checks named by the work order.
- Use the current generated work orders from `pnpm run check:spec-completion`.
- Do not include secrets, API keys, customer documents, raw audio, or private
  clipboard contents in returned evidence.

## Review Steps

1. Open `.tmp/spec-completion/work-orders.md` and locate the assigned work order.
2. Run each validator command listed in the work order.
3. Exercise the named workflow in the packaged app or current local app.
4. Capture screenshots, exported artifacts, or native-viewer evidence paths.
5. Record every checklist item as pass or as a non-release-blocking exception
   with a specific rationale.
6. Save the completed sign-off as
   `.tmp/manual-review/<work-order-id>/signoff.json`.
7. Put screenshots or related artifacts under
   `.tmp/manual-review/<work-order-id>/artifacts/`.
8. Ingest the evidence with
   `pnpm run ingest:evidence -- --source .tmp/manual-review/<work-order-id>`.
9. Re-run `pnpm run check:release-readiness` and
   `pnpm run check:spec-completion`.

## Required Sign-Off Shape

```json
{
  "schema": "neditor.manual-review.signoff.v1",
  "workOrderId": "001-manual-review-example",
  "reviewer": "Reviewer Name",
  "platform": "darwin-arm64",
  "appVersion": "0.1.0",
  "sourceCommit": "git-sha",
  "sourceTreeClean": true,
  "artifacts": ["artifacts/screenshot.png"],
  "checklist": [
    {
      "id": "workflow-01",
      "status": "pass",
      "evidence": "artifacts/screenshot.png",
      "notes": "Observed the required state."
    }
  ],
  "unresolvedBlockers": []
}
```

## Acceptance Criteria

- The reviewer is named.
- The app version and Git commit match the release candidate under review.
- Every checklist item has evidence.
- `unresolvedBlockers` is empty.
- The listed validator commands pass after ingest.
