# Compiling and Releasing NEditor

This guide is the release-operator path for turning a checked-out NEditor source
tree into verified desktop artifacts, a packaged `ned` command-line helper, and
a release-candidate evidence packet. It complements the user-facing README and
the Homebrew runbook; when the instructions disagree, treat executable scripts
in `package.json`, `scripts/`, and `src-tauri/tauri.conf.json` as the source of
truth.

## Release Principles

- Release from a clean Git checkout. Uncommitted source or evidence changes make
  provenance ambiguous and should block final release-candidate generation.
- Compile both surfaces: the Vue/Vite frontend and the Tauri/Rust desktop app.
  The desktop app is incomplete without the `ned` sidecar prepared for the
  current target triple.
- Do not publish unsigned, unnotarized, or SHA-placeholder artifacts as public
  releases. Local builds are useful for testing, not distribution.
- Treat release readiness as evidence-driven. A green build is necessary, but a
  releasable package also needs browser, platform, signing, export, accessibility,
  security, performance, and human-review evidence where applicable.
- Keep generated evidence under `.tmp/` out of source control unless a specific
  file is intentionally documented as a fixture.

## Supported Outputs

NEditor produces these release outputs from one source tree:

| Output | Primary command | Expected location |
| --- | --- | --- |
| Frontend static bundle | `pnpm run build` | `dist/` |
| Desktop app binary | `cargo build --manifest-path src-tauri/Cargo.toml --locked --release` | `src-tauri/target/release/neditor` |
| `ned` CLI binary | `cargo build --manifest-path src-tauri/Cargo.toml --locked --release --bin ned` | `src-tauri/target/release/ned` |
| Tauri `ned` sidecar | `pnpm run prepare:sidecars` | `src-tauri/binaries/ned-<target-triple>` |
| Tauri desktop package | `pnpm tauri build` or `./node_modules/.bin/tauri build` | `src-tauri/target/release/bundle/` |
| Local release candidate | `pnpm run release:local` | `.tmp/release-candidate/` |

On Windows, the `ned` binary and sidecar include `.exe`.

## Prerequisites

Install the platform toolchain before compiling:

- Node.js compatible with this repository's pnpm setup.
- `pnpm` matching `package.json`.
- Rust stable with `cargo`, `rustc`, `rustfmt`, and `clippy`.
- Tauri platform prerequisites for the target OS.
- A browser usable by Playwright. The repository prefers
  `.tmp/ms-playwright`, but can fall back to an installed Chrome-compatible
  browser.
- macOS release hosts need Xcode command line tools, code-signing identity, and
  notarization credentials for distributable macOS artifacts.
- Windows release hosts need Authenticode signing credentials and a timestamp
  server for distributable Windows installers.
- Linux release hosts need package-signing/checksum tooling appropriate to the
  package type being published.

Install JavaScript dependencies:

```sh
pnpm install
```

Refresh the project-local Chromium cache when the host does not already have a
usable Playwright browser:

```sh
PLAYWRIGHT_BROWSERS_PATH=.tmp/ms-playwright pnpm exec playwright install chromium
```

## Source Preparation

Before compiling a release candidate:

```sh
git status --short
pnpm run verify:local -- --list
pnpm run verify:local:full -- --list
```

The two `--list` commands print the quick and full verification sequences
without running them. Use the list output to decide whether the host can produce
all required evidence or whether some evidence must come from supported Windows,
Linux, macOS, or credentialed release machines.

Version metadata must remain synchronized across:

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

`pnpm run check:platform-packaging` enforces this contract and also verifies the
bundle targets, Markdown file associations, icon coverage, CSP guardrails, and
the packaged `ned` sidecar configuration.

## Development Compile

Use this loop when validating source changes before release packaging:

```sh
pnpm run check
pnpm run test:unit
pnpm run build
cargo check --manifest-path src-tauri/Cargo.toml --locked
```

Run the desktop app in development mode:

```sh
pnpm tauri dev
```

Development mode uses Vite and the Tauri dev shell. It proves the workbench can
run locally, but it does not prove release packaging, sidecar preparation,
signing, notarization, or installer behavior.

## Production Compile

Build the frontend:

```sh
pnpm run build
```

Build native release binaries without packaging:

```sh
cargo build --manifest-path src-tauri/Cargo.toml --locked --release
cargo build --manifest-path src-tauri/Cargo.toml --locked --release --bin ned
```

Prepare the `ned` sidecar for Tauri packaging:

```sh
pnpm run prepare:sidecars
```

The sidecar preparation script compiles `ned`, copies it into
`src-tauri/binaries/ned-<target-triple>`, marks it executable on Unix-like
hosts, rejects unexpectedly small binaries, and runs `ned --version` to confirm
the sidecar matches `package.json`.

Build the Tauri desktop package:

```sh
pnpm tauri build
```

For a faster native compile without creating package bundles:

```sh
./node_modules/.bin/tauri build --no-bundle
```

For a macOS `.app` bundle only:

```sh
./node_modules/.bin/tauri build --bundles app
```

The Tauri config uses `beforeBuildCommand` to run
`pnpm run prepare:sidecars && pnpm run build` before packaging, so a normal
Tauri release build should refresh both the sidecar and frontend bundle.

## Verification Levels

Use the smallest meaningful verification during development and the full gate
for release candidates.

### Quick Verification

```sh
pnpm run verify:local
```

Quick verification covers the browser environment check, frontend typecheck,
unit tests, project structure, accessibility guard, dependency admission,
external-transform documentation, AI/provider/runtime evidence contracts,
Homebrew packaging contract, sidecar script syntax, platform packaging
configuration, release CI workflow guard, release-candidate script syntax,
release-candidate checker syntax, Markdown links, spec-completion matrix, spec
manual-review evidence contract, table-editor manual review contract, Rust
formatting, Rust dev check, and whitespace checks.

### Full Verification

```sh
pnpm run verify:local:full
```

Full verification adds the production build, full browser workflow suite,
runtime accessibility audit, manual accessibility contract, optional engine
probe, native-watch Rust check, clippy, Rust tests, rendered export audit,
Google Docs import evidence contract, AI provider evidence contract, performance
profile contract, platform evidence contract, release-signing contract, release
evidence-kit generation/checking, Tauri release compile, host-specific desktop
bundle checks, desktop smoke tests, WebDriver smoke, and release-readiness
aggregation.

Run focused checks while diagnosing failures instead of repeatedly running the
full suite:

```sh
node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "workflow name" --project chromium
pnpm run check:e2e-env
pnpm run check:release-readiness
pnpm run check:evidence-kit
pnpm run check:release-candidate
```

## Release Evidence

Release readiness is aggregated from machine-generated and human-returned
evidence under `.tmp/`. The central commands are:

```sh
pnpm run check:spec-completion
pnpm run check:manual-review
pnpm run collect:evidence-kit
pnpm run check:evidence-kit
pnpm run check:release-readiness
```

`collect:evidence-kit` creates a sendable work packet under
`.tmp/release-evidence-kit/` with runbooks, templates, return paths, validator
commands, ingest commands, and final readiness commands. Send that directory to
supported-host owners or human reviewers when the local machine cannot produce
the needed evidence. After `pnpm run check:manual-review`, the kit also includes
the spec manual-review dashboard at `manual-review/dashboard.html` and
`manual-review/dashboard.md`, an assignment CSV at
`manual-review/assignments.csv`, and all generated work-order sign-off
templates under `templates/spec-manual-review/`.

To produce a single Markdown handoff for tickets, reviewers, or credentialed
operators, run:

```sh
ned evidence-packet --output release-evidence-return-packet.md
```

The packet combines release evidence work items, spec/manual work orders,
recognized ingest paths, validator commands, redaction rules, and closure
commands without including document content or secrets.

When evidence returns from another host, ingest it:

```sh
pnpm run ingest:evidence -- --source /path/to/unpacked-returned-evidence
pnpm run check:evidence-kit
pnpm run check:release-readiness
```

Typical returned evidence includes:

- Windows/Linux package and WebDriver evidence.
- macOS signing and notarization evidence.
- Windows Authenticode/timestamp evidence.
- Linux package-signing/checksum evidence.
- Google Docs import/readback evidence.
- Live AI provider evidence.
- Security review evidence.
- Release-device performance profile evidence.
- Accessibility and rendered-export human sign-offs.
- Optional external-transform engine evidence.

Release readiness rejects stale evidence when it was collected for a different
source commit or when the current source tree is dirty.

### AI Runtime Device Evidence

The AI runtime gap must be closed from a real browser or packaged Tauri WebView
session that can request microphone permission and exercise clipboard read/write
without storing content. On the device host, generate the readiness input
template:

```sh
pnpm run collect:ai-runtime -- --write-template
```

Run the Docs Live **Check AI runtime** action, save the JSON-shaped readiness
result using the template shape in
`.tmp/ai-runtime-evidence/templates/runtime-readiness.template.json`, and then
collect validator-ready evidence:

```sh
pnpm run collect:ai-runtime -- \
  --readiness-json /path/to/runtime-readiness.json \
  --microphone-result stream-opened \
  --clipboard-write-succeeded true
pnpm run check:ai-runtime
```

The collector requires a clean Git tree, records only capability states and
character counts, sets `audioStored: false` and clipboard `contentStored:
false`, and refuses to store audio or clipboard material in release evidence.
Return `.tmp/ai-runtime-evidence/external/runtime-evidence.json` through the
release evidence kit or ingest it into the release checkout.

### Release-Device Performance Evidence

The performance-profile gap must be closed with a real release-device run, not
with synthetic metrics. On the profiling host, first generate the metrics input
template:

```sh
pnpm run collect:performance-profile -- --write-template
```

Run the packaged release app for at least 30 minutes and record the required
startup/open, large-document edit/preview, export-suite, file-watch conflict,
and Agent Workspace review scenarios in
`.tmp/performance-profile/templates/native-profile-metrics.template.json` or a
copy of that file. Then collect hashed evidence from the metrics and profiler
artifacts:

```sh
pnpm run collect:performance-profile -- \
  --metrics /path/to/native-profile-metrics.json \
  --summary-artifact /path/to/profiler-summary.txt \
  --trace-artifact /path/to/profiler-trace.json \
  --reviewer-name "Reviewer Name"
pnpm run check:performance-profile
```

The collector requires a clean Git tree, hashes the profiled release binary and
profiler artifacts, writes
`.tmp/performance-profile/external/native-profile.json`, and leaves validation
to `check:performance-profile`. Return that JSON through the release evidence
kit or ingest it into the release checkout.

## Creating a Local Release Candidate

From a clean checkout:

```sh
pnpm run release:local
pnpm run check:release-candidate
```

`release:local` writes `.tmp/release-candidate/manifest.json`,
`.tmp/release-candidate/SHA256SUMS`, and a reviewer-facing README. It builds the
frontend and native release binaries, prepares the Tauri `ned` sidecar, refreshes
prerequisite evidence templates, materializes the Tauri app bundle so packaged
resources such as the capability showcase are present, validates the release
evidence kit, reruns release readiness, records compiled artifact hashes, and
joins remaining readiness gaps to evidence-kit work items.

Useful operator flags:

```sh
node scripts/create-release-candidate.mjs --skip-build
node scripts/create-release-candidate.mjs --allow-dirty
node scripts/create-release-candidate.mjs --skip-prerequisite-evidence
node scripts/create-release-candidate.mjs --refresh-browser-evidence
node scripts/create-release-candidate.mjs --refresh-native-launch-evidence
```

Use `--skip-build` only after a fresh compile. Use `--allow-dirty` only for a
non-releaseable dry run. Use `--skip-prerequisite-evidence` only when evidence
templates are already current for the exact commit. Browser and native launch
refreshes are opt-in because some CI, headless, or sandboxed hosts cannot launch
real windows reliably.

`check:release-candidate` independently validates the candidate manifest,
hashes, README, required artifact kinds, file sizes, SHA-256 values, prepared
sidecar hash parity, clean-checkout provenance, current Git commit, and
`releaseable: true` handoff status. Do not hand off a candidate that fails this
check. For local inspection of an old or deliberately dirty dry-run packet only,
use:

```sh
node scripts/check-release-candidate.mjs --allow-nonreleaseable
```

That flag still validates hashes and artifact structure, but the generated
`check-report.json` records that the result is not suitable for release handoff.

## Signing and Notarization

Local builds default to an `unsigned-local-builds` stance. That is acceptable for
developer testing and unacceptable for public distribution.

Run the signing evidence contract:

```sh
pnpm run check:release-signing
```

On credentialed release hosts, collect proof for signed artifacts:

```sh
pnpm run collect:release-signing -- --artifact kind=/path/to/artifact --proof kind='verification command'
```

Examples of proof commands include:

- macOS: `codesign --verify --deep --strict /path/to/NEditor.app`
- macOS: `spctl --assess --type execute /path/to/NEditor.app`
- macOS: `xcrun stapler validate /path/to/NEditor.app`
- Windows: `signtool verify /pa /tw /path/to/NEditor.exe`
- Linux: `gpg --verify /path/to/package.sig /path/to/package`

Copy returned signing evidence into the release evidence flow and rerun:

```sh
pnpm run check:release-signing
pnpm run check:release-readiness
```

## Platform Packaging Evidence

The repository can validate cross-platform package configuration from any host:

```sh
pnpm run check:platform-packaging
```

Actual package evidence must come from the host that produced the package. On a
Windows or Linux release host, build the package, run the desktop WebDriver
smoke, then collect evidence:

```sh
pnpm tauri build
pnpm run test:tauri-webdriver
pnpm run collect:platform-evidence
```

Copy `.tmp/platform-evidence/external/<platform>/` back to the release checkout
or return it through the evidence kit. Validate it with:

```sh
pnpm run check:platform-evidence
```

On macOS, also run the app-bundle and DMG checks when applicable:

```sh
./node_modules/.bin/tauri build --bundles app
pnpm run test:desktop-bundle
pnpm run test:desktop-dmg
NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke
```

## Independent Security Review Evidence

The security-review release gate requires a real independent review. The local
collector does not perform the review; it packages reviewer-supplied report and
tool-output hashes into the strict evidence schema.

After the independent reviewer has covered the Tauri command boundary,
filesystem/snapshot/include/export/Git boundaries, external transform execution,
AI provider boundary, persistence migration, and release evidence contracts, run:

```sh
pnpm run collect:security-review -- \
  --report-file /path/to/security-review-report.md \
  --tool-output-file /path/to/scanner-output.txt \
  --reviewer-name "Reviewer Name" \
  --reviewer-organization "Independent Org"
pnpm run check:security-review
```

The collector requires a clean Git tree, records zero critical/high/unresolved
findings, accepts at most three medium findings, hashes the reviewer report and
optional tool output, and writes
`.tmp/security-review/external/security-review.json`. Return that JSON through
the release evidence kit or ingest it into the release checkout.

## Rendered Export Human Signoff Evidence

The rendered-export release gate requires a reviewer to open the primary export
artifacts and every generated review-case artifact in native or browser viewers.
Generate the audit bundle and strict signoff template first:

```sh
pnpm run test:rendered-exports
```

After the reviewer has collected native-viewer screenshots, notes, or ticket
references, package the signoff:

```sh
pnpm run collect:rendered-exports:manual -- \
  --reviewer-name "Reviewer Name" \
  --reviewer-platform "macOS 15.5" \
  --native-viewer "Preview" \
  --native-viewer "Microsoft Word" \
  --evidence-reference /path/to/rendered-export-review-artifacts \
  --notes "Reviewed primary and review-case export artifacts with no blockers."
NEDITOR_RENDERED_EXPORT_SIGNOFF=.tmp/rendered-export-audit/external/visual-review-signoff.json \
  pnpm run test:rendered-exports -- --validate-signoff-only
```

The collector requires a clean Git tree and uses
`.tmp/rendered-export-audit/visual-review-signoff.template.json` so artifact
paths, hashes, source commit, and current app version stay aligned.

## Accessibility Human Signoff Evidence

The accessibility release gate requires real assistive-technology review across
screen-reader navigation, keyboard-only operation, native shell traversal, and
export artifact review. Generate the strict template after static and runtime
accessibility checks:

```sh
pnpm run check:a11y
pnpm run check:a11y:runtime
pnpm run check:a11y:manual
```

After the reviewer has completed the assistive-technology sessions and collected
artifact references, package the signoff:

```sh
pnpm run collect:a11y:manual -- \
  --reviewer-name "Reviewer Name" \
  --platform-version "macOS 15.5" \
  --platform-device "MacBook Pro" \
  --assistive-technology "VoiceOver" \
  --assistive-technology-version "macOS 15.5" \
  --browser-or-webview "Tauri WebView" \
  --browser-or-webview-version "WebKit 620" \
  --evidence-reference /path/to/accessibility-review-artifacts \
  --notes "Reviewed screen-reader, keyboard, native shell, and export artifact workflows with no blockers."
NEDITOR_ACCESSIBILITY_SIGNOFF=.tmp/accessibility/external/manual-review-signoff.json \
  pnpm run check:a11y:manual
```

The collector requires a clean Git tree and uses the generated
`.tmp/accessibility/manual-review-template.json` so prerequisite report hashes,
source commit, and current app version stay aligned.

## Table Editor Human Signoff Evidence

The table-editor release gate requires a named human review of source-to-grid,
grid-to-source, concurrent edit protection, spreadsheet exchange, rendered
exports, keyboard/accessibility behavior, and supported-host results. Generate
the strict template first:

```sh
pnpm run check:tables:manual
```

After the reviewer has completed the table sessions and collected artifact
references, package the signoff:

```sh
pnpm run collect:tables:manual -- \
  --reviewer-name "Reviewer Name" \
  --platform-version "macOS 15.5" \
  --platform-device "MacBook Pro" \
  --webview-or-browser "Tauri WebView" \
  --evidence-reference /path/to/table-review-artifacts \
  --notes "Reviewed source/grid/export workflows with no blockers."
NEDITOR_TABLE_EDITOR_SIGNOFF=.tmp/table-editor/external/manual-review-signoff.json pnpm run check:tables:manual
```

The collector requires a clean Git tree and uses the generated
`.tmp/table-editor/manual-review-template.json` so prerequisite report hashes,
source commit, and current app version stay aligned.

## Homebrew Release

Use a cask, not a formula. The template lives at
`packaging/homebrew/Casks/neditor.rb.template`, and the detailed Homebrew runbook
is [Homebrew Distribution](homebrew-distribution.md).

Before publishing a Homebrew cask:

```sh
pnpm run check:homebrew
```

After producing a signed and notarized macOS artifact, replace template
placeholders with the final version and SHA-256:

```sh
pnpm run release:homebrew -- \
  --artifact /path/to/NEditor-<version>-macos.zip \
  --output /path/to/homebrew-neditor/Casks/neditor.rb
NEDITOR_HOMEBREW_CASK=/path/to/homebrew-neditor/Casks/neditor.rb \
NEDITOR_HOMEBREW_ARTIFACT=/path/to/NEditor-<version>-macos.zip \
pnpm run check:homebrew
```

`release:homebrew` computes the SHA-256, writes the concrete cask, copies the
artifact into the Homebrew evidence directory, and records
`.tmp/homebrew/materialize-cask-report.json`. It does not replace
signing/notarization or release-readiness gates. Return that JSON report with
the cask and artifact so `check:homebrew` can verify that the cask SHA, artifact
SHA, and materialization record agree.

When Homebrew is available on the release host:

```sh
brew audit --cask --new /path/to/homebrew-neditor/Casks/neditor.rb
brew install --cask /path/to/homebrew-neditor/Casks/neditor.rb
ned --version
brew uninstall --cask neditor
```

Do not publish a cask that points at an unsigned local build or uses a
placeholder SHA.

## GitHub Release Checklist

Use this checklist after compile, verification, signing, and release-candidate
checks pass:

1. Confirm `git status --short` is clean.
2. Confirm version metadata is synchronized.
3. Confirm `pnpm run verify:local:full` passed or every host-specific gap has
   accepted returned evidence.
4. Confirm `pnpm run check:release-readiness` has no local failures and no
   unowned evidence gaps.
5. Confirm `pnpm run release:local` and `pnpm run check:release-candidate`
   passed for the final commit and artifacts.
6. Confirm signed artifacts match the hashes recorded in
   `.tmp/release-candidate/SHA256SUMS`.
7. Confirm Homebrew cask SHA and URL point to the final signed macOS artifact.
8. Tag the release from the verified commit.
9. Upload signed installers/packages, checksums, release notes, and the release
   candidate manifest.
10. Archive `.tmp/release-candidate/` and accepted external evidence with the
    release record.

## Troubleshooting

| Symptom | Action |
| --- | --- |
| `Release candidates must be created from a clean Git worktree` | Commit or stash source changes before final release. Use `--allow-dirty` only for dry runs. |
| `release-evidence-kit ... stale-for-current-source-commit` | Rerun `pnpm run collect:evidence-kit`, `pnpm run check:evidence-kit`, and `pnpm run check:release-readiness` after the latest commit. |
| Browser workflow cannot launch Chromium | Run `pnpm run check:e2e-env`; refresh `.tmp/ms-playwright` with the Playwright install command; use returned CI evidence if the host cannot run browsers. |
| Playwright port collision | Stop the previous Vite/Playwright run or rerun focused workflows sequentially. |
| `ned` sidecar missing from a package | Run `pnpm run prepare:sidecars`, confirm `src-tauri/binaries/ned-<target-triple>` exists, then rebuild the Tauri package. |
| `check:homebrew` reports placeholder SHA | Run `pnpm run release:homebrew -- --artifact /path/to/NEditor-<version>-macos.zip --output /path/to/homebrew-neditor/Casks/neditor.rb`, then rerun with `NEDITOR_HOMEBREW_CASK` and `NEDITOR_HOMEBREW_ARTIFACT`. |
| Signing evidence remains missing | Collect evidence on a credentialed host with `collect:release-signing`, ingest it, then rerun signing and readiness checks. |
| Full verification is expensive on battery | Run focused checks while developing, then run full verification once per coherent release slice or on a powered/CI host. |

## Do Not Release If

- The release candidate was generated from a dirty checkout.
- `pnpm run check:release-candidate` fails.
- The package contains a placeholder or stale `ned` sidecar.
- Signing, notarization, Authenticode, or package-signing evidence is missing
  for a public distribution artifact.
- Homebrew cask URL or SHA does not match the final signed artifact.
- Browser, platform, export, accessibility, security, or performance evidence is
  stale for the release commit.
- Release notes hide known evidence gaps or unsupported host limitations.
