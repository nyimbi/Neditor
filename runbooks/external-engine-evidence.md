# External Engine Evidence Runbook

Use this runbook for optional transform-engine proof: Graphviz variants, D2,
PlantUML, Java, Pikchr, SQLite, and related executable-path behavior.

## Collect Evidence

Install the optional engines on the verifier host, then run:

```sh
NEDITOR_TEST_PIKCHR=/absolute/path/to/pikchr NEDITOR_TEST_SQLITE3=/absolute/path/to/sqlite3 pnpm run collect:engine-evidence
```

If an engine is unavailable on that host, omit its environment variable and let
the collector record the explicit missing-engine state.

Validate with:

```sh
NEDITOR_EXTERNAL_ENGINE_EVIDENCE_DIR=.tmp/external-engines/external pnpm run check:engines
```

## Expected Evidence

- Engine command path and version probe.
- Platform and architecture.
- App version and Git commit.
- Smoke artifact hashes for generated SVG or CSV output.
- Trust, timeout, and input-mode behavior.
- No secrets or private document content.

## Return Paths

- `.tmp/external-engines/external/<platform>/external-engine-evidence.json`
- Engine-specific files such as `.tmp/external-engines/external/pikchr.json`
  or `.tmp/external-engines/external/sqlite.json` when requested by the work
  order.
