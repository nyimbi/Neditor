# NEditor Release Evidence Workflow

Workflow file: `.github/workflows/neditor-release-evidence.yml`
Trigger: manual (`workflow_dispatch`).

> **Two-workflow release model**: this workflow is the **pre-release gate** —
> it collects cross-platform evidence that the build is correct and the native
> workflows pass.  The **ship step** is the separate publish workflow
> (`.github/workflows/neditor-release-publish.yml`, documented at
> `docs/release-publish-workflow.md`).  Run evidence first; push the tag to
> trigger publish once evidence is accepted.

## Required secrets

Configure all secrets at **Settings → Secrets and variables → Actions** in the
GitHub repository before dispatching the workflow.  Jobs that cannot find a
secret emit a warning and continue; the resulting build will be unsigned and the
evidence-kit validation step will surface any gaps.

| Secret name | Used by | Purpose |
|---|---|---|
| `APPLE_ID` | `platform-proof-macos` | Apple ID (email) used to submit for notarization via `xcrun notarytool`. Required for the notarization staple verification step and signing evidence collection. |
| `APPLE_ID_PASSWORD` | `platform-proof-macos` | App-specific password for the Apple ID above (generated at appleid.apple.com). Forwarded to Tauri as `APPLE_PASSWORD`. |
| `APPLE_TEAM_ID` | `platform-proof-macos` | Ten-character Apple Developer Team ID. Used by Tauri for both the signing identity and notarytool team flag. |
| `APPLE_CERT_P12_BASE64` | `platform-proof-macos` | Base64-encoded Apple Developer ID Application certificate + private key in PKCS#12 format. Generate with: `base64 -i DeveloperIDApplication.p12`. Forwarded to Tauri as `APPLE_CERTIFICATE` and imported into a temporary keychain. |
| `APPLE_CERT_PASSWORD` | `platform-proof-macos` | Passphrase protecting `APPLE_CERT_P12_BASE64`. Forwarded to Tauri as `APPLE_CERTIFICATE_PASSWORD`. |
| `WINDOWS_CERT_PFX_BASE64` | `platform-proof` (Windows leg) | Base64-encoded Authenticode code-signing certificate in PFX/PKCS#12 format. Generate with: `base64 -w0 NEditorSign.pfx`. Forwarded to Tauri as `WINDOWS_CERTIFICATE`. |
| `WINDOWS_CERT_PASSWORD` | `platform-proof` (Windows leg) | Passphrase protecting `WINDOWS_CERT_PFX_BASE64`. Forwarded to Tauri as `WINDOWS_CERTIFICATE_PASSWORD`. |
| `GOOGLE_DRIVE_OAUTH_JSON` | `evidence-kit` | OAuth 2.0 credentials JSON for the Google Drive service account used by `collect:evidence-kit` to upload the signed bundle. Optional — the kit is still assembled and validated locally if absent. |
| `AI_PROVIDER_API_KEY` | `evidence-kit` | API key for the AI provider used by evidence-kit enrichment steps (forwarded as `NEDITOR_AI_PROVIDER_API_KEY`). Optional — kit assembly proceeds without AI enrichment if absent. |

## Jobs

| Job | Runner | WebDriver | Closes spec-matrix rows |
|---|---|---|---|
| `browser-workflows` | `ubuntu-latest` | Playwright/Chromium (browser, not native) | Browser workflow cluster |
| `platform-proof` (Windows) | `windows-latest` | `msedgedriver` + `tauri-driver` | `windows-linux-tauri-webdriver-execution`, `windows-package-artifact-proof` |
| `platform-proof` (Linux) | `ubuntu-latest` | `WebKitWebDriver` + `tauri-driver` under `xvfb-run` | `windows-linux-tauri-webdriver-execution`, `linux-package-artifact-proof` |
| `platform-proof-macos` | `macos-latest` | None (WebDriver unsupported on darwin — see [run-tauri-webdriver.mjs](../scripts/run-tauri-webdriver.mjs) lines 139–147) | `macos-native-launch-current-binary-proof`, `macos-native-window-visibility-proof`, `release-signing-and-notarization/darwin` |
| `optional-engine-proof` | Windows + Linux | — | Optional engine cluster |
| `rendered-export-review` | `ubuntu-latest` | — | Rendered export cluster |
| `accessibility-review` | `ubuntu-latest` | — | Accessibility cluster |
| `evidence-kit` | `ubuntu-latest` | — | `cross-platform-ci-evidence`, `native-workflow-bundle-100-assertions` |

## v2 WebDriver report contract

`scripts/check-platform-evidence.mjs` rejects reports with
`schema !== "neditor.tauri-webdriver-report.v2"`.  The script
`scripts/run-tauri-webdriver.mjs` emits v2 reports.  The key checks are:

- `nativeWorkflowArtifacts.assertionCount >= 100`
- `nativeWorkflowArtifacts.status === "passed"`
- All named assertions present and passed (snapshot restore, Markdown list
  continuation, bracket pairing, Emacs/Vim keybindings, table editor / source
  map, HTML export, and others — see `requiredWebdriverAssertions` in the
  checker script).

## Self-hosted runners

The workflow uses GitHub-hosted runners by default.  To use self-hosted runners,
replace the `runs-on` values:

```yaml
# platform-proof matrix:
os: windows-latest  →  self-hosted-windows-x64
os: ubuntu-latest   →  self-hosted-ubuntu-x64
# platform-proof-macos:
runs-on: macos-latest  →  self-hosted-macos-arm64  (or x86_64)
```

Ensure each self-hosted runner has Rust (stable), pnpm, Node 22, and the
platform's WebDriver dependencies pre-installed (see job steps for the exact
package list).

## Open TODOs

- **Windows cert format**: verify whether your Tauri version reads
  `WINDOWS_CERTIFICATE` as a raw base64 blob or requires a decoded `.pfx` file
  on disk (see comment in `Import Windows signing certificate` step).
- **macOS keychain import**: if Tauri's internal certificate import is
  sufficient, remove the `Import Apple signing certificate into keychain` and
  `Delete temporary keychain` steps to reduce attack surface.
- **macOS WebDriver**: if a future Tauri release adds Safari WebDriver support
  on darwin, add the darwin platform spec to
  `scripts/check-platform-evidence.mjs` and replace the smoke-test steps with
  WebDriver steps.
- **Linux GPG signing**: the Linux leg signs deb/rpm/AppImage packages via
  `collect:release-signing`.  Add a `gpg --import` step and
  `GPG_SIGNING_KEY_BASE64` / `GPG_SIGNING_KEY_PASSWORD` secrets when that
  runbook is finalised.
