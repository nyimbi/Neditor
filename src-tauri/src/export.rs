use crate::{
    document_ast::{export_body_text_from_ast, DocumentBlock, FootnoteEntry, InlineNode},
    escape_css, escape_html, escape_pdf, escape_xml,
    export_media::{
        drawingml_source_crop, export_dimensions_emu_size, normalized_fit, normalized_position,
        parse_export_image, safe_bundle_path, ExportImageDimensions,
    },
    generated_sections::toc_depth,
    layout::{matches_layout_break, LayoutSettings},
    metadata_string, metadata_string_list, render_export_template, sha256_uri,
    table_cells::TableCell,
    tables::delimited_rows_for_export,
    CompileResponse, ExportManifest,
};
use chrono::Utc;
use serde_json::{json, Value};
use std::{
    fs,
    io::{Cursor, Write},
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

mod shared;
pub(crate) use shared::export_text;
use shared::*;

mod html;
pub(crate) use html::render_full_html;

mod pdf;
pub(crate) use pdf::render_pdf_bytes;

mod docx;
pub(crate) use docx::render_docx_bytes;

mod latex;
pub(crate) use latex::render_latex_bytes;

mod pptx;
pub(crate) use pptx::render_pptx_bytes;

mod bundle;
pub(crate) use bundle::render_markdown_bundle_bytes;

mod blog;
pub(crate) use blog::render_blog_publish_package_bytes;

mod google_docs;
pub(crate) use google_docs::render_google_docs_package_bytes;

mod epub;
pub(crate) use epub::render_epub_bytes;
