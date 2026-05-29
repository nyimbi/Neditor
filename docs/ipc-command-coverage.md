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
| `stop_file_watcher` | `src-tauri/src/filesystem_watch.rs` | `filesystem_watch::tests::stop_file_watcher_clears_active_watcher_state`; `ipc_command_tests::spec_25_4_ipc_commands_are_registered_and_documented` |
| `rename_file` | `src-tauri/src/filesystem.rs` | `file_command_tests::file_duplicate_and_rename_commands_move_content` |
| `duplicate_file` | `src-tauri/src/filesystem.rs` | `file_command_tests::file_duplicate_and_rename_commands_move_content` |
| `copy_data_source_file` | `src-tauri/src/filesystem.rs` | `file_command_tests::copy_data_source_file_is_binary_safe_project_relative_and_collision_safe` |
| `reveal_path` | `src-tauri/src/filesystem.rs` | `file_command_tests::reveal_command_for_existing_path_is_platform_specific_and_argument_safe`; `ipc_command_tests::spec_25_4_ipc_commands_are_registered_and_documented` |
| `file_metadata` | `src-tauri/src/filesystem.rs` | `file_command_tests::file_duplicate_and_rename_commands_move_content`; `file_command_tests::stable_file_ipc_aliases_open_save_as_and_watch_paths` |
| `pending_cli_open_paths` | `src-tauri/src/cli.rs` | `cli_tests::ned_cli_opens_markdown_paths_without_subcommand`; desktop launch smoke covers CLI-open path handoff |
| `cli_deploy_plan` | `src-tauri/src/cli.rs` | `cli_tests::deploy_cli_installs_user_level_ned_launcher_without_overwriting_conflicts`; Settings and File menu expose the same guarded deployment plan |
| `deploy_cli` | `src-tauri/src/cli.rs` | `cli_tests::deploy_cli_installs_user_level_ned_launcher_without_overwriting_conflicts`; Settings Deploy CLI button and native File -> Deploy CLI menu route to the guarded deployment command |
| `default_markdown_reader_plan` | `src-tauri/src/cli.rs` | `cli_tests::ned_cli_doctor_reports_json_capabilities`; Settings copy uses the same default-reader plan contract |
| `configure_default_markdown_reader` | `src-tauri/src/cli.rs` | `cli_tests::ned_cli_doctor_reports_json_capabilities`; Settings checkbox invokes the guarded platform default-reader plan |
| `create_support_bundle` | `src-tauri/src/cli.rs` | `cli_tests::ned_cli_creates_redaction_safe_support_bundles`; Settings support-bundle action invokes the same redaction-safe JSON contract with setup, release-readiness, release evidence action-plan work items, spec-completion work orders, release-candidate status, transform-engine probe, and release evidence report summaries |
| `list_workspace_files` | `src-tauri/src/workspace_files.rs` | `file_command_tests::workspace_listing_skips_hidden_and_build_artifacts` |
| `compile_document` | `src-tauri/src/compiler.rs` | `compiler_core_tests::compiler_resolves_metadata_variables_transforms_and_manifest` |
| `compile_document_with_options` | `src-tauri/src/compiler.rs` | `export_option_tests::compile_options_supply_brand_profile_defaults`; `export_option_tests::compile_options_do_not_override_document_brand_profile` |
| `export_document` | `src-tauri/src/export_commands.rs` | `export_command_tests::export_document_writes_optional_sidecar_manifest` |
| `prepare_for_export` | `src-tauri/src/export_commands.rs` | `export_command_tests::prepare_for_export_carries_broad_readiness_audit_to_manifest`; `export_command_tests::prepare_for_export_validates_target_and_options` |
| `prepare_google_docs_live_import` | `src-tauri/src/export_commands.rs` | `export_command_tests::google_docs_live_import_preparation_creates_reviewable_package`; Google Docs live-import UI wiring is statically covered by `tests/frontend-unit.test.ts` |
| `import_spreadsheet_table` | `src-tauri/src/data_exchange.rs` | `table_tests::spreadsheet_table_import_export_round_trips_csv_and_xlsx` |
| `export_markdown_tables` | `src-tauri/src/data_exchange.rs` | `table_tests::spreadsheet_table_import_export_round_trips_csv_and_xlsx` |
| `start_google_oauth_sign_in` | `src-tauri/src/google_auth.rs` | `google_auth::tests::google_oauth_authorization_url_uses_pkce_and_loopback_values`; Settings Google sign-in wiring is statically covered by `tests/frontend-unit.test.ts` |
| `poll_google_oauth_sign_in` | `src-tauri/src/google_auth.rs` | `google_auth::tests::google_oauth_callback_parser_decodes_code_state_and_error`; Settings Google token polling wiring is statically covered by `tests/frontend-unit.test.ts` |
| `cancel_google_oauth_sign_in` | `src-tauri/src/google_auth.rs` | `google_auth::tests::google_oauth_authorization_url_uses_pkce_and_loopback_values`; Settings Google sign-out/cancel wiring is statically covered by `tests/frontend-unit.test.ts` |
| `import_rfp_source` | `src-tauri/src/rfp_import.rs` | `rfp_import::tests::import_rfp_source_accepts_pasted_markdown_aliases`; `rfp_import::tests::import_rfp_source_rejects_unsupported_source_types`; `rfp_import::tests::docx_xml_text_preserves_paragraphs_and_table_cells`; `tests/frontend-unit.test.ts` static RFP import wiring; RFP wizard source import workflows are covered by focused browser/native smoke gates |
| `prepare_local_agent_handoff` | `src-tauri/src/local_agents.rs` | `local_agents::tests::prepares_handoff_file_inside_workspace`; `local_agents::tests::document_workspace_paths_resolve_to_their_parent_folder`; `local_agents::tests::rejects_empty_local_agent_handoff_prompts`; `local_agents::tests::rejects_unknown_local_agent_profiles`; `local_agents::tests::allowlists_expected_local_agent_profiles`; `tests/frontend-unit.test.ts` static local-agent handoff wiring; Agent Workspace provider handoff workflows are covered by focused browser/native smoke gates |
| `import_local_agent_response` | `src-tauri/src/local_agents.rs` | `local_agents::tests::imports_local_agent_response_from_handoff_folder`; `local_agents::tests::blocks_local_agent_response_outside_handoff_folder`; `tests/frontend-unit.test.ts` static local-agent response import wiring |
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
| `search_citation_sources` | `src-tauri/src/citation_discovery.rs` | `citation_discovery::tests::duckduckgo_result_links_decode_target_url`; References source-search UI wiring is statically covered by `tests/frontend-unit.test.ts` |
| `download_citation_source` | `src-tauri/src/citation_discovery.rs` | `citation_discovery::tests::citation_source_directory_is_document_associated`; References source-download UI wiring is statically covered by `tests/frontend-unit.test.ts` |
| `list_citation_sources` | `src-tauri/src/citation_discovery.rs` | `citation_discovery::tests::citation_source_library_lists_manifest_items_newest_first`; References source-library UI wiring is statically covered by `tests/frontend-unit.test.ts` |
| `list_ollama_models` | `src-tauri/src/ollama_models.rs` | `ollama_models::tests::ollama_tags_endpoint_normalizes_native_chat_urls`; `ollama_models::tests::parse_ollama_models_reads_model_names_and_details`; Ollama model-picker UI wiring is statically covered by `tests/frontend-unit.test.ts` |
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
