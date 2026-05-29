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
release-candidate checker syntax, Markdown links, spec-completion matrix,
table-editor manual review contract, Rust formatting, Rust dev check, and
whitespace checks.

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
pnpm run collect:evidence-kit
pnpm run check:evidence-kit
pnpm run check:release-readiness
```

`collect:evidence-kit` creates a sendable work packet under
`.tmp/release-evidence-kit/` with runbooks, templates, return paths, validator
commands, ingest commands, and final readiness commands. Send that directory to
supported-host owners or human reviewers when the local machine cannot produce
the needed evidence.

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

## Creating a Local Release Candidate

From a clean checkout:

```sh
pnpm run release:local
pnpm run check:release-candidate
```

`release:local` writes `.tmp/release-candidate/manifest.json`,
`.tmp/release-candidate/SHA256SUMS`, and a reviewer-facing README. It builds the
frontend and native release binaries, prepares the Tauri `ned` sidecar, refreshes
prerequisite evidence templates, validates the release evidence kit, reruns
release readiness, records compiled artifact hashes, and joins remaining
readiness gaps to evidence-kit work items.

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
hashes, README, required artifact kinds, file sizes, SHA-256 values, and prepared
sidecar hash parity. Do not hand off a candidate that fails this check.

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
