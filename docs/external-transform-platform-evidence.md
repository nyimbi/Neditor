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

The probe reports installed/missing optional engines without failing solely
because an optional engine is absent. For installed engines, it also renders a
small SVG smoke artifact through the same adapter shape NEditor uses and writes
those artifacts under `.tmp/external-engines/artifacts/`.

The same command now writes external evidence templates under
`.tmp/external-engines/templates/`. Copy a completed evidence file into
`.tmp/external-engines/external/<engine>.json`, or point
`NEDITOR_EXTERNAL_ENGINE_EVIDENCE_DIR` at another directory, when a supported
host has an optional engine that is not installed locally. Supplied evidence is
validated for schema, engine key, status, timestamp, platform, command, version,
SVG smoke bytes, smoke hash, required text markers, and zero unresolved
blockers. Malformed supplied evidence fails the probe; absent optional engines
without copied proof remain explicit release-readiness gaps.

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
```

Rust conformance evidence:

```text
NEDITOR_TEST_PIKCHR=/Users/nyimbiodero/src/pjs/tooling/neditor/.tmp/pikchr-build/pikchr-trunk/pikchr cargo test --locked external_transform_conformance_runs_installed_engines --lib -- --nocapture
external transform conformance verified: dot, circo, neato, fdp, osage, twopi, d2, plantuml, pikchr; skipped:
```

Interpretation:

- Graphviz/DOT, Graphviz layout engines (`circo`, `neato`, `fdp`, `osage`,
  `twopi`), D2, PlantUML, and Pikchr are verified on this macOS host through
  the standalone engine probe and through the same Rust external transform
  execution path used by NEditor.
- The standalone probe now produces inspectable SVG smoke artifacts for every
  installed engine and fails if an installed engine cannot render the expected
  smoke output.
- External evidence templates now make copied Linux, Windows, or alternate
  macOS optional-engine proof auditable instead of relying on prose notes.
- PlantUML file-mode execution is verified locally and Java is available.
- Pikchr remains optional and unbundled; this host verifies it through
  `NEDITOR_TEST_PIKCHR` because the executable lives under `.tmp/pikchr-build/`
  rather than on `PATH`.

## Remaining Platform Evidence Gaps

- Refresh Linux installed-engine evidence locally when those engines are
  available outside retired remote workflows.
- Add Windows evidence for Graphviz, D2, PlantUML, Java, and Pikchr executable
  paths, including package-manager shims.
