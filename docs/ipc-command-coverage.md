# NEditor IPC Command Coverage

Updated: 2026-05-21

This table tracks the initial Tauri IPC command surface required by
`docs/specification.md` section 25.4. The Rust test
`ipc_command_tests::spec_25_4_ipc_commands_are_registered_and_documented`
keeps this table synchronized with the specification and with
`tauri::generate_handler!` in `src-tauri/src/lib.rs`.

| Command | Rust implementation | Current direct evidence |
| --- | --- | --- |
| `open_file` | `src-tauri/src/filesystem.rs` | `file_command_tests::stable_file_ipc_aliases_open_save_as_and_watch_paths` |
| `save_file` | `src-tauri/src/filesystem.rs` | `file_command_tests::save_file_rejects_stale_expected_hash` |
| `save_file_as` | `src-tauri/src/filesystem.rs` | `file_command_tests::stable_file_ipc_aliases_open_save_as_and_watch_paths` |
| `watch_file` | `src-tauri/src/filesystem_watch.rs` | `file_command_tests::stable_file_ipc_aliases_open_save_as_and_watch_paths` |
| `compile_document` | `src-tauri/src/compiler.rs` | `compiler_core_tests::compiler_resolves_metadata_variables_transforms_and_manifest` |
| `export_document` | `src-tauri/src/export_commands.rs` | `export_command_tests::export_document_writes_optional_sidecar_manifest` |
| `list_transform_engines` | `src-tauri/src/transforms/external.rs` | `external_transform_tests::external_transform_adapters_shape_engine_specific_invocations` |
| `run_transform` | `src-tauri/src/compiler.rs` | `transform_tests::transform_registry_covers_required_first_release_transforms` |
| `get_git_status` | `src-tauri/src/git.rs` | `file_command_tests::git_history_diff_commit_tag_and_restore_workflow` |
| `create_snapshot` | `src-tauri/src/snapshot.rs` | `snapshot_storage::tests::project_snapshot_gitignore_entry_is_idempotent` |
| `list_snapshots` | `src-tauri/src/snapshot.rs` | `snapshot_storage::tests::project_snapshot_gitignore_entry_is_idempotent` |
| `restore_snapshot` | `src-tauri/src/snapshot.rs` | `snapshot::tests::snapshot_restore_is_scoped_to_active_document_store`; `snapshot::tests::snapshot_restore_rejects_out_of_scope_and_mismatched_sources` |
