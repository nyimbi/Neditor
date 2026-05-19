use crate::{
    diagnostics::diag, document_ast::AstSourceRange, front_matter::strip_front_matter,
    path_to_string, DocumentDiagnostic, IncludeEdge, SourceMapEntry,
};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

const MAX_INCLUDE_DEPTH: usize = 16;

#[allow(clippy::too_many_arguments)]
pub(crate) fn expand_includes(
    text: &str,
    current_path: Option<&Path>,
    source_file: &str,
    depth: usize,
    visited: &mut HashSet<PathBuf>,
    include_graph: &mut Vec<IncludeEdge>,
    source_map: &mut Vec<SourceMapEntry>,
    generated_line_count: &mut usize,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    if depth > MAX_INCLUDE_DEPTH {
        diagnostics.push(diag(
            "error",
            "Maximum include depth exceeded.",
            Some(source_file.to_string()),
            None,
            Some("Reduce nested include directives."),
        ));
        return String::new();
    }

    let base_dir = current_path
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut output = String::new();
    for (line_index, line) in text.lines().enumerate() {
        if let Some(include_target) = parse_include_directive(line) {
            let child = base_dir.join(include_target);
            let canonical = child.canonicalize().unwrap_or(child.clone());
            if visited.contains(&canonical) {
                let mut diagnostic = diag(
                    "error",
                    "Circular include detected.",
                    Some(source_file.to_string()),
                    Some(line_index + 1),
                    Some("Remove the cycle or include a different file."),
                );
                diagnostic
                    .related
                    .push(format!("Include target: {}", child.display()));
                diagnostic
                    .related
                    .push(format!("Canonical path: {}", canonical.display()));
                diagnostics.push(diagnostic);
                continue;
            }
            if !child.exists() {
                let mut diagnostic = diag(
                    "error",
                    format!("Missing include file: {}", child.display()),
                    Some(source_file.to_string()),
                    Some(line_index + 1),
                    Some("Create the file or update the include path."),
                );
                diagnostic
                    .related
                    .push(format!("Include target: {include_target}"));
                diagnostic
                    .related
                    .push(format!("Resolved path: {}", child.display()));
                diagnostics.push(diagnostic);
                continue;
            }
            match fs::read_to_string(&child) {
                Ok(child_text) => {
                    include_graph.push(IncludeEdge {
                        parent: source_file.to_string(),
                        child: path_to_string(&child),
                        depth: depth + 1,
                    });
                    visited.insert(canonical.clone());
                    let child_without_front_matter = strip_front_matter(&child_text);
                    push_unmapped_expanded_text(
                        &mut output,
                        generated_line_count,
                        &format!("\n\n<!-- begin include: {} -->\n", child.display()),
                    );
                    output.push_str(&expand_includes(
                        &child_without_front_matter,
                        Some(&child),
                        &path_to_string(&child),
                        depth + 1,
                        visited,
                        include_graph,
                        source_map,
                        generated_line_count,
                        diagnostics,
                    ));
                    push_unmapped_expanded_text(
                        &mut output,
                        generated_line_count,
                        &format!("\n<!-- end include: {} -->\n\n", child.display()),
                    );
                    visited.remove(&canonical);
                }
                Err(err) => diagnostics.push(diag(
                    "error",
                    format!("Unable to read include file: {err}"),
                    Some(source_file.to_string()),
                    Some(line_index + 1),
                    Some("Check file permissions."),
                )),
            }
        } else {
            let generated_line = *generated_line_count + 1;
            source_map.push(SourceMapEntry {
                generated_line,
                source_file: source_file.to_string(),
                source_line: line_index + 1,
            });
            output.push_str(line);
            output.push('\n');
            *generated_line_count += 1;
        }
    }
    output
}

fn push_unmapped_expanded_text(output: &mut String, generated_line_count: &mut usize, text: &str) {
    output.push_str(text);
    *generated_line_count += text.chars().filter(|ch| *ch == '\n').count();
}

fn parse_include_directive(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("!include ") {
        return Some(rest.trim());
    }
    if let Some(rest) = trimmed.strip_prefix("{{include ") {
        return rest.strip_suffix("}}").map(str::trim);
    }
    if let Some(rest) = trimmed.strip_prefix("<!-- include:") {
        return rest.strip_suffix("-->").map(str::trim);
    }
    None
}

pub(crate) fn normalize_source_map_after_front_matter(
    source_map: &mut Vec<SourceMapEntry>,
    body_start_line: usize,
) {
    let offset = body_start_line.saturating_sub(1);
    source_map.retain(|entry| entry.generated_line >= body_start_line);
    for entry in source_map {
        entry.generated_line = entry.generated_line.saturating_sub(offset);
    }
}

pub(crate) fn ast_source_range_for_generated_lines(
    source_map: &[SourceMapEntry],
    line: usize,
    end_line: usize,
) -> Option<AstSourceRange> {
    let start = source_map
        .iter()
        .find(|entry| entry.generated_line == line)?;
    let end = source_map
        .iter()
        .rev()
        .find(|entry| {
            entry.generated_line >= line
                && entry.generated_line <= end_line
                && entry.source_file == start.source_file
        })
        .unwrap_or(start);
    Some(AstSourceRange {
        source_file: start.source_file.clone(),
        source_line: start.source_line,
        end_source_line: end.source_line,
    })
}
