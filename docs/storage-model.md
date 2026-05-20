# Storage Model

NEditor is local-first. Source documents, preferences, snapshots, exports, and
evidence sidecars stay on the user's machine unless the user places them in a
synced folder or repository.

## Storage Locations

| Data | Default location | Purpose | Portability |
| --- | --- | --- | --- |
| Source documents | User-chosen files and folders | Markdown source of truth | Portable by normal file copy/Git. |
| Workspace preferences | Tauri store `settings.json` | Theme, typography, export defaults, recents, transform paths, trust state, open tabs, scroll positions | Local machine state. |
| Automatic snapshots | App data snapshot store | Recovery history for drafts and includes | Local by default. |
| Project-local snapshots | `.neditor/` when enabled | Team-portable recovery history | Explicit opt-in. |
| Export artifacts | User-chosen output paths | HTML, PDF, DOCX, PPTX, Markdown bundle | Portable deliverables. |
| Export manifests | `<output>.manifest.json` when enabled | Hashes, options, readiness, diagnostics, source map, layout, progress evidence | Portable sidecar evidence. |
| Markdown bundle manifest | `manifest.json` inside the bundle zip | Bundle-local export evidence | Travels with the bundle. |

## Workspace Schema

Workspace preferences are normalized through `src/lib/workspacePersistence.ts`
before the Pinia store applies them.

- `WORKSPACE_SCHEMA_VERSION` identifies the current persisted shape.
- Legacy aliases such as `workspacePath` and `activeFile` are migrated to
  `workspaceRoot` and `activePath`.
- Numeric settings are clamped before use.
- Recent paths are deduplicated and capped.
- Transform path, trust, and input-mode records discard invalid values.
- Scroll positions are normalized to `0..1`.
- Export, bibliography, brand, Git, and AI cleanup defaults are filled with
  explicit defaults.

The unit test `workspace persistence migration versions and normalizes saved
settings` proves this schema boundary without requiring Tauri or a real store.

## Snapshot Policy

Automatic snapshots default to app data so ordinary editing does not create
hidden sidecar directories beside every document. Project-local snapshots are
available when a team deliberately wants portable recovery evidence.

When project-local snapshots are enabled, NEditor adds `.neditor/` to the
project `.gitignore` idempotently so private draft history is not accidentally
committed.

Snapshot metadata records:

- Source hash.
- Timestamp.
- Document version and status when present.
- Author when present.
- Include graph hash.

## Export Sidecars

Export manifests are sidecar evidence files written next to exported artifacts
when `includeManifest` is true. The path is deterministic:

```text
<output-path>.manifest.json
```

For example, exporting `board-pack.pdf` writes:

```text
board-pack.pdf
board-pack.pdf.manifest.json
```

The manifest includes:

- Document title, version, status, and source hash.
- Export target, options, output path, and output hash.
- Included file hashes.
- Transform artifact metadata.
- Readiness summary and diagnostics.
- Source map and layout sections.
- Progress steps for compile, transform, readiness, render, and manifest work.

`export_document_writes_optional_sidecar_manifest` proves sidecar behavior for
HTML, PDF, DOCX, PPTX, and Markdown bundle exports, and proves no sidecar is
written when `includeManifest` is false.

## Retention And Deletion

NEditor does not automatically upload or sync stored data. Deletion follows the
storage location:

- Delete source documents and exports from their filesystem location.
- Clear preferences by removing the app `settings.json` store.
- Remove app-data snapshots from the app data snapshot directory.
- Remove project-local snapshots by deleting `.neditor/`.
- Remove export evidence by deleting the exported artifact and its manifest
  sidecar.

## Non-Goals

- No cloud document storage.
- No background telemetry.
- No automatic external sync.
- No hidden network persistence for AI or transform workflows.
