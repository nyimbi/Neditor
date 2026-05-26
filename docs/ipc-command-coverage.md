# NEditor IPC Command Coverage

Updated: 2026-05-22

This table tracks the full registered Tauri IPC command surface, including the
initial commands required by `docs/specification.md` section 25.4 and later
native workflow commands added during the buildout. The Rust test
`ipc_command_tests::spec_25_4_ipc_commands_are_registered_and_documented`
keeps this table synchronized with the specification, with
`tauri::generate_handler!` in `src-tauri/src/lib.rs`, and with the evidence
rows below so new native commands cannot quietly bypass the coverage ledger.

| Command | Rust implementation | Current direct evidence |
| --- | --- | --- |
| `open_file` | `src-tauri/src/filesystem.rs` | `file_command_tests::stable_file_ipc_aliases_open_save_as_and_watch_paths` |
| `read_file` | `src-tauri/src/filesystem.rs` | `file_command_tests::stable_file_ipc_aliases_open_save_as_and_watch_paths`; `file_command_tests::file_duplicate_and_rename_commands_move_content` |
| `save_file_as` | `src-tauri/src/filesystem.rs` | `file_command_tests::stable_file_ipc_aliases_open_save_as_and_watch_paths` |
| `save_file` | `src-tauri/src/filesystem.rs` | `file_command_tests::save_file_rejects_stale_expected_hash` |
| `watch_file` | `src-tauri/src/filesystem_watch.rs` | `file_command_tests::stable_file_ipc_aliases_open_save_as_and_watch_paths` |
| `start_file_watcher` | `src-tauri/src/filesystem_watch.rs` | `file_command_tests::stable_file_ipc_aliases_open_save_as_and_watch_paths`; `ipc_command_tests::spec_25_4_ipc_commands_are_registered_and_documented` |
| `stop_file_watcher` | `src-tauri/src/filesystem_watch.rs` | `ipc_command_tests::spec_25_4_ipc_commands_are_registered_and_documented` |
| `rename_file` | `src-tauri/src/filesystem.rs` | `file_command_tests::file_duplicate_and_rename_commands_move_content` |
| `duplicate_file` | `src-tauri/src/filesystem.rs` | `file_command_tests::file_duplicate_and_rename_commands_move_content` |
| `reveal_path` | `src-tauri/src/filesystem.rs` | `ipc_command_tests::spec_25_4_ipc_commands_are_registered_and_documented` |
| `file_metadata` | `src-tauri/src/filesystem.rs` | `file_command_tests::file_duplicate_and_rename_commands_move_content`; `file_command_tests::stable_file_ipc_aliases_open_save_as_and_watch_paths` |
| `pending_cli_open_paths` | `src-tauri/src/cli.rs` | `cli_tests::ned_cli_opens_markdown_paths_without_subcommand`; desktop launch smoke covers CLI-open path handoff |
| `default_markdown_reader_plan` | `src-tauri/src/cli.rs` | `cli_tests::ned_cli_doctor_reports_json_capabilities`; Settings copy uses the same default-reader plan contract |
| `configure_default_markdown_reader` | `src-tauri/src/cli.rs` | `cli_tests::ned_cli_doctor_reports_json_capabilities`; Settings checkbox invokes the guarded platform default-reader plan |
| `create_support_bundle` | `src-tauri/src/cli.rs` | `cli_tests::ned_cli_creates_redaction_safe_support_bundles`; Settings support-bundle action invokes the same redaction-safe JSON contract |
| `list_workspace_files` | `src-tauri/src/workspace_files.rs` | `file_command_tests::workspace_listing_skips_hidden_and_build_artifacts` |
| `compile_document` | `src-tauri/src/compiler.rs` | `compiler_core_tests::compiler_resolves_metadata_variables_transforms_and_manifest` |
| `compile_document_with_options` | `src-tauri/src/compiler.rs` | `export_option_tests::compile_options_supply_brand_profile_defaults`; `export_option_tests::compile_options_do_not_override_document_brand_profile` |
| `export_document` | `src-tauri/src/export_commands.rs` | `export_command_tests::export_document_writes_optional_sidecar_manifest` |
| `prepare_for_export` | `src-tauri/src/export_commands.rs` | `export_command_tests::prepare_for_export_carries_broad_readiness_audit_to_manifest`; `export_command_tests::prepare_for_export_validates_target_and_options` |
| `import_spreadsheet_table` | `src-tauri/src/data_exchange.rs` | `table_tests::spreadsheet_table_import_export_round_trips_csv_and_xlsx` |
| `export_markdown_tables` | `src-tauri/src/data_exchange.rs` | `table_tests::spreadsheet_table_import_export_round_trips_csv_and_xlsx` |
| `import_rfp_source` | `src-tauri/src/rfp_import.rs` | `tests/frontend-unit.test.ts` static RFP import wiring; RFP wizard source import workflows are covered by focused browser/native smoke gates |
| `prepare_local_agent_handoff` | `src-tauri/src/local_agents.rs` | `tests/frontend-unit.test.ts` static local-agent handoff wiring; Agent Workspace provider handoff workflows are covered by focused browser/native smoke gates |
| `create_snapshot` | `src-tauri/src/snapshot.rs` | `snapshot_storage::tests::project_snapshot_gitignore_entry_is_idempotent`; `snapshot::tests::snapshot_restore_is_scoped_to_active_document_store` |
| `list_snapshots` | `src-tauri/src/snapshot.rs` | `snapshot_storage::tests::project_snapshot_gitignore_entry_is_idempotent`; `snapshot::tests::snapshot_restore_is_scoped_to_active_document_store` |
| `restore_snapshot` | `src-tauri/src/snapshot.rs` | `snapshot::tests::snapshot_restore_is_scoped_to_active_document_store`; `snapshot::tests::snapshot_restore_rejects_out_of_scope_and_mismatched_sources` |
| `get_git_status` | `src-tauri/src/git.rs` | `file_command_tests::git_history_diff_commit_tag_and_restore_workflow` |
| `git_history` | `src-tauri/src/git.rs` | `file_command_tests::git_history_diff_commit_tag_and_restore_workflow` |
| `git_diff` | `src-tauri/src/git.rs` | `file_command_tests::git_history_diff_commit_tag_and_restore_workflow` |
| `commit_document_changes` | `src-tauri/src/git.rs` | `file_command_tests::git_history_diff_commit_tag_and_restore_workflow` |
| `tag_release` | `src-tauri/src/git.rs` | `file_command_tests::git_history_diff_commit_tag_and_restore_workflow`; `file_command_tests::git_restore_and_tag_reject_option_shaped_refs` |
| `restore_git_revision` | `src-tauri/src/git.rs` | `file_command_tests::git_history_diff_commit_tag_and_restore_workflow`; `file_command_tests::git_restore_and_tag_reject_option_shaped_refs`; `file_command_tests::git_restore_refuses_symlink_targets` |
| `list_transform_engines` | `src-tauri/src/transforms/external.rs` | `external_transform_tests::external_transform_adapters_shape_engine_specific_invocations` |
| `list_transform_handler_installers` | `src-tauri/src/transform_install.rs` | `transform_install_tests::transform_handler_installer_plans_cover_supported_platforms` |
| `install_transform_handlers` | `src-tauri/src/transform_install.rs` | `transform_install_tests::transform_handler_install_steps_are_allowlisted` |
| `run_transform` | `src-tauri/src/compiler.rs` | `transform_tests::transform_registry_covers_required_first_release_transforms`; `transform_tests::transform_aliases_render_through_canonical_artifacts` |
| `run_external_transform` | `src-tauri/src/transforms/external.rs` | `external_transform_tests::external_transform_adapters_shape_engine_specific_invocations`; `external_transform_tests::external_transform_cache_invalidates_when_trusted_executable_changes` |
| `cleanup_ai_paste` | `src-tauri/src/ai_cleanup.rs` | `ai_cleanup_tests::ai_cleanup_normalizes_chat_artifacts`; `ai_cleanup_tests::ai_cleanup_converts_rich_html_clipboard_content`; `ai_cleanup_tests::ai_cleanup_normalizes_ai_code_fence_variants` |
| `inspect_native_tts` | `src-tauri/src/tts.rs` | `tts_tests::tts_inspection_reports_browser_and_configured_native_engines_without_launching` |
| `download_tts_model` | `src-tauri/src/tts.rs` | `tts_tests::tts_model_download_command_requires_explicit_acknowledgement` |
| `read_text_aloud` | `src-tauri/src/tts.rs` | `tts_tests::tts_command_builders_use_argument_safe_native_engines`; `tts_tests::macos_say_reads_text_via_stdin_instead_of_shell_interpolation` on macOS |
| `stop_text_aloud` | `src-tauri/src/tts.rs` | `tts_tests::tts_command_builders_use_argument_safe_native_engines`; `ipc_command_tests::spec_25_4_ipc_commands_are_registered_and_documented` |
| `write_desktop_ui_smoke_report` | `src-tauri/src/lib.rs` | `pnpm run test:desktop-smoke` with `NEDITOR_DESKTOP_SMOKE_LAUNCH=1` validates the guarded native UI smoke report when `NEDITOR_DESKTOP_UI_SMOKE_REPORT` is set |
| `desktop_workflow_smoke_enabled` | `src-tauri/src/lib.rs` | `pnpm run test:desktop-smoke` with `NEDITOR_DESKTOP_SMOKE_LAUNCH=1` validates the guarded app-authored native workflow smoke gate |
| `desktop_workflow_smoke_autorun_enabled` | `src-tauri/src/lib.rs` | `pnpm run test:desktop-smoke` with `NEDITOR_DESKTOP_SMOKE_LAUNCH=1` validates the guarded app-authored native workflow autorun gate |
| `desktop_workflow_smoke_file_path` | `src-tauri/src/lib.rs` | `pnpm run test:desktop-smoke` with `NEDITOR_DESKTOP_SMOKE_LAUNCH=1` validates deterministic native workflow Markdown/include file paths under `.tmp/desktop-smoke/` |
| `desktop_workflow_smoke_named_path` | `src-tauri/src/lib.rs` | `pnpm run test:tauri-webdriver` records the supported-platform plan for deterministic WebDriver rename and duplicate Markdown paths under `.tmp/desktop-webdriver/` |
| `desktop_workflow_smoke_export_path` | `src-tauri/src/lib.rs` | `pnpm run test:desktop-smoke` with `NEDITOR_DESKTOP_SMOKE_LAUNCH=1` validates deterministic native workflow export and conflict-copy paths under `.tmp/desktop-smoke/` |
| `emit_desktop_workflow_smoke_menu_command` | `src-tauri/src/lib.rs` | `pnpm run test:desktop-smoke` with `NEDITOR_DESKTOP_SMOKE_LAUNCH=1` validates guarded native menu-event routing for `File` -> `Export` -> `HTML Export` |
| `write_desktop_workflow_smoke_report` | `src-tauri/src/lib.rs` | `pnpm run test:desktop-smoke` with `NEDITOR_DESKTOP_SMOKE_LAUNCH=1` validates the guarded native workflow report when `NEDITOR_DESKTOP_WORKFLOW_SMOKE_REPORT` is set |
