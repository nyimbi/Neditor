# Spec Completion Closure Runbook

Use this runbook when closing implementation, local proof, documentation proof,
or matrix triage work.

## Closure Rules

- Do not mark a row `Complete` until direct current evidence proves the exact
  requirement.
- Do not use historical workflow output as current proof.
- Do not replace a broad requirement with a narrower one.
- Do not count manual, external, signing, or platform evidence as complete until
  the validator-shaped artifact is accepted.

## Steps

1. Read `docs/specification.md` for the relevant requirement.
2. Inspect the current implementation and tests.
3. Add missing code, tests, artifact checks, or documentation.
4. Run the smallest verification set that proves the changed requirement.
5. Update `docs/spec-completion-matrix.md` only after evidence exists.
6. Add a concise progress entry to `docs/progress.md` if the matrix changes.
7. Run:

```sh
pnpm run check:docs
pnpm run check:spec-completion
```

8. If release readiness evidence changed, also run:

```sh
pnpm run check:release-readiness
```

## Completion Test

A row is closed only when the evidence would convince a reviewer who has no
memory of the implementation work and only the current checkout, command output,
and artifacts.
