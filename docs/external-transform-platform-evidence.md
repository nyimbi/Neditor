# External Transform Platform Evidence

Updated: 2026-05-25

NEditor keeps external transform engines optional. This page records current
platform evidence for configured local engines and the repeatable command used
to refresh that evidence.

## Local Probe Command

```sh
pnpm run check:engines
```

On this macOS host, Pikchr is verified with the workspace-built executable:

```sh
NEDITOR_TEST_PIKCHR=/Users/nyimbiodero/src/pjs/tooling/neditor/.tmp/pikchr-build/pikchr-trunk/pikchr pnpm run check:engines
```

To collect reusable evidence JSON for every installed compatible engine, run:

```sh
NEDITOR_TEST_PIKCHR=/Users/nyimbiodero/src/pjs/tooling/neditor/.tmp/pikchr-build/pikchr-trunk/pikchr pnpm run collect:engine-evidence
```

The probe reports installed/missing optional engines without failing solely
because an optional engine is absent. For installed diagram engines, it renders
a small SVG smoke artifact through the same adapter shape NEditor uses. For
SQLite, it runs a read-only in-memory CSV query through `sqlite3`. The probe
writes those artifacts under `.tmp/external-engines/artifacts/`.

The same command now writes external evidence templates under
`.tmp/external-engines/templates/`. Copy a completed evidence file into
`.tmp/external-engines/external/<engine>.json`, or point
`NEDITOR_EXTERNAL_ENGINE_EVIDENCE_DIR` at another directory, when a supported
host has an optional engine that is not installed locally. Supplied evidence is
validated for schema, engine key, status, timestamp, platform, command, version,
smoke bytes, smoke hash, required text markers, and zero unresolved
blockers. Malformed supplied evidence fails the probe; absent optional engines
without copied proof remain explicit release-readiness gaps.

`pnpm run collect:engine-evidence` uses the same smoke proof but writes accepted
`neditor.external-engine-evidence.v1` JSON under both the legacy flat
`.tmp/external-engines/external/<engine>.json` path and the platform-qualified
`.tmp/external-engines/external/<platform>/<engine>.json` path. The validator
accepts either shape and aggregates multiple platform files for the same
engine, so Linux and Windows evidence can be ingested without overwriting the
macOS proof for that engine.

The release evidence workflow now includes a Linux optional-engine proof job.
It installs Graphviz, Java/PlantUML, SQLite, D2, and Pikchr CLI, runs:

```sh
pnpm run collect:engine-evidence -- --require-installed
pnpm run check:engines
```

and uploads `.tmp/external-engines/external/linux/*.json` plus the smoke
artifacts and probe report as `neditor-optional-engine-evidence-linux`.
Release owners can ingest that artifact directly:

```sh
pnpm run ingest:evidence -- --source /path/to/neditor-optional-engine-evidence-linux
pnpm run check:engines
```

Use:

```sh
pnpm run check:engines -- --require-installed
```

when a release checklist needs every optional engine installed on the host.

## macOS Evidence

Host:

```text
Darwin arm64
```

Latest local command results:

```text
Graphviz / DOT: installed
  command: dot
  path: /opt/homebrew/bin/dot
  version: dot - graphviz version 15.0.0 (20260523.1842)
  smoke: passed
  artifact: .tmp/external-engines/artifacts/dot.svg
  bytes: 1553
Graphviz / circo: installed
  command: circo
  path: /opt/homebrew/bin/circo
  version: circo - graphviz version 15.0.0 (20260523.1842)
  smoke: passed
  artifact: .tmp/external-engines/artifacts/circo.svg
  bytes: 1467
Graphviz / neato: installed
  command: neato
  path: /opt/homebrew/bin/neato
  version: neato - graphviz version 15.0.0 (20260523.1842)
  smoke: passed
  artifact: .tmp/external-engines/artifacts/neato.svg
  bytes: 1495
Graphviz / fdp: installed
  command: fdp
  path: /opt/homebrew/bin/fdp
  version: fdp - graphviz version 15.0.0 (20260523.1842)
  smoke: passed
  artifact: .tmp/external-engines/artifacts/fdp.svg
  bytes: 1507
Graphviz / osage: installed
  command: osage
  path: /opt/homebrew/bin/osage
  version: osage - graphviz version 15.0.0 (20260523.1842)
  smoke: passed
  artifact: .tmp/external-engines/artifacts/osage.svg
  bytes: 1460
Graphviz / twopi: installed
  command: twopi
  path: /opt/homebrew/bin/twopi
  version: twopi - graphviz version 15.0.0 (20260523.1842)
  smoke: passed
  artifact: .tmp/external-engines/artifacts/twopi.svg
  bytes: 1455
D2: installed
  command: d2
  path: /opt/homebrew/bin/d2
  version: 0.7.1
  smoke: passed
  artifact: .tmp/external-engines/artifacts/d2.svg
  bytes: 15694
PlantUML: installed
  command: plantuml
  path: /opt/homebrew/bin/plantuml
  version: PlantUML version 1.2026.4 / 7d5d424 [2026-05-20 20:25:26 UTC]
  smoke: passed
  artifact: .tmp/external-engines/artifacts/plantuml.svg
  bytes: 3289
Pikchr: installed
  command: /Users/nyimbiodero/src/pjs/tooling/neditor/.tmp/pikchr-build/pikchr-trunk/pikchr
  path: /Users/nyimbiodero/src/pjs/tooling/neditor/.tmp/pikchr-build/pikchr-trunk/pikchr
  version: pikchr 1.0 20260403102956
  smoke: passed
  artifact: .tmp/external-engines/artifacts/pikchr.svg
  bytes: 1420
SQLite / sqlite3: installed
  command: sqlite3
  path: /usr/bin/sqlite3
  version: 3.51.0 2025-06-12 13:14:41 f0ca7bba1c5e232e5d279fad6338121ab55af0c8c68c84cdfb18ba5114dcaapl (64-bit)
  smoke: passed
  artifact: .tmp/external-engines/artifacts/sqlite.csv
  bytes: 731
```

Rust conformance evidence:

```text
NEDITOR_TEST_PIKCHR=/Users/nyimbiodero/src/pjs/tooling/neditor/.tmp/pikchr-build/pikchr-trunk/pikchr cargo test --locked external_transform_conformance_runs_installed_engines --lib -- --nocapture
external transform conformance verified: dot, circo, neato, fdp, osage, twopi, d2, plantuml, pikchr; skipped:
```

Interpretation:

- Graphviz/DOT, Graphviz layout engines (`circo`, `neato`, `fdp`, `osage`,
  `twopi`), D2, PlantUML, Pikchr, and SQLite are verified on this macOS host
  through the standalone engine probe. Diagram engines are also verified
  through the same Rust external transform execution path used by NEditor, and
  SQLite SQL transform trust/read-only behavior is covered by focused Rust
  table transform tests.
- The standalone probe now produces inspectable SVG smoke artifacts for every
  installed diagram engine plus a CSV query artifact for SQLite, and fails if
  an installed engine cannot render the expected smoke output.
- External evidence templates now make copied Linux, Windows, or alternate
  macOS optional-engine proof auditable instead of relying on prose notes.
- PlantUML file-mode execution is verified locally and Java is available.
- Pikchr remains optional and unbundled; this host verifies it through
  `NEDITOR_TEST_PIKCHR` because the executable lives under `.tmp/pikchr-build/`
  rather than on `PATH`.

## Remaining Platform Evidence Gaps

- Refresh Linux installed-engine evidence locally when those engines are
  available outside retired remote workflows, or ingest the
  `neditor-optional-engine-evidence-linux` artifact from the current release
  evidence workflow.
- Add Windows evidence for Graphviz, D2, PlantUML, Java, Pikchr, and SQLite
  executable paths, including package-manager shims.
