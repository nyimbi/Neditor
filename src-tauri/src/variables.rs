use crate::{
    calculations::eval_expression,
    diagnostics::{diag, with_range, DocumentDiagnostic},
    format_value, metadata_lookup,
    source_mapping::{
        advance_source_position, diagnostic_end_line, diagnostic_location_for_generated_line,
        source_position_after,
    },
    value_to_string, SourceMapEntry,
};
use serde_json::Value;
use std::collections::HashMap;

pub(crate) fn interpolate_variables(
    text: &str,
    metadata: &Value,
    calculations: &HashMap<String, f64>,
    source_map: &[SourceMapEntry],
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let mut output = String::new();
    let mut rest = text;
    let mut generated_line = 1usize;
    let mut generated_column = 1usize;
    while let Some(start) = rest.find("{{") {
        let (before, after_start) = rest.split_at(start);
        output.push_str(before);
        advance_source_position(&mut generated_line, &mut generated_column, before);
        if let Some(end) = after_start.find("}}") {
            let token = after_start[2..end].trim();
            let token_line = generated_line;
            let token_column = generated_column;
            let token_source = &after_start[..end + 2];
            let (token_end_line, token_end_column) =
                source_position_after(token_line, token_column, token_source);
            let mut formula_token = false;
            let replacement = if let Some(expr) = token.strip_prefix('=') {
                formula_token = true;
                match evaluate_inline_formula(expr, calculations) {
                    Ok(value) => Some(value),
                    Err(error) => {
                        let (source_file, line) =
                            diagnostic_location_for_generated_line(source_map, token_line);
                        let end_line = diagnostic_end_line(line, token_line, token_end_line);
                        diagnostics.push(with_range(
                            diag(
                                "error",
                                format!("Inline formula error for {{{{{token}}}}}: {error}"),
                                source_file,
                                line,
                                Some("Use numeric expressions, supported functions, and names defined in calc blocks."),
                            ),
                            token_column,
                            end_line,
                            token_end_column,
                        ));
                        None
                    }
                }
            } else {
                let (path, default_value) = variable_path_and_default(token);
                metadata_lookup(metadata, path)
                    .map(value_to_string)
                    .or(default_value)
            };
            if let Some(value) = replacement {
                output.push_str(&value);
            } else if formula_token || matches!(token, "page" | "pages") {
                output.push_str(&format!("{{{{{token}}}}}"));
            } else {
                let (source_file, line) =
                    diagnostic_location_for_generated_line(source_map, token_line);
                let end_line = diagnostic_end_line(line, token_line, token_end_line);
                diagnostics.push(with_range(
                    diag(
                        "warning",
                        format!("Missing document variable: {token}"),
                        source_file,
                        line,
                        Some("Define the variable in front matter or a calc block."),
                    ),
                    token_column,
                    end_line,
                    token_end_column,
                ));
                output.push_str(&format!("{{{{{token}}}}}"));
            }
            advance_source_position(&mut generated_line, &mut generated_column, token_source);
            rest = &after_start[end + 2..];
        } else {
            output.push_str(after_start);
            rest = "";
        }
    }
    output.push_str(rest);
    output
}

fn evaluate_inline_formula(
    expression: &str,
    calculations: &HashMap<String, f64>,
) -> Result<String, String> {
    let expression = expression.trim();
    let (formula, filter) = expression
        .split_once('|')
        .map(|(formula, filter)| (formula.trim(), filter.trim()))
        .unwrap_or((expression, ""));
    eval_expression(formula, calculations).map(|value| format_value(value, filter))
}

fn variable_path_and_default(token: &str) -> (&str, Option<String>) {
    let Some((path, filter)) = token.split_once('|') else {
        return (token.trim(), None);
    };
    let filter = filter.trim();
    let default = filter
        .strip_prefix("default:")
        .or_else(|| filter.strip_prefix("default="))
        .or_else(|| filter.strip_prefix("default "))
        .map(unquote_variable_default);
    (path.trim(), default)
}

fn unquote_variable_default(value: &str) -> String {
    value
        .trim()
        .trim_matches(|ch| ch == '"' || ch == '\'')
        .to_string()
}
