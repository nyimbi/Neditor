# Security Threat Model

NEditor is a local desktop workbench for business documents. Its primary
security objective is to keep document work local, explicit, reproducible, and
auditable while allowing carefully bounded file, export, Git, and transform
workflows.

## Assets

- Markdown source documents and included files.
- Draft snapshots and project-local recovery history.
- Exported HTML, PDF, DOCX, PPTX, and Markdown bundle artifacts.
- Export manifests and sidecar evidence.
- Bibliographies, media, variables, data transforms, and generated sections.
- User preferences, recent paths, external engine paths, and trust state.
- Git history and release tags for repositories the user opens.

## Trust Boundaries

| Boundary | Trusted side | Untrusted or lower-trust side | Controls |
| --- | --- | --- | --- |
| Tauri command boundary | Rust commands | Vue UI events and persisted settings | Stable command names, validation, typed request structures. |
| Filesystem boundary | User-selected paths | External disk edits and missing/moved files | Hash checks, stale-save blocking, conflict UI, snapshots before destructive recovery. |
| Include boundary | Root document | Included Markdown/media/data files | Include graph tracking, diagnostics, source hashes. |
| Export boundary | Compiler/export modules | Target artifact consumers | Readiness checks, manifests, output hashes, sidecar evidence. |
| Git boundary | Opened repository worktree | User-controlled tags, revisions, and worktree paths | Argument-array invocation, pathspec separators, ref validation, symlink restore blocking, repository-contained restore targets. |
| External transform boundary | Native transform fallback and trusted adapter | User-installed executables | Disabled by default, per-engine trust, executable checks, fixed adapters, no shell interpolation, timeout and output limits. |
| Persistence boundary | Versioned schema helper | Old or malformed `settings.json` data | Migration, clamping, invalid value filtering. |

## Threats And Mitigations

### Accidental Overwrite

Threat: A document changes on disk while the user is editing.

Mitigations:

- Save commands compare expected hashes.
- Stale saves are blocked.
- Clean documents reload external changes automatically.
- Dirty documents open a conflict flow.
- Conflict actions include accept external, keep local, save copy, and merge.

### Malicious Or Broken External Transform

Threat: A configured diagram executable reads unintended input, runs too long,
produces hostile output, or fails unpredictably.

Mitigations:

- External engines are disabled until explicitly trusted per engine.
- Engine paths must point to executable files.
- Adapters construct fixed argument lists and do not interpolate shell strings.
- Execution is bounded by timeout and output-size limits.
- Cache keys include source, engine path, input mode, and adapter behavior.
- Failed external execution falls back to native rendering when available.
- Diagnostics include setup, execution, stderr, timeout, and cache details.

### Unsafe Git Restore Or Tag Input

Threat: A malicious repository or malformed UI state turns a versioning action
into an unintended Git option or writes restored content outside the worktree.

Mitigations:

- Git commands are invoked with argument arrays, not interpolated shell strings.
- Path-oriented commands use `--` before pathspecs.
- Release tags and restore revisions reject option-shaped and unsupported ref
  syntax before invoking Git.
- Restore writes reject symlink targets.
- Restore target parents must resolve inside the Git repository root.

### Unsafe Persisted Preferences

Threat: Old or malformed persisted settings put the app in an unsafe or broken
state.

Mitigations:

- `src/lib/workspacePersistence.ts` owns the schema version.
- Invalid enum values are discarded.
- Numeric settings are clamped.
- Path arrays are deduplicated and capped.
- Transform trust/path/input-mode records are type-filtered.
- Legacy keys are migrated deliberately.

### Export Without Evidence

Threat: A business deliverable is produced without enough audit evidence to
recreate or inspect the export state.

Mitigations:

- Export readiness reports include diagnostics, source maps, layout sections,
  progress steps, and manifest previews.
- Export manifests include source and output hashes.
- Manifests are written beside artifacts when enabled.
- Markdown bundles carry their own `manifest.json`.
- Direct exports record dirty-Git warnings in readiness and manifests.

### Accidental Disclosure

Threat: Private drafts or local preferences are committed or shared.

Mitigations:

- Snapshots default to app data, not document folders.
- Project-local snapshots are opt-in.
- `.neditor/` is added to `.gitignore` when project-local snapshots are used.
- Export manifests are explicit sidecars beside deliverables.

## Explicit Non-Goals

- NEditor is not a sandbox for arbitrary untrusted executables.
- NEditor does not provide cloud access control.
- NEditor does not encrypt source documents or exported artifacts.
- NEditor does not verify third-party engine supply chains.
- NEditor does not send telemetry or sync documents automatically.

## Verification Evidence

- `external_transforms_are_trust_gated_and_limited`
- `external_transform_rejects_non_executable_engine_path`
- `external_transform_timeout_covers_blocked_stdin`
- `external_transform_exit_errors_include_stderr`
- `compiler_falls_back_when_external_transform_is_untrusted`
- `export_document_writes_optional_sidecar_manifest`
- `export_document_manifest_records_dirty_git_warning`
- `git_restore_and_tag_reject_option_shaped_refs`
- `git_restore_refuses_symlink_targets`
- `workspace persistence migration versions and normalizes saved settings`
- `save_file_rejects_stale_expected_hash`
