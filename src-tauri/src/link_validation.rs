use crate::{
    compiler_support::fenced_code_marker,
    diagnostics::{diag, with_range, DocumentDiagnostic},
    metadata_string,
    source_mapping::diagnostic_location_for_generated_line,
    SourceMapEntry,
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
    let mut fence_marker = None;
    for (line_index, line) in markdown.lines().enumerate() {
        if let Some(marker) = fence_marker {
            if line.trim_start().starts_with(marker) {
                fence_marker = None;
            }
            continue;
        }
        if let Some(marker) = fenced_code_marker(line) {
            fence_marker = Some(marker);
            continue;
        }
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
    markdown: &str,
    source_file: &str,
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
        let mut diagnostic =
            if let Some((line, column, end_column)) = front_matter_logo_value_range(markdown) {
                with_range(
                    diag(
                        "warning",
                        format!("Broken logo path: {}", path.display()),
                        Some(source_file.to_string()),
                        Some(line),
                        Some("Create the logo file or update the logo metadata path."),
                    ),
                    column,
                    Some(line),
                    end_column,
                )
            } else {
                diag(
                    "warning",
                    format!("Broken logo path: {}", path.display()),
                    Some(source_file.to_string()),
                    None,
                    Some("Create the logo file or update the logo metadata path."),
                )
            };
        diagnostic
            .related
            .push(format!("Logo target: {}", path.display()));
        diagnostics.push(diagnostic);
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
    let mut fence_marker = None;
    for (line_index, line) in markdown.lines().enumerate() {
        if let Some(marker) = fence_marker {
            if line.trim_start().starts_with(marker) {
                fence_marker = None;
            }
            continue;
        }
        if let Some(marker) = fenced_code_marker(line) {
            fence_marker = Some(marker);
            continue;
        }
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

fn front_matter_logo_value_range(markdown: &str) -> Option<(usize, usize, usize)> {
    let mut lines = markdown.lines().enumerate();
    let (_, first) = lines.next()?;
    if first.trim() != "---" {
        return None;
    }
    let mut stack: Vec<(usize, String)> = Vec::new();
    for (index, line) in lines {
        if line.trim() == "---" {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = yaml_indent_width(line);
        while stack
            .last()
            .is_some_and(|(stack_indent, _)| *stack_indent >= indent)
        {
            stack.pop();
        }
        let Some(colon_index) = line.find(':') else {
            continue;
        };
        let key = line[..colon_index]
            .trim()
            .trim_matches('"')
            .trim_matches('\'');
        if key.is_empty() {
            continue;
        }
        let value_start_zero = colon_index
            + 1
            + line[colon_index + 1..]
                .chars()
                .take_while(|character| character.is_whitespace())
                .map(char::len_utf8)
                .sum::<usize>();
        let value_end_zero = markdown_value_end(line, value_start_zero);
        let full_path = front_matter_path(&stack, key);
        if is_logo_metadata_path(&full_path) && value_end_zero > value_start_zero {
            return Some((index + 1, value_start_zero + 1, value_end_zero + 1));
        }
        if value_start_zero >= line.len() {
            stack.push((indent, key.to_string()));
        }
    }
    None
}

fn front_matter_path(stack: &[(usize, String)], key: &str) -> Vec<String> {
    let mut path = stack
        .iter()
        .map(|(_, part)| part.clone())
        .collect::<Vec<_>>();
    path.extend(key.split('.').map(str::to_string));
    path
}

fn is_logo_metadata_path(path: &[String]) -> bool {
    matches!(
        path,
        [key] if key == "logo"
    ) || matches!(
        path,
        [parent, key] if (parent == "brand" || parent == "layout") && key == "logo"
    )
}

fn markdown_value_end(line: &str, value_start_zero: usize) -> usize {
    let mut in_single = false;
    let mut in_double = false;
    for (offset, character) in line[value_start_zero..].char_indices() {
        match character {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '#' if !in_single && !in_double => return value_start_zero + offset,
            _ => {}
        }
    }
    line.len()
}

fn yaml_indent_width(line: &str) -> usize {
    line.chars()
        .take_while(|character| character.is_whitespace())
        .map(|character| if character == '\t' { 2 } else { 1 })
        .sum()
}
