#[cfg(test)]
use serde_json::{json, Value};
#[cfg(test)]
use std::{collections::BTreeSet, fs, path::Path, path::PathBuf};

mod ai_cleanup;
mod bibliography;
mod calculations;
mod compile_options;
mod compiler;
mod compiler_support;
mod compiler_types;
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
mod html_preview;
mod indexing;
mod layout;
mod link_validation;
mod manifest;
mod markdown_tables;
mod paged_document;
mod provenance;
mod references;
mod review;
mod rich_blocks;
mod snapshot;
mod snapshot_metadata;
mod snapshot_storage;
mod source_mapping;
mod table_cells;
mod tables;
mod transforms;
mod utils;
mod validation;
mod variables;
mod workspace_files;

use ai_cleanup::cleanup_ai_paste;
#[cfg(test)]
use ai_cleanup::AiCleanupRequest;
#[cfg(test)]
use compiler::compile;
pub(crate) use compiler::compile_with_options;
use compiler::{compile_document, compile_document_with_options, run_transform};
pub(crate) use compiler_types::{
    CompileRequest, CompileResponse, ExportManifest, Heading, IncludeEdge, SourceMapEntry,
};
pub(crate) use diagnostics::{diag, DocumentDiagnostic};
#[cfg(test)]
use document_ast::DocumentBlock;
#[cfg(test)]
use export::{
    render_docx_bytes, render_full_html, render_markdown_bundle_bytes, render_pdf_bytes,
    render_pptx_bytes,
};
use export_commands::{export_document, prepare_for_export};
#[cfg(test)]
use export_commands::{ExportRequest, PrepareExportRequest};
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
use snapshot::{create_snapshot, list_snapshots, restore_snapshot};
#[cfg(test)]
use transforms::external::ExternalTransformRequest;
use transforms::external::{list_transform_engines, run_external_transform};
#[cfg(test)]
use transforms::renderer::supported_transform;
pub(crate) use utils::{
    escape_css, escape_html, escape_pdf, escape_xml, format_value, metadata_lookup,
    metadata_string, path_to_string, render_export_template, sha256_hex, sha256_uri,
    value_to_string,
};
use workspace_files::list_workspace_files;
#[cfg(test)]
use workspace_files::WorkspaceFileRequest;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(FileWatcherState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
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
            list_workspace_files,
            compile_document,
            compile_document_with_options,
            export_document,
            prepare_for_export,
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
            run_transform,
            run_external_transform,
            cleanup_ai_paste
        ])
        .run(tauri::generate_context!())
        .expect("error while running NEditor");
}

#[cfg(test)]
mod tests;
