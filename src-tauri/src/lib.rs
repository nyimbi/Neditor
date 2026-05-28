#[cfg(test)]
use serde_json::{json, Value};
#[cfg(test)]
use std::{collections::BTreeSet, fs, path::Path, path::PathBuf};

mod ai_cleanup;
mod bibliography;
mod calculations;
mod citation_discovery;
pub mod cli;
mod compile_options;
mod compiler;
mod compiler_support;
mod compiler_types;
mod data_exchange;
mod diagnostics;
mod diagnostics_types;
mod document_ast;
mod export;
mod export_commands;
mod export_media;
mod filesystem;
mod filesystem_watch;
mod footnotes;
mod front_matter;
mod generated_sections;
mod git;
mod git_support;
mod git_types;
mod google_auth;
mod html_preview;
mod indexing;
mod layout;
mod link_validation;
mod local_agents;
mod manifest;
mod markdown_tables;
mod ollama_models;
mod paged_document;
mod provenance;
mod references;
mod review;
mod rfp_import;
mod rich_blocks;
mod snapshot;
mod snapshot_metadata;
mod snapshot_storage;
mod source_mapping;
mod table_cells;
mod tables;
mod transform_install;
mod transforms;
mod tts;
mod utils;
mod validation;
mod variables;
mod workspace_files;

use ai_cleanup::cleanup_ai_paste;
#[cfg(test)]
use ai_cleanup::AiCleanupRequest;
use citation_discovery::{
    download_citation_source, list_citation_sources, search_citation_sources,
};
use cli::{
    configure_default_markdown_reader, create_support_bundle, default_markdown_reader_plan,
    pending_cli_open_paths,
};
#[cfg(test)]
use compiler::compile;
pub(crate) use compiler::compile_with_options;
use compiler::{compile_document, compile_document_with_options, run_transform};
pub(crate) use compiler_types::{
    CompileRequest, CompileResponse, ExportManifest, Heading, IncludeEdge, SourceMapEntry,
};
use data_exchange::{export_markdown_tables, import_spreadsheet_table};
pub(crate) use diagnostics::{diag, DocumentDiagnostic};
#[cfg(test)]
use document_ast::DocumentBlock;
#[cfg(test)]
use export::{
    render_blog_publish_package_bytes, render_docx_bytes, render_epub_bytes, render_full_html,
    render_google_docs_package_bytes, render_latex_bytes, render_markdown_bundle_bytes,
    render_pdf_bytes, render_pptx_bytes,
};
use export_commands::{export_document, prepare_for_export, prepare_google_docs_live_import};
#[cfg(test)]
use export_commands::{ExportRequest, GoogleDocsLiveImportRequest, PrepareExportRequest};
use filesystem::{
    duplicate_file, file_metadata, open_file, read_file, rename_file, reveal_path, save_file,
    save_file_as, FileResponse,
};
#[cfg(test)]
use filesystem::{DuplicateFileRequest, RenameFileRequest, SaveFileRequest};
#[cfg(all(test, feature = "native-watch"))]
use filesystem_watch::notify_event_should_emit;
#[cfg(test)]
use filesystem_watch::WatchFileRequest;
use filesystem_watch::{start_file_watcher, stop_file_watcher, watch_file, FileWatcherState};
use git::{
    commit_document_changes, get_git_status, git_diff, git_history, restore_git_revision,
    tag_release,
};
#[cfg(test)]
use git_support::run_git;
#[cfg(test)]
use git_types::{GitCommitRequest, GitPathRequest, GitRestoreRequest, GitTagRequest};
use google_auth::{
    cancel_google_oauth_sign_in, poll_google_oauth_sign_in, start_google_oauth_sign_in,
    GoogleAuthState,
};
use local_agents::{import_local_agent_response, prepare_local_agent_handoff};
use ollama_models::list_ollama_models;
use rfp_import::import_rfp_source;
use snapshot::{create_snapshot, list_snapshots, restore_snapshot};
use tauri::{
    menu::{Menu, MenuItemBuilder, SubmenuBuilder},
    AppHandle, Emitter, Manager, Runtime,
};
use transform_install::{install_transform_handlers, list_transform_handler_installers};
#[cfg(test)]
use transforms::external::ExternalTransformRequest;
use transforms::external::{list_transform_engines, run_external_transform};
#[cfg(test)]
use transforms::renderer::supported_transform;
use tts::{
    download_tts_model, inspect_native_tts, read_text_aloud, stop_text_aloud, NativeTtsState,
};
pub(crate) use utils::{
    escape_css, escape_html, escape_pdf, escape_xml, format_value, metadata_lookup,
    metadata_string, metadata_string_list, path_to_string, render_export_template, sha256_hex,
    sha256_uri, value_to_string,
};
use workspace_files::list_workspace_files;
#[cfg(test)]
use workspace_files::WorkspaceFileRequest;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .menu(build_neditor_menu)
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            if id.starts_with("neditor-") {
                let _ = app.emit("neditor-menu-command", id);
            }
        })
        .manage(FileWatcherState::default())
        .manage(GoogleAuthState::default())
        .manage(NativeTtsState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            write_desktop_smoke_report(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_file,
            read_file,
            save_file_as,
            save_file,
            watch_file,
            start_file_watcher,
            stop_file_watcher,
            rename_file,
            duplicate_file,
            reveal_path,
            file_metadata,
            pending_cli_open_paths,
            default_markdown_reader_plan,
            configure_default_markdown_reader,
            create_support_bundle,
            list_workspace_files,
            compile_document,
            compile_document_with_options,
            export_document,
            prepare_for_export,
            prepare_google_docs_live_import,
            import_spreadsheet_table,
            export_markdown_tables,
            start_google_oauth_sign_in,
            poll_google_oauth_sign_in,
            cancel_google_oauth_sign_in,
            import_rfp_source,
            prepare_local_agent_handoff,
            import_local_agent_response,
            create_snapshot,
            list_snapshots,
            restore_snapshot,
            get_git_status,
            git_history,
            git_diff,
            commit_document_changes,
            tag_release,
            restore_git_revision,
            list_transform_engines,
            list_transform_handler_installers,
            install_transform_handlers,
            run_transform,
            run_external_transform,
            cleanup_ai_paste,
            search_citation_sources,
            download_citation_source,
            list_citation_sources,
            list_ollama_models,
            inspect_native_tts,
            download_tts_model,
            read_text_aloud,
            stop_text_aloud,
            write_desktop_ui_smoke_report,
            desktop_workflow_smoke_enabled,
            desktop_workflow_smoke_autorun_enabled,
            desktop_workflow_smoke_file_path,
            desktop_workflow_smoke_named_path,
            desktop_workflow_smoke_export_path,
            emit_desktop_workflow_smoke_menu_command,
            write_desktop_workflow_smoke_report
        ])
        .run(tauri::generate_context!())
        .expect("error while running NEditor");
}

fn build_neditor_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let export_menu = SubmenuBuilder::new(app, "Export")
        .item(&menu_item(app, "neditor-export-html", "HTML Export")?)
        .separator()
        .item(&menu_item(
            app,
            "neditor-prepare-export",
            "Prepare for Export",
        )?)
        .item(&menu_item(
            app,
            "neditor-export-current",
            "Export Selected Target",
        )?)
        .separator()
        .item(&menu_item(
            app,
            "neditor-open-publishing-handoff",
            "Open Publishing Handoff",
        )?)
        .item(&menu_item(
            app,
            "neditor-prepare-publishing-handoff",
            "Prepare Publishing Packet",
        )?)
        .item(&menu_item(
            app,
            "neditor-copy-publishing-payload",
            "Copy Publishing Payload",
        )?)
        .item(&menu_item(
            app,
            "neditor-copy-publishing-content",
            "Copy Publishing Content",
        )?)
        .separator()
        .item(&menu_item(app, "neditor-export-pdf", "PDF Export")?)
        .item(&menu_item(app, "neditor-export-docx", "DOCX Export")?)
        .item(&menu_item(app, "neditor-export-pptx", "PPTX Export")?)
        .item(&menu_item(
            app,
            "neditor-export-markdown-bundle",
            "Markdown Bundle Export",
        )?)
        .item(&menu_item(
            app,
            "neditor-export-blog",
            "Blog Package Export",
        )?)
        .item(&menu_item(
            app,
            "neditor-export-substack",
            "Substack Package Export",
        )?)
        .item(&menu_item(app, "neditor-export-latex", "LaTeX Export")?)
        .separator()
        .item(&menu_item(
            app,
            "neditor-configure-google-docs",
            "Configure Google Docs Sign-In",
        )?)
        .item(&menu_item(
            app,
            "neditor-sign-in-google",
            "Sign in with Google",
        )?)
        .item(&menu_item(
            app,
            "neditor-import-google-docs",
            "Import Current Document to Google Docs",
        )?)
        .separator()
        .item(&menu_item(
            app,
            "neditor-export-google-docs",
            "Google Docs Package Export",
        )?)
        .item(&menu_item(app, "neditor-export-epub", "EPUB Export")?)
        .build()?;

    let file_menu = SubmenuBuilder::new(app, "File")
        .item(&menu_item(app, "neditor-new-document", "New Document")?)
        .item(&menu_item(app, "neditor-open-document", "Open Document")?)
        .separator()
        .item(&menu_item(app, "neditor-save-document", "Save Document")?)
        .item(&menu_item(
            app,
            "neditor-save-document-as",
            "Save Document As",
        )?)
        .separator()
        .item(&menu_item(
            app,
            "neditor-rename-document",
            "Rename Document",
        )?)
        .item(&menu_item(
            app,
            "neditor-duplicate-document",
            "Duplicate Document",
        )?)
        .item(&menu_item(
            app,
            "neditor-create-snapshot",
            "Create Snapshot",
        )?)
        .separator()
        .item(&export_menu)
        .separator()
        .item(&menu_item(app, "neditor-open-folder", "Open Folder")?)
        .item(&menu_item(app, "neditor-save-workspace", "Save Workspace")?)
        .separator()
        .close_window()
        .build()?;

    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .separator()
        .item(&menu_item(app, "neditor-open-search", "Find and Replace")?)
        .build()?;

    let view_menu = SubmenuBuilder::new(app, "View")
        .item(&menu_item(app, "neditor-mode-split", "Split View")?)
        .item(&menu_item(app, "neditor-mode-source", "Source Only")?)
        .item(&menu_item(app, "neditor-mode-preview", "Preview Only")?)
        .item(&menu_item(app, "neditor-mode-focus", "Focus Mode")?)
        .item(&menu_item(app, "neditor-mode-outline", "Outline Mode")?)
        .item(&menu_item(app, "neditor-mode-export", "Export Preview")?)
        .separator()
        .item(&menu_item(app, "neditor-show-outline", "Document Outline")?)
        .item(&menu_item(app, "neditor-show-exports", "Export Panel")?)
        .separator()
        .fullscreen()
        .build()?;

    let document_wizards_menu = SubmenuBuilder::new(app, "Document Wizards")
        .item(&menu_item(
            app,
            "neditor-open-document-wizards",
            "Open Wizard Hub",
        )?)
        .separator()
        .item(&menu_item(app, "neditor-wizard-proposal", "Proposal")?)
        .item(&menu_item(
            app,
            "neditor-wizard-rfp-response",
            "RFP Response",
        )?)
        .item(&menu_item(
            app,
            "neditor-wizard-tender-response",
            "Tender Response",
        )?)
        .separator()
        .item(&menu_item(
            app,
            "neditor-wizard-lesson-plan",
            "Lesson Plan",
        )?)
        .item(&menu_item(
            app,
            "neditor-wizard-lesson-content",
            "Lesson Content",
        )?)
        .item(&menu_item(
            app,
            "neditor-wizard-technical-textbook",
            "Technical Textbook",
        )?)
        .separator()
        .item(&menu_item(app, "neditor-wizard-novel", "Novel")?)
        .item(&menu_item(
            app,
            "neditor-wizard-podcast-script",
            "Podcast Script",
        )?)
        .item(&menu_item(
            app,
            "neditor-wizard-movie-script",
            "Movie Script",
        )?)
        .build()?;

    let writing_tools_menu = SubmenuBuilder::new(app, "Writing Tools")
        .item(&menu_item(app, "neditor-open-search", "Find and Replace")?)
        .separator()
        .item(&menu_item(app, "neditor-insert-table", "Insert Table")?)
        .item(&menu_item(
            app,
            "neditor-open-table-editor",
            "Open Table Editor",
        )?)
        .item(&menu_item(
            app,
            "neditor-edit-table-at-cursor",
            "Edit Table at Cursor",
        )?)
        .item(&menu_item(
            app,
            "neditor-edit-table-cell-at-cursor",
            "Edit Table Cell at Cursor",
        )?)
        .item(&menu_item(
            app,
            "neditor-go-to-source-table",
            "Go to Source Table",
        )?)
        .item(&menu_item(
            app,
            "neditor-insert-code-fence",
            "Insert Code Fence",
        )?)
        .item(&menu_item(
            app,
            "neditor-insert-equation",
            "Insert Equation",
        )?)
        .item(&menu_item(
            app,
            "neditor-insert-toc",
            "Insert Table of Contents",
        )?)
        .item(&menu_item(
            app,
            "neditor-open-templates",
            "Transform Templates",
        )?)
        .item(&menu_item(
            app,
            "neditor-install-transform-handlers",
            "Install Transform Handlers",
        )?)
        .item(&document_wizards_menu)
        .separator()
        .item(&menu_item(
            app,
            "neditor-read-selection-aloud",
            "Read Selection Aloud",
        )?)
        .item(&menu_item(
            app,
            "neditor-read-document-aloud",
            "Read Document Aloud",
        )?)
        .item(&menu_item(app, "neditor-stop-reading", "Stop Reading")?)
        .separator()
        .item(&menu_item(app, "neditor-open-docs-live", "Docs Live")?)
        .item(&menu_item(
            app,
            "neditor-open-deep-research",
            "Deep Research",
        )?)
        .item(&menu_item(
            app,
            "neditor-open-agent-workspace",
            "AI Agent Workspace",
        )?)
        .item(&menu_item(
            app,
            "neditor-ai-create-document",
            "AI Create Document",
        )?)
        .item(&menu_item(app, "neditor-clean-ai-paste", "Clean AI Paste")?)
        .build()?;

    let quality_menu = SubmenuBuilder::new(app, "Quality")
        .item(&menu_item(
            app,
            "neditor-run-qa-review",
            "Run QA Recommendations",
        )?)
        .item(&menu_item(
            app,
            "neditor-insert-qa-report",
            "Insert QA Report",
        )?)
        .item(&menu_item(
            app,
            "neditor-improve-with-agent",
            "Improve with Agent",
        )?)
        .separator()
        .item(&menu_item(
            app,
            "neditor-prepare-release-metadata",
            "Prepare Release Metadata",
        )?)
        .item(&menu_item(
            app,
            "neditor-insert-release-audit",
            "Insert Release Audit",
        )?)
        .build()?;

    let help_menu = SubmenuBuilder::new(app, "Help")
        .item(&menu_item(app, "neditor-open-help", "NEditor Help Center")?)
        .item(&menu_item(app, "neditor-guided-demo", "Guided Demo")?)
        .separator()
        .item(&menu_item(
            app,
            "neditor-help-getting-started",
            "Getting Started",
        )?)
        .item(&menu_item(app, "neditor-help-docs-live", "Docs Live")?)
        .item(&menu_item(
            app,
            "neditor-help-exports",
            "Export and Publishing",
        )?)
        .item(&menu_item(
            app,
            "neditor-help-shortcuts",
            "Keyboard Shortcuts",
        )?)
        .build()?;

    Menu::with_items(
        app,
        &[
            &file_menu,
            &edit_menu,
            &view_menu,
            &writing_tools_menu,
            &quality_menu,
            &help_menu,
        ],
    )
}

fn menu_item<R: Runtime>(
    app: &AppHandle<R>,
    id: &'static str,
    label: &'static str,
) -> tauri::Result<tauri::menu::MenuItem<R>> {
    MenuItemBuilder::with_id(id, label).build(app)
}

fn write_desktop_smoke_report(app: &tauri::App) {
    let Ok(report_path) = std::env::var("NEDITOR_DESKTOP_SMOKE_REPORT") else {
        return;
    };
    let window = app.get_webview_window("main");
    let package_info = app.package_info();
    let payload = serde_json::json!({
        "generatedAt": chrono::Utc::now().to_rfc3339(),
        "packageName": package_info.name,
        "version": package_info.version.to_string(),
        "identifier": app.config().identifier,
        "window": window.as_ref().map(|item| serde_json::json!({
            "label": item.label(),
            "title": item.title().ok(),
            "visible": item.is_visible().ok(),
            "innerSize": item.inner_size().ok().map(|size| serde_json::json!({
                "width": size.width,
                "height": size.height,
            })),
            "scaleFactor": item.scale_factor().ok(),
        })),
    });
    if let Some(parent) = std::path::Path::new(&report_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(serialized) = serde_json::to_string_pretty(&payload) {
        let _ = std::fs::write(report_path, format!("{serialized}\n"));
    }
}

#[tauri::command]
fn write_desktop_ui_smoke_report(payload: serde_json::Value) -> Result<(), String> {
    let Ok(report_path) = std::env::var("NEDITOR_DESKTOP_UI_SMOKE_REPORT") else {
        return Ok(());
    };
    write_guarded_desktop_report(report_path, payload)
}

#[tauri::command]
fn desktop_workflow_smoke_enabled() -> bool {
    std::env::var("NEDITOR_DESKTOP_WORKFLOW_SMOKE_REPORT").is_ok()
}

#[tauri::command]
fn desktop_workflow_smoke_autorun_enabled() -> bool {
    std::env::var("NEDITOR_DESKTOP_WORKFLOW_SMOKE_REPORT").is_ok()
        && std::env::var("NEDITOR_DESKTOP_WORKFLOW_DISABLE_AUTORUN").is_err()
}

#[tauri::command]
fn desktop_workflow_smoke_file_path(extension: String) -> Result<Option<String>, String> {
    desktop_workflow_smoke_artifact_path("native-workflow-file", extension)
}

#[tauri::command]
fn desktop_workflow_smoke_named_path(
    file_stem: String,
    extension: String,
) -> Result<Option<String>, String> {
    let safe_file_stem = file_stem
        .trim()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == '_')
        .collect::<String>();
    if safe_file_stem.is_empty() {
        return Err("desktop workflow smoke file stem is empty".to_string());
    }
    desktop_workflow_smoke_artifact_path(&safe_file_stem, extension)
}

#[tauri::command]
fn desktop_workflow_smoke_export_path(extension: String) -> Result<Option<String>, String> {
    desktop_workflow_smoke_artifact_path("native-workflow-export", extension)
}

#[tauri::command]
fn emit_desktop_workflow_smoke_menu_command(app: AppHandle, command: String) -> Result<(), String> {
    if std::env::var("NEDITOR_DESKTOP_WORKFLOW_SMOKE_REPORT").is_err() {
        return Err("desktop workflow smoke menu command is disabled".to_string());
    }
    if !command.starts_with("neditor-") {
        return Err("desktop workflow smoke menu command must be an NEditor command".to_string());
    }
    tauri::async_runtime::spawn(async move {
        let _ = app.emit("neditor-menu-command", command);
    });
    Ok(())
}

fn desktop_workflow_smoke_artifact_path(
    file_stem: &str,
    extension: String,
) -> Result<Option<String>, String> {
    let Ok(report_path) = std::env::var("NEDITOR_DESKTOP_WORKFLOW_SMOKE_REPORT") else {
        return Ok(None);
    };
    let safe_extension = extension
        .trim()
        .trim_start_matches('.')
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>();
    if safe_extension.is_empty() {
        return Err("desktop workflow smoke export extension is empty".to_string());
    }
    let mut output_path = std::path::PathBuf::from(report_path);
    output_path.set_file_name(format!("{file_stem}.{safe_extension}"));
    Ok(Some(path_to_string(&output_path)))
}

#[tauri::command]
fn write_desktop_workflow_smoke_report(payload: serde_json::Value) -> Result<(), String> {
    let Ok(report_path) = std::env::var("NEDITOR_DESKTOP_WORKFLOW_SMOKE_REPORT") else {
        return Ok(());
    };
    write_guarded_desktop_report(report_path, payload)
}

fn write_guarded_desktop_report(
    report_path: String,
    payload: serde_json::Value,
) -> Result<(), String> {
    let payload = serde_json::json!({
        "generatedAt": chrono::Utc::now().to_rfc3339(),
        "payload": payload,
    });
    if let Some(parent) = std::path::Path::new(&report_path).parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let serialized = serde_json::to_string_pretty(&payload).map_err(|error| error.to_string())?;
    std::fs::write(report_path, format!("{serialized}\n")).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests;
