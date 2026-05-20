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
            let mut missing_variable_path = None;
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
                let variable = parse_variable_token(token);
                let replacement = metadata_lookup(metadata, variable.path)
                    .map(value_to_string)
                    .or_else(|| variable.default.clone());
                if replacement.is_none() {
                    missing_variable_path = Some(variable.path);
                }
                replacement.map(|value| {
                    apply_variable_filters(
                        variable.path,
                        value,
                        &variable.filters,
                        source_map,
                        token_line,
                        token_column,
                        token_end_line,
                        token_end_column,
                        diagnostics,
                    )
                })
            };
            if let Some(value) = replacement {
                output.push_str(&value);
            } else if formula_token || is_passthrough_token(token) {
                output.push_str(&format!("{{{{{token}}}}}"));
            } else {
                let (source_file, line) =
                    diagnostic_location_for_generated_line(source_map, token_line);
                let end_line = diagnostic_end_line(line, token_line, token_end_line);
                diagnostics.push(with_range(
                    diag(
                        "warning",
                        format!(
                            "Missing document variable: {}",
                            missing_variable_path.unwrap_or(token)
                        ),
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

fn is_passthrough_token(token: &str) -> bool {
    matches!(token, "page" | "pages" | "page-break")
        || token.starts_with("section-break")
        || token.starts_with("slide")
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

struct VariableToken<'a> {
    path: &'a str,
    default: Option<String>,
    filters: Vec<&'a str>,
}

fn parse_variable_token(token: &str) -> VariableToken<'_> {
    let mut parts = token.split('|').map(str::trim);
    let path = parts.next().unwrap_or("").trim();
    let mut default = None;
    let mut filters = Vec::new();
    for filter in parts {
        if filter.is_empty() {
            continue;
        }
        if default.is_none() {
            default = filter
                .strip_prefix("default:")
                .or_else(|| filter.strip_prefix("default="))
                .or_else(|| filter.strip_prefix("default "))
                .map(unquote_variable_default);
            if default.is_some() {
                continue;
            }
        }
        filters.push(filter);
    }
    VariableToken {
        path,
        default,
        filters,
    }
}

fn unquote_variable_default(value: &str) -> String {
    value
        .trim()
        .trim_matches(|ch| ch == '"' || ch == '\'')
        .to_string()
}

#[allow(clippy::too_many_arguments)]
fn apply_variable_filters(
    path: &str,
    mut value: String,
    filters: &[&str],
    source_map: &[SourceMapEntry],
    token_line: usize,
    token_column: usize,
    token_end_line: usize,
    token_end_column: usize,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    for filter in filters {
        value = match variable_filter_name(filter) {
            "trim" => value.trim().to_string(),
            "upper" | "uppercase" => value.to_ascii_uppercase(),
            "lower" | "lowercase" => value.to_ascii_lowercase(),
            "title" | "titlecase" => title_case(&value),
            "round" | "number" | "percent" | "currency" => match parse_variable_number(&value) {
                Some(number) => format_value(number, variable_filter_name(filter)),
                None => {
                    push_variable_filter_diagnostic(
                        path,
                        filter,
                        "numeric",
                        source_map,
                        token_line,
                        token_column,
                        token_end_line,
                        token_end_column,
                        diagnostics,
                    );
                    value
                }
            },
            _ => {
                push_variable_filter_diagnostic(
                    path,
                    filter,
                    "supported",
                    source_map,
                    token_line,
                    token_column,
                    token_end_line,
                    token_end_column,
                    diagnostics,
                );
                value
            }
        };
    }
    value
}

fn variable_filter_name(filter: &str) -> &str {
    filter
        .split_once(':')
        .map(|(name, _)| name.trim())
        .or_else(|| filter.split_once('=').map(|(name, _)| name.trim()))
        .unwrap_or(filter.trim())
}

fn parse_variable_number(value: &str) -> Option<f64> {
    value
        .trim()
        .trim_start_matches('$')
        .trim_end_matches('%')
        .replace(',', "")
        .parse::<f64>()
        .ok()
}

fn title_case(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            let mut output = first.to_uppercase().collect::<String>();
            output.push_str(&chars.as_str().to_ascii_lowercase());
            output
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[allow(clippy::too_many_arguments)]
fn push_variable_filter_diagnostic(
    path: &str,
    filter: &str,
    kind: &str,
    source_map: &[SourceMapEntry],
    token_line: usize,
    token_column: usize,
    token_end_line: usize,
    token_end_column: usize,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let (source_file, line) = diagnostic_location_for_generated_line(source_map, token_line);
    let end_line = diagnostic_end_line(line, token_line, token_end_line);
    let (message, suggestion) = if kind == "numeric" {
        (
            format!("Cannot apply numeric document variable filter '{filter}' to {path}."),
            "Use numeric values for percent, currency, round, and number filters.",
        )
    } else {
        (
            format!("Unsupported document variable filter '{filter}' for {path}."),
            "Use supported filters: default, trim, upper, lower, title, number, round, percent, currency.",
        )
    };
    diagnostics.push(with_range(
        diag("warning", message, source_file, line, Some(suggestion)),
        token_column,
        end_line,
        token_end_column,
    ));
}
