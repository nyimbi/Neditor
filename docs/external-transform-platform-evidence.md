# External Transform Platform Evidence

Updated: 2026-05-21

NEditor keeps external transform engines optional. This page records current
platform evidence for configured local engines and the repeatable command used
to refresh that evidence.

## Local Probe Command

```sh
pnpm run check:engines
```

The probe reports installed/missing optional engines without failing solely
because an optional engine is absent. Use:

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
  version: dot - graphviz version 14.1.5 (20260411.2331)
Graphviz / circo: installed
  command: circo
  path: /opt/homebrew/bin/circo
  version: circo - graphviz version 14.1.5 (20260411.2331)
Graphviz / neato: installed
  command: neato
  path: /opt/homebrew/bin/neato
  version: neato - graphviz version 14.1.5 (20260411.2331)
Graphviz / fdp: installed
  command: fdp
  path: /opt/homebrew/bin/fdp
  version: fdp - graphviz version 14.1.5 (20260411.2331)
Graphviz / osage: installed
  command: osage
  path: /opt/homebrew/bin/osage
  version: osage - graphviz version 14.1.5 (20260411.2331)
Graphviz / twopi: installed
  command: twopi
  path: /opt/homebrew/bin/twopi
  version: twopi - graphviz version 14.1.5 (20260411.2331)
D2: installed
  command: d2
  path: /opt/homebrew/bin/d2
  version: 0.7.1
PlantUML: installed
  command: plantuml
  path: /opt/homebrew/bin/plantuml
  version: PlantUML version 1.2026.3 / 208af3a [2026-05-08 16:11:48 UTC]
Pikchr: missing
  command: pikchr or pikchr-cli
  note: Set NEDITOR_TEST_PIKCHR to an absolute executable path to force a probe.
```

Rust conformance evidence:

```text
cargo test --locked external_transform_conformance_runs_installed_engines --lib -- --nocapture
external transform conformance verified: dot, circo, neato, fdp, osage, twopi, d2, plantuml; skipped: pikchr
```

Interpretation:

- Graphviz/DOT, Graphviz layout engines (`circo`, `neato`, `fdp`, `osage`,
  `twopi`), D2, and PlantUML are verified on this macOS host through the same
  external transform execution path used by NEditor.
- PlantUML file-mode execution is verified locally and Java is available.
- Pikchr remains a macOS evidence gap on this host because neither `pikchr` nor
  `pikchr-cli` is installed.

## Remaining Platform Evidence Gaps

- Install and verify Pikchr on macOS.
- Refresh Linux installed-engine evidence locally when those engines are
  available outside retired remote workflows.
- Add Windows evidence for Graphviz, D2, PlantUML, Java, and Pikchr executable
  paths, including package-manager shims.
