use super::TransformArtifact;
use crate::{
    diagnostics::DocumentDiagnostic, source_mapping::ast_source_range_for_generated_lines,
    SourceMapEntry,
};
use serde_json::Value;

pub(crate) fn apply_transform_fences<F, S>(
    text: &str,
    source_map: &[SourceMapEntry],
    diagnostics: &mut Vec<DocumentDiagnostic>,
    mut is_supported: S,
    mut render: F,
) -> (String, Vec<TransformArtifact>)
where
    F: FnMut(&str, &str, &Value, &mut Vec<DocumentDiagnostic>) -> TransformArtifact,
    S: FnMut(&str) -> bool,
{
    let mut output = String::new();
    let mut artifacts = Vec::new();
    let mut lines = text.lines().enumerate().peekable();
    while let Some((line_index, line)) = lines.next() {
        if let Some(info) = line.trim().strip_prefix("```") {
            let name = info.split_whitespace().next().unwrap_or("");
            if is_supported(name) {
                let source_line = line_index + 1;
                let mut end_source_line = source_line;
                let fence_options = transform_fence_options(info);
                let mut body = String::new();
                for (body_line_index, body_line) in lines.by_ref() {
                    if body_line.trim() == "```" {
                        end_source_line = body_line_index + 1;
                        break;
                    }
                    body.push_str(body_line);
                    body.push('\n');
                    end_source_line = body_line_index + 1;
                }
                let diagnostic_start = diagnostics.len();
                let mut artifact = render(name, &body, &fence_options, diagnostics);
                attach_transform_source(
                    &mut artifact,
                    source_map,
                    source_line,
                    end_source_line,
                    &mut diagnostics[diagnostic_start..],
                );
                output.push_str(&artifact.html);
                output.push('\n');
                artifacts.push(artifact);
                continue;
            }
        }
        output.push_str(line);
        output.push('\n');
    }
    (output, artifacts)
}

fn attach_transform_source(
    artifact: &mut TransformArtifact,
    source_map: &[SourceMapEntry],
    generated_start_line: usize,
    generated_end_line: usize,
    diagnostics: &mut [DocumentDiagnostic],
) {
    if let Some(source) =
        ast_source_range_for_generated_lines(source_map, generated_start_line, generated_end_line)
    {
        artifact.source_file = Some(source.source_file);
        artifact.source_line = Some(source.source_line);
        artifact.end_source_line = Some(source.end_source_line);
    } else {
        artifact.source_line = Some(generated_start_line);
        artifact.end_source_line = Some(generated_end_line);
    }

    let source_file = artifact.source_file.clone();
    let source_line = artifact.source_line;
    let end_source_line = artifact.end_source_line;
    let transform_name = artifact.name.clone();
    for diagnostic in &mut artifact.diagnostics {
        attach_transform_diagnostic_source(
            diagnostic,
            &transform_name,
            source_file.as_deref(),
            source_line,
            end_source_line,
        );
    }
    for diagnostic in diagnostics {
        attach_transform_diagnostic_source(
            diagnostic,
            &transform_name,
            source_file.as_deref(),
            source_line,
            end_source_line,
        );
    }
}

fn attach_transform_diagnostic_source(
    diagnostic: &mut DocumentDiagnostic,
    transform_name: &str,
    source_file: Option<&str>,
    source_line: Option<usize>,
    end_source_line: Option<usize>,
) {
    let should_translate_relative_line = diagnostic.source_file.is_none();
    let had_line = diagnostic.line.is_some();
    if diagnostic.source_file.is_none() {
        diagnostic.source_file = source_file.map(ToString::to_string);
    }
    if should_translate_relative_line {
        if let (Some(source_line), Some(line)) = (source_line, diagnostic.line) {
            diagnostic.line = Some(source_line + line);
        }
        if let (Some(source_line), Some(end_line)) = (source_line, diagnostic.end_line) {
            diagnostic.end_line = Some(source_line + end_line);
        }
    }
    if diagnostic.line.is_none() {
        diagnostic.line = source_line;
    }
    if diagnostic.end_line.is_none() {
        diagnostic.end_line = if had_line {
            diagnostic.line
        } else {
            end_source_line
        };
    }
    if diagnostic.column.is_none() {
        diagnostic.column = Some(1);
    }
    if diagnostic.end_column.is_none() {
        diagnostic.end_column = Some(4);
    }
    push_related_once(diagnostic, format!("transform: {transform_name}"));
    if let Some(source_line) = source_line {
        let end_source_line = end_source_line.unwrap_or(source_line);
        push_related_once(
            diagnostic,
            format!("source range: {source_line}-{end_source_line}"),
        );
    }
}

fn push_related_once(diagnostic: &mut DocumentDiagnostic, value: String) {
    if !diagnostic.related.iter().any(|related| related == &value) {
        diagnostic.related.push(value);
    }
}

fn transform_fence_options(info: &str) -> Value {
    let mut fields = serde_json::Map::new();
    for token in transform_info_tokens(info).into_iter().skip(1) {
        if let Some((key, value)) = token.split_once('=') {
            let value = value.trim_matches(|ch| ch == '"' || ch == '\'');
            fields.insert(key.to_string(), Value::String(value.to_string()));
        } else if !token.is_empty() {
            fields.insert(token.to_string(), Value::Bool(true));
        }
    }
    Value::Object(fields)
}

fn transform_info_tokens(info: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut quote = None::<char>;
    for ch in info.chars() {
        if let Some(quote_ch) = quote {
            if ch == quote_ch {
                quote = None;
            } else {
                token.push(ch);
            }
        } else if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if ch.is_whitespace() {
            if !token.is_empty() {
                tokens.push(std::mem::take(&mut token));
            }
        } else {
            token.push(ch);
        }
    }
    if !token.is_empty() {
        tokens.push(token);
    }
    tokens
}
