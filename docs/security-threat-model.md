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
- Session-only AI provider keys entered into the Agent Workspace.

## Trust Boundaries

| Boundary | Trusted side | Untrusted or lower-trust side | Controls |
| --- | --- | --- | --- |
| Tauri command boundary | Rust commands | Vue UI events and persisted settings | Stable command names, validation, typed request structures. |
| Filesystem boundary | User-selected paths | External disk edits and missing/moved files | Hash checks, stale-save blocking, conflict UI, snapshots before destructive recovery. |
| Snapshot boundary | Active document snapshot store | Arbitrary paths, stale metadata, snapshots for another document | Store-root scoping, Markdown-only restore files, required metadata, active source-path matching. |
| Include boundary | Root document | Included Markdown/media/data files | Include graph tracking, diagnostics, source hashes. |
| Export boundary | Compiler/export modules | Target artifact consumers | Readiness checks, manifests, output hashes, sidecar evidence. |
| Git boundary | Opened repository worktree | User-controlled tags, revisions, and worktree paths | Argument-array invocation, pathspec separators, ref validation, symlink restore blocking, repository-contained restore targets. |
| External transform boundary | Native transform fallback and trusted adapter | User-installed executables | Disabled by default, per-engine trust, executable checks, fixed adapters, no shell interpolation, timeout and output limits. |
| AI provider boundary | Local agent packet and session key field | Approved external HTTPS providers or local model gateways | User-entered endpoint/model, redacted request package, session-only API key, response preview, human-review apply flow. |
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

### Unsafe SQL Transform

Threat: A document-authored SQL block mutates a local database or chains a
read-only query with a destructive statement.

Mitigations:

- SQL transforms require an explicitly trusted `sqlite3` executable.
- SQL blocks must use a document-local `database` path that resolves inside the
  document folder.
- Only `SELECT` and `WITH` queries are accepted.
- Stacked statements such as `SELECT ...; DELETE ...` are rejected before
  `sqlite3` is invoked.
- Mutation keywords are detected on SQL word boundaries while quoted strings
  and mutation-like identifier names remain usable as read-only values.
- The SQLite process is invoked directly without a shell and with a bounded
  timeout.

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

### Unsafe Snapshot Restore

Threat: A malformed UI event or stale snapshot list restores content from a
different document or from outside the configured snapshot store.

Mitigations:

- Snapshot restore takes a typed request with active file path and storage mode.
- Restored files must be Markdown snapshots inside the configured snapshot root.
- Matching JSON metadata is required before content is returned.
- Snapshot metadata `sourcePath` must match the active saved document when one
  exists; saved-document snapshots cannot be restored into an unsaved document.

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

### Unsafe AI Provider Execution

Threat: A document run sends sensitive text or API credentials to an
unapproved model endpoint, or imports generated text as if it were already
reviewed.

Mitigations:

- Agent Workspace first builds a redacted provider request package for review.
- API keys are typed into a session-only password field and are not written to
  Markdown, preferences, snapshots, or manifests.
- Provider endpoint and model remain visible before execution.
- Provider responses are previewed as Markdown before being applied.
- Applied responses are routed to Review and remain human-review material.
- The desktop content-security policy allows user-approved HTTPS providers and
  local model gateways while preserving `object-src 'none'` and
  `frame-ancestors 'none'`.

## Explicit Non-Goals

- NEditor is not a sandbox for arbitrary untrusted executables.
- NEditor does not provide cloud access control.
- NEditor does not encrypt source documents or exported artifacts.
- NEditor does not verify third-party engine supply chains.
- NEditor does not send telemetry or sync documents automatically.
- NEditor does not persist AI provider API keys or certify provider compliance.

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
- `snapshot_restore_is_scoped_to_active_document_store`
- `snapshot_restore_rejects_out_of_scope_and_mismatched_sources`
- `sql_transform_requires_read_only_trusted_queries`
- `sql_transform_blocks_document_relative_database_escape`
- `workspace persistence migration versions and normalizes saved settings`
- `save_file_rejects_stale_expected_hash`
- `AI provider packages redact secrets and preserve agent governance context`
- `AI provider execution extracts Markdown without persisting secrets`
- `pnpm run check:security-review` emits the independent review evidence
  template and validates returned reviewer scope, findings, report hashes, and
  release sign-off before security review can satisfy release readiness.
