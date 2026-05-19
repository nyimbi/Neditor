use crate::{
    diagnostic_location_for_generated_line,
    diagnostics::{diag, with_range, DocumentDiagnostic},
    metadata_string, path_to_string, SourceMapEntry,
};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub(crate) fn validate_image_paths(
    markdown: &str,
    root_path: Option<&Path>,
    source_map: &[SourceMapEntry],
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let base_dir = root_path
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    for (line_index, line) in markdown.lines().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("![") {
            continue;
        }
        let Some(close_index) = line.find("](") else {
            continue;
        };
        let target_start = close_index + 2;
        let Some(relative_end) = line[target_start..].find(')') else {
            continue;
        };
        let target_end = target_start + relative_end;
        let src = &line[target_start..target_end];
        if src.starts_with("http://") || src.starts_with("https://") || src.starts_with("data:") {
            continue;
        }
        let path = base_dir.join(src);
        if !path.exists() {
            let (source_file, line) =
                diagnostic_location_for_generated_line(source_map, line_index + 1);
            let mut diagnostic = with_range(
                diag(
                    "warning",
                    format!("Broken image path: {}", path.display()),
                    source_file,
                    line,
                    Some("Create the image file or update the image path."),
                ),
                target_start + 1,
                line,
                target_end + 1,
            );
            diagnostic
                .related
                .push(format!("Image target: {}", path.display()));
            diagnostics.push(diagnostic);
        }
    }
}

pub(crate) fn validate_logo_path(
    metadata: &Value,
    root_path: Option<&Path>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let Some(logo) = metadata_string(metadata, "brand.logo")
        .or_else(|| metadata_string(metadata, "layout.logo"))
        .or_else(|| metadata_string(metadata, "logo"))
    else {
        return;
    };
    if logo.trim().is_empty() || !should_validate_local_link(&logo) {
        return;
    }
    let base_dir = root_path
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let path = base_dir.join(&logo);
    if !path.exists() {
        diagnostics.push(diag(
            "warning",
            format!("Broken logo path: {}", path.display()),
            Some(path_to_string(&path)),
            None,
            Some("Create the logo file or update the logo metadata path."),
        ));
    }
}

pub(crate) fn validate_link_paths(
    markdown: &str,
    root_path: Option<&Path>,
    source_map: &[SourceMapEntry],
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let base_dir = root_path
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    for (line_index, line) in markdown.lines().enumerate() {
        let mut search_from = 0usize;
        while let Some(relative_close) = line[search_from..].find("](") {
            let close_index = search_from + relative_close;
            let Some(open_index) = line[..close_index].rfind('[') else {
                search_from = close_index + 2;
                continue;
            };
            if open_index > 0 && line.as_bytes().get(open_index - 1) == Some(&b'!') {
                search_from = close_index + 2;
                continue;
            }
            let target_start = close_index + 2;
            let Some(relative_end) = line[target_start..].find(')') else {
                break;
            };
            let target_end = target_start + relative_end;
            if let Some(destination) = markdown_link_destination(&line[target_start..target_end]) {
                if should_validate_local_link(&destination) {
                    let path_part = destination
                        .split_once('#')
                        .map_or(destination.as_str(), |(path, _)| path);
                    if !path_part.is_empty() {
                        let path = base_dir.join(path_part);
                        if !path.exists() {
                            let (source_file, line) =
                                diagnostic_location_for_generated_line(source_map, line_index + 1);
                            let mut diagnostic = with_range(
                                diag(
                                    "warning",
                                    format!("Broken link path: {}", path.display()),
                                    source_file,
                                    line,
                                    Some("Create the linked file or update the Markdown link."),
                                ),
                                target_start + 1,
                                line,
                                target_end + 1,
                            );
                            diagnostic
                                .related
                                .push(format!("Link target: {}", path.display()));
                            diagnostics.push(diagnostic);
                        }
                    }
                }
            }
            search_from = target_end + 1;
        }
    }
}

fn markdown_link_destination(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(stripped) = trimmed.strip_prefix('<') {
        return stripped
            .split_once('>')
            .map(|(destination, _)| destination.to_string());
    }
    Some(trimmed.split_whitespace().next()?.to_string())
}

fn should_validate_local_link(destination: &str) -> bool {
    !destination.starts_with('#')
        && !destination.starts_with("mailto:")
        && !destination.starts_with("tel:")
        && !destination.starts_with("data:")
        && !destination.starts_with("{{")
        && !destination.contains("://")
}
