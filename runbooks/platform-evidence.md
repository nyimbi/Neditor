# Platform Evidence Runbook

Use this runbook for supported-host proof on Windows, Linux, and macOS.

## GitHub Actions Path

The fastest supported-host path is the release evidence workflow:

```sh
gh workflow run neditor-release-evidence.yml
gh run download <run-id> --name neditor-platform-evidence-win32-json --dir returned-evidence
gh run download <run-id> --name neditor-platform-evidence-linux-json --dir returned-evidence
pnpm run ingest:evidence -- --source returned-evidence
pnpm run check:platform-evidence
pnpm run check:release-readiness
```

The ingest command accepts the downloaded artifact directories as-is. You do
not need to flatten or rename `neditor-platform-evidence-win32-json` or
`neditor-platform-evidence-linux-json`.

## Windows

```sh
git fetch --all --tags
git checkout <source-commit>
git status --porcelain
pnpm install --frozen-lockfile
pnpm run build
./node_modules/.bin/tauri build --bundles all
pnpm run test:tauri-webdriver -- --strict
NEDITOR_PLATFORM_EVIDENCE_PLATFORM=win32 pnpm run collect:platform-evidence
pnpm run check:platform-evidence
```

Return:

- `.tmp/platform-evidence/external/win32/package-artifacts.json`
- `.tmp/platform-evidence/external/win32/tauri-webdriver-report.json`

## Linux

```sh
git fetch --all --tags
git checkout <source-commit>
git status --porcelain
pnpm install --frozen-lockfile
pnpm run build
./node_modules/.bin/tauri build --bundles all
pnpm run test:tauri-webdriver -- --strict
NEDITOR_PLATFORM_EVIDENCE_PLATFORM=linux pnpm run collect:platform-evidence
pnpm run check:platform-evidence
```

Return:

- `.tmp/platform-evidence/external/linux/package-artifacts.json`
- `.tmp/platform-evidence/external/linux/tauri-webdriver-report.json`

## macOS

```sh
pnpm run build
./node_modules/.bin/tauri build --no-bundle
pnpm run test:desktop-smoke
NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke
pnpm run test:tauri-webdriver
pnpm run check:release-readiness
```

## Rules

- Do not synthesize Windows or Linux evidence from macOS.
- Returned JSON must match the current package version and Git commit.
- Evidence must come from a clean tree.
