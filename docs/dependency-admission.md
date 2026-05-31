# Dependency Admission Record

This record satisfies the dependency admission gate in `docs/specification.md`.

`pnpm run check:deps` verifies that every dependency declared in `package.json`
and `src-tauri/Cargo.toml` appears in this record, that each recorded license
expectation is permissive, and that the repository package metadata still
declares MIT licensing. Add or update a row here before adding a manifest
dependency.

## JavaScript Runtime Dependencies

| Package | Version | License expectation | Purpose | Packaging | Simpler in-house alternative | Export/security impact |
| --- | --- | --- | --- | --- | --- | --- |
| `vue` | `^3.5.13` | MIT | UI framework required by the specification | Bundled frontend | Not appropriate; framework is required | Runs in Tauri WebView |
| `pinia` | `^3.0.4` | MIT | Application state stores required by the specification | Bundled frontend | Hand-rolled reactive store would add risk | No direct export/security impact |
| `@tauri-apps/api` | `^2` | MIT/Apache-2.0 | Frontend IPC bridge to Rust commands | Bundled frontend | Required for Tauri IPC | Security follows Tauri command allowlist |
| `@tauri-apps/plugin-dialog` | `^2.7.1` | MIT/Apache-2.0 | Native open/save dialogs | Bundled frontend/Rust plugin | Handwritten dialogs are not native | File paths only from explicit user action |
| `@tauri-apps/plugin-fs` | `^2.5.1` | MIT/Apache-2.0 | Scoped frontend file support | Bundled frontend/Rust plugin | Rust-only file commands cover complex flows | Must keep scopes explicit |
| `@tauri-apps/plugin-shell` | `^2.3.5` | MIT/Apache-2.0 | Future scoped transform engine execution | Bundled frontend/Rust plugin | Rust `Command` is used for backend paths | Requires command allowlists |
| `@tauri-apps/plugin-store` | `^2.4.3` | MIT/Apache-2.0 | Preferences and UI state persistence | Bundled frontend/Rust plugin | JSON sidecar store possible but less integrated | Local app data only |
| `@tauri-apps/plugin-window-state` | `^2.4.1` | MIT/Apache-2.0 | Native window state persistence | Bundled frontend/Rust plugin | Custom Rust state service possible | Local app data only |
| `@tauri-apps/plugin-opener` | `^2` | MIT/Apache-2.0 | Open/reveal generated artifacts | Bundled frontend/Rust plugin | Platform-specific shell calls | User-triggered only |
| `@codemirror/state` | `^6.6.0` | MIT | Editor state engine | Bundled frontend | Building an editor is out of scope | Local document text only |
| `@codemirror/view` | `^6.43.0` | MIT | Editor rendering | Bundled frontend | Building an editor is out of scope | Local document text only |
| `@codemirror/commands` | `^6.10.3` | MIT | Editing commands/history | Bundled frontend | In-house command engine would add risk | Local document text only |
| `@codemirror/language` | `^6.12.3` | MIT | Language extension support | Bundled frontend | Required for Markdown tooling | Local document text only |
| `@codemirror/lang-markdown` | `^6.5.0` | MIT | Markdown syntax support | Bundled frontend | Required by CodeMirror Markdown use | Local document text only |
| `@codemirror/search` | `^6.7.0` | MIT | Find/search keymaps | Bundled frontend | In-house search possible but lower quality | Local document text only |
| `@codemirror/autocomplete` | `^6.20.2` | MIT | Future snippets/completions | Bundled frontend | Could defer; useful for commands/snippets | Local document text only |
| `@codemirror/lint` | `^6.9.6` | MIT | Future diagnostics gutter integration | Bundled frontend | Could render diagnostics manually | Local document text only |

## JavaScript Development Dependencies

| Package | Version | License expectation | Purpose | Packaging | Simpler in-house alternative | Export/security impact |
| --- | --- | --- | --- | --- | --- | --- |
| `@tauri-apps/cli` | `^2` | MIT/Apache-2.0 | Tauri development/build CLI | Dev-only | Required by Tauri | Build-time only |
| `@playwright/test` | `^1.60.0` | Apache-2.0 | Browser workflow testing for the Vite-rendered workbench | Dev-only | Manual browser QA or DOM-only tests would miss interaction regressions | Test-only; the runner prefers local Playwright Chromium and falls back to an installed Chrome-compatible browser when needed |
| `@vitejs/plugin-vue` | `^5.2.1` | MIT | Vue SFC compilation | Dev-only | Required for Vue/Vite | Build-time only |
| `typescript` | `~5.6.2` | Apache-2.0 | Type checking | Dev-only | Not appropriate | Build-time only |
| `vite` | `^6.0.3` | MIT | Frontend build server/bundler | Dev-only | Required by scaffold | Build-time only |
| `vue-tsc` | `^2.1.10` | MIT | Vue type checking | Dev-only | Not appropriate | Build-time only |

## Rust Dependencies

| Package | Version | License expectation | Purpose | Packaging | Simpler in-house alternative | Export/security impact |
| --- | --- | --- | --- | --- | --- | --- |
| `tauri` | `2` | MIT/Apache-2.0 | Native desktop runtime | Linked application core | Required by specification | Main IPC/security boundary |
| `tauri-build` | `2` | MIT/Apache-2.0 | Tauri build integration | Build dependency | Required by Tauri | Build-time only |
| `tauri-plugin-dialog` | `2` | MIT/Apache-2.0 | Native dialogs | Linked plugin | Required for native dialogs | User-triggered file paths |
| `tauri-plugin-fs` | `2` | MIT/Apache-2.0 | Scoped filesystem support and native file watching for root/include refresh | Linked plugin | Rust commands cover complex flows; polling is less responsive for watch workflows | Scope carefully; watcher events stay local |
| `tauri-plugin-opener` | `2` | MIT/Apache-2.0 | Open/reveal artifacts | Linked plugin | Platform-specific code possible | User-triggered only |
| `tauri-plugin-shell` | `2` | MIT/Apache-2.0 | Scoped external engine support | Linked plugin | Rust `Command` possible | Requires no shell interpolation |
| `tauri-plugin-store` | `2` | MIT/Apache-2.0 | Preferences storage | Linked plugin | JSON store possible | Local app data only |
| `tauri-plugin-window-state` | `2` | MIT/Apache-2.0 | Window state persistence | Linked plugin | Small custom store possible | Local app data only |
| `serde` | `1` | MIT/Apache-2.0 | Typed IPC serialization | Linked library | Manual JSON is error-prone | IPC schema correctness |
| `serde_json` | `1` | MIT/Apache-2.0 | JSON values/manifests | Linked library | Manual JSON is error-prone | Export manifest correctness |
| `serde_yaml` | `0.9` | MIT/Apache-2.0 | YAML front matter parsing | Linked library | In-house YAML parser is not practical | Local document metadata only |
| `chrono` | `0.4` | MIT/Apache-2.0 | Export/snapshot timestamps | Linked library | SystemTime formatting manually possible | Metadata only |
| `pulldown-cmark` | `0.13` | MIT | Markdown-to-HTML preview/export | Linked library | In-house Markdown parser is not practical | HTML output must escape safely |
| `sha2` | `0.10` | MIT/Apache-2.0 | Source/include/export hashes | Linked library | In-house SHA-256 is inappropriate | Manifest integrity |
| `getrandom` | `0.2` | MIT/Apache-2.0 | OS-backed OAuth state and PKCE verifier entropy | Linked library | Timestamp/counter-derived tokens are predictable; platform-specific `/dev/urandom` code would weaken Windows support | Prevents predictable OAuth callback state and verifier material |
| `zip` | `2.4` | MIT | Minimal DOCX/PPTX package writing | Linked library | Handwriting ZIP is inappropriate | Export artifact integrity |
| `notify` | `8.2.0` | CC0-1.0 | Rust watcher backend for root/include refresh when the `native-watch` feature is enabled | Optional linked library, disabled by default | Tauri fs watcher remains the default UI path; Rust watcher gives the backend an event source for packaged/native workflows | Local filesystem events only; verify with `cargo check --locked --features native-watch` |
