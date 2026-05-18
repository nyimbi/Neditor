# NEditor

NEditor is a local-first business Markdown workbench built with Tauri 2, Vue 3,
Pinia, CodeMirror 6, vanilla CSS, and a Rust command backend.

The implementation is driven by `docs/specification.md`. The current product
surface provides the app shell, tabs, CodeMirror editing, live preview, command
palette, front matter compilation, include expansion, TOC/index generation,
diagnostics, safe transform artifacts, Git status, app-data preference
persistence, workspace restore, sidebar folder browsing, pinned tabs, recently
closed document reopening, snapshots, AI paste cleanup, file
open/save/save-as/revert/rename/duplicate/reveal, export readiness validation,
and export commands for HTML, PDF, DOCX, PPTX, and Markdown bundle outputs.
Save operations include a final on-disk hash guard so external edits are not
silently overwritten between refresh polls.

The compiler currently handles YAML front matter, document variables, include
graphs, `calc`, `csv`, `tsv`, `json`, `yaml`, `glossary`, `timeline`, `chart`,
captured external transform artifacts, BibTeX, CSL JSON, citations, glossary
terms, automatic index terms, labels, cross-reference diagnostics, inline math,
numbered block equations, table summaries, release metadata validation, export
manifests, and source/include hashing.
Figure syntax with labels and captions renders as semantic figures, and local
image paths are validated with diagnostics. Citation parsing supports multiple
citation keys with locator text without treating locators as missing keys.
Markdown tables plus CSV and TSV transform tables evaluate simple formula cells
such as `=10+15` or `=SUM(2,3)`, and the transform registry covers the
first-release diagram/data set with native renderers or clear setup diagnostics.

External refresh checks the root document and captured include hashes. Clean
documents reload or recompile automatically; dirty documents keep local edits and
show compare, accept external, keep local, and save-copy actions instead of
overwriting user changes.

## Development

```sh
pnpm install
pnpm run build
./node_modules/.bin/tauri dev
```

## Verification

```sh
pnpm run build
cd src-tauri && cargo fmt --check
cd src-tauri && cargo check
cd src-tauri && cargo clippy --locked --all-targets -- -D warnings
cd src-tauri && cargo test
./node_modules/.bin/tauri build --bundles app
```

Run `pnpm run build` before desktop packaging; the Tauri build consumes the
generated project-local `dist` path instead of launching a nested package
manager command.

GitHub Actions also runs formatting, Rust check/test, clippy static analysis,
frontend build, and a `./node_modules/.bin/tauri build --no-bundle` desktop
compile on macOS, Windows, and Linux via `.github/workflows/ci.yml`.

`cargo check` requires access to crates.io the first time dependencies are
resolved. If network access is blocked, the frontend build and Rust formatting
can still be verified, but backend compilation remains unverified.

## Current Packaging Note

`./node_modules/.bin/tauri build --bundles app` succeeds on macOS and produces
`src-tauri/target/release/bundle/macos/NEditor.app`.

`./node_modules/.bin/tauri build --bundles dmg` currently reaches the `.app` bundle step and
then fails inside Tauri's generated `bundle_dmg.sh` without surfacing a useful
`hdiutil` sub-error in this environment.
