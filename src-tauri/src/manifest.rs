use crate::{
    compiler_types::{ManifestFile, ManifestLayoutSection},
    document_ast::{AstSourceRange, DocumentAst, DocumentBlock},
    paged_document::PagedDocument,
    path_to_string, sha256_uri,
};
use std::{collections::BTreeSet, fs, path::PathBuf};

pub(crate) fn count_figures(text: &str) -> usize {
    text.matches("![").count()
}

pub(crate) fn count_equations(text: &str) -> usize {
    text.matches("$$").count() / 2
}

pub(crate) fn manifest_file(path: &str) -> Option<ManifestFile> {
    let bytes = fs::read(path).ok()?;
    Some(ManifestFile {
        path: path.to_string(),
        hash: sha256_uri(&bytes),
    })
}

pub(crate) fn manifest_media_files(document_ast: &DocumentAst) -> Vec<ManifestFile> {
    let mut seen = BTreeSet::new();
    let mut files = Vec::new();
    for block in &document_ast.blocks {
        let DocumentBlock::Figure {
            src: Some(src),
            source,
            ..
        } = block
        else {
            continue;
        };
        let Some(path) = manifest_media_path(src, source.as_ref()) else {
            continue;
        };
        if seen.insert(path.clone()) {
            if let Some(file) = manifest_file(&path) {
                files.push(file);
            }
        }
    }
    files
}

pub(crate) fn manifest_layout_sections(
    paged_document: &PagedDocument,
) -> Vec<ManifestLayoutSection> {
    paged_document
        .sections
        .iter()
        .map(|section| ManifestLayoutSection {
            id: section.id.clone(),
            title: section.title.clone(),
            start_line: section.start_line,
            end_line: section.end_line,
            columns: section.layout.columns,
            page_size: section.layout.page_size.clone(),
            orientation: section.layout.orientation.clone(),
            margins: section.layout.margins.clone(),
            header: section.layout.header.clone(),
            footer: section.layout.footer.clone(),
        })
        .collect()
}

fn manifest_media_path(src: &str, source: Option<&AstSourceRange>) -> Option<String> {
    if src.starts_with("data:") || src.contains("://") || src.starts_with('#') {
        return None;
    }
    let path = PathBuf::from(src);
    let resolved = if path.is_absolute() {
        path
    } else if let Some(source) = source {
        PathBuf::from(&source.source_file)
            .parent()
            .map(|parent| parent.join(src))
            .unwrap_or(path)
    } else {
        path
    };
    Some(path_to_string(&resolved))
}
