# NEditor Release Publish Workflow

Workflow file: `.github/workflows/neditor-release-publish.yml`
Trigger: `v*.*.*` tag push, or manual `workflow_dispatch`.

This is the **ship step**. It builds signed platform artifacts, creates a
GitHub Release, and auto-bumps the Homebrew cask tap.
The **pre-release gate** is the separate evidence workflow
(`.github/workflows/neditor-release-evidence.yml`); run that first.

## Cutting a release

1. Bump `version` in `package.json` and `src-tauri/tauri.conf.json` to the
   same value (e.g. `0.1.1`).
2. Optionally write release notes to `docs/release-notes/v0.1.1.md`; the
   workflow uses that file if it exists, otherwise GitHub auto-generates notes
   from the commit range.
3. Commit, tag, and push:
   ```sh
   git add package.json src-tauri/tauri.conf.json
   git commit -m "chore: release v0.1.1"
   git tag v0.1.1
   git push && git push --tags
   ```
4. The publish workflow starts automatically on the tag push.

## Dry-run flow

Trigger manually with `dry_run: true` to build and sign all artifacts without
creating a GitHub Release.  Useful for verifying signing credentials or testing
the matrix build.

```sh
gh workflow run neditor-release-publish.yml \
  --field dry_run=true \
  --ref v0.1.1
```

## Jobs

| Job | Runner | Depends on | Purpose |
|---|---|---|---|
| `guard` | ubuntu | — | Version-consistency check + `check:release-readiness` |
| `build-macos` | macos-latest | guard | Universal DMG, Apple sign + notarize |
| `build-windows` | windows-latest | guard | MSI + NSIS, Authenticode sign |
| `build-linux` | ubuntu-latest | guard | deb + AppImage, GPG-sign |
| `publish` | ubuntu | all builds | SHA256SUMS + `gh release create` |
| `homebrew` | ubuntu | publish | Cask auto-bump on GA signed releases only |

## GA vs pre-release

The `publish` job marks the release GA only when **both** macOS and Windows
signing secrets are present and signing succeeded.  Missing Linux GPG keys are
tolerated.  Missing Apple or Windows credentials produce a pre-release instead;
signing can be added later by re-running the workflow with the secrets
populated.

## Required secrets

Configure at **Settings → Secrets and variables → Actions**.

| Secret | Job | Purpose |
|---|---|---|
| `APPLE_ID` | `build-macos` | Apple ID email for notarytool submission |
| `APPLE_ID_PASSWORD` | `build-macos` | App-specific password for the Apple ID |
| `APPLE_TEAM_ID` | `build-macos` | 10-character Apple Developer Team ID |
| `APPLE_CERT_P12_BASE64` | `build-macos` | Base64-encoded Developer ID Application P12. Generate: `base64 -i DeveloperIDApplication.p12` |
| `APPLE_CERT_PASSWORD` | `build-macos` | Passphrase protecting the P12 |
| `WINDOWS_CERT_PFX_BASE64` | `build-windows` | Base64-encoded Authenticode PFX. Generate: `base64 -w0 NEditorSign.pfx` |
| `WINDOWS_CERT_PASSWORD` | `build-windows` | Passphrase protecting the PFX |
| `GPG_SIGNING_KEY_BASE64` | `build-linux` | Base64-encoded GPG private key for Linux artifact signatures (optional) |
| `GPG_SIGNING_KEY_PASSWORD` | `build-linux` | Passphrase for the GPG key (optional) |
| `HOMEBREW_TAP_TOKEN` | `homebrew` | PAT with `repo` scope on the tap repo (`nyimbi/homebrew-neditor`) |
| `TAURI_SIGNING_PRIVATE_KEY` | all builds | Tauri updater Ed25519 private key |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | all builds | Passphrase for the Tauri updater key |

No secret is ever interpolated into a log line.  Decoded values are masked
with `::add-mask::` before use; all secrets pass through `env:` scoping only.

## Homebrew tap configuration

The tap repository defaults to `nyimbi/homebrew-neditor`.  Override at
dispatch time with the `tap_repo` workflow input.

The `homebrew` job:
1. Downloads the signed universal DMG from the `publish` job artifacts.
2. Runs `pnpm run release:homebrew -- --artifact <dmg>` to materialise the
   cask at `.tmp/homebrew/external/neditor.rb`.
3. Validates the cask with `pnpm run check:homebrew`.
4. Checks out the tap repo using `HOMEBREW_TAP_TOKEN`.
5. Copies the cask to `Casks/neditor.rb` in the tap.
6. Either commits directly (`auto_merge_cask: true`) or opens a PR
   (`auto_merge_cask: false`, the default).

The `homebrew` job is skipped on pre-releases and dry runs.

## Open TODOs (human decision required)

- **`src-tauri/entitlements.plist`**: the explicit codesign step in
  `build-macos` applies `--entitlements` only when this file exists.  Create a
  minimal entitlements plist if your app requires hardened-runtime entitlements
  (e.g. `com.apple.security.network.client`).
- **Windows double-signing**: verify whether your Tauri version already signs
  via `WINDOWS_CERTIFICATE`; if so, remove the `Sign MSI + NSIS with signtool`
  step to avoid signing twice.
- **macOS keychain import**: if Tauri's internal certificate import suffices,
  remove the `Import Apple signing certificate into keychain` and
  `Delete temporary keychain` steps to reduce attack surface.

## `pnpm ship` — local single-command release

`scripts/ship-release.mjs` automates the full local release path in one command:

```sh
# Dry-run first (fast — no builds, no pushes)
pnpm ship -- --dry-run --skip-preflight

# Full release from main with signing
APPLE_ID=you@example.com \
APPLE_PASSWORD=app-specific-pw \
APPLE_TEAM_ID=XXXXXXXXXX \
pnpm ship -- --sign

# With Homebrew tap update
pnpm ship -- --sign --homebrew
```

### Steps performed

| # | Step | Notes |
|---|------|-------|
| 1 | Preflight | Tree clean, branch=main, versions match, `check:release-readiness` |
| 2 | Compile | macOS: cross-compile `ned` for arm64+x86_64, `lipo` universal, `pnpm tauri build --bundles dmg --target universal-apple-darwin` |
| 3 | Sign/notarize | macOS only; pass `APPLE_ID`/`APPLE_PASSWORD`/`APPLE_TEAM_ID` env vars — Tauri picks them up automatically.  Skipped with a warning when `APPLE_ID` is unset; fails hard with `--sign`. |
| 4 | Verify | Mount DMG, `lipo -info`, PlistBuddy version check, codesign probe, SHA256SUMS |
| 5 | Tag | `git tag vX.Y.Z && git push origin vX.Y.Z`; skips if tag already at correct commit |
| 6 | Release | `gh release create vX.Y.Z` with notes from `docs/release-notes/vX.Y.Z.md`; placeholder auto-generated if missing |
| 7 | Homebrew | Clone/create `nyimbi/homebrew-neditor`, run `pnpm release:homebrew`, push cask.  **Off by default; requires `--homebrew` flag.** |

### CLI flags

| Flag | Effect |
|------|--------|
| `--dry-run` | Print every command without executing; skip tag/release/homebrew pushes |
| `--sign` | Hard-fail if `APPLE_ID` unset |
| `--homebrew` | Enable step 7 |
| `--skip-preflight` | Skip step 1 (for testing) |

### Windows / Linux

Run `pnpm ship` on a Windows or Linux host — the script detects `process.platform` and
invokes the appropriate Tauri bundle targets (`msi,nsis` / `deb,appimage`).
These targets are skipped automatically when run on an incompatible host.

### Open TODOs (operator action required)

- **Apple certs**: set `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`, and optionally
  `APPLE_SIGNING_IDENTITY` before running `--sign`.  Import the Developer ID Application
  p12 into your keychain first (`security import DeveloperIDApplication.p12`).
- **Windows host**: MSI/NSIS bundles must be built on a Windows machine or a
  `windows-latest` CI runner — `cross` cannot cross-compile the Tauri Windows bundler.
- **Linux host**: deb/AppImage must be built on an Ubuntu-compatible host.
- **GPG signing** (Linux): not automated in this script; sign `.deb`/`.AppImage`
  manually with `gpg --detach-sign` and upload the `.sig` files to the release.
- **`src-tauri/entitlements.plist`**: add if your app needs hardened-runtime
  entitlements (`com.apple.security.network.client`, etc.); Tauri will pass it
  to `codesign` automatically via `tauri.conf.json`.
- **`HOMEBREW_TAP_TOKEN`**: set to a GitHub PAT with `repo` scope on
  `nyimbi/homebrew-neditor` if the current `gh` auth doesn't cover the tap.
