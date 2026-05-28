# External Evidence Runbook

Use this runbook for work orders classified as `external-evidence`: live
provider checks, real-device checks, Google authorization checks, independent
review, and release-device evidence that cannot be synthesized locally.

## General Flow

1. Start from a clean checkout at the target commit.
2. Run the prerequisite local checks listed by the assigned work order.
3. Collect evidence only on the host or account that can genuinely exercise the
   external dependency.
4. Redact secrets and private content before returning evidence.
5. Save the validator-shaped JSON under the return path named by the work order.
6. Run the corresponding `pnpm run check:*` validator.
7. Ingest with `pnpm run ingest:evidence -- --source <returned-evidence-dir>`.
8. Re-run `pnpm run check:release-readiness` and
   `pnpm run check:spec-completion`.

## Required Metadata

Every returned evidence JSON must include:

- `schema`
- `appVersion`
- `sourceCommit`
- `sourceTreeClean: true`
- `platform`
- `reviewer` or `host`
- Artifact hashes or report hashes
- Redaction statement

## Do Not Return

- API keys, OAuth refresh tokens, certificates, or signing secrets.
- Raw microphone recordings.
- Raw clipboard content.
- Customer documents or proprietary test prompts.
