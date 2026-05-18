use crate::{
    calculations::eval_expression,
    diagnostics::{diag, DocumentDiagnostic},
    escape_html, format_value,
};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Serialize)]
pub(crate) struct TableSummary {
    pub(crate) line: usize,
    pub(crate) columns: Vec<String>,
    pub(crate) rows: usize,
    pub(crate) numeric_columns: BTreeMap<String, f64>,
}

pub(crate) fn render_delimited_table(
    body: &str,
    delimiter: char,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let rows = body
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(line_index, line)| {
            line.split(delimiter)
                .map(|cell| {
                    render_table_cell(cell.trim(), line_index + 1, artifact_diags, diagnostics)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return "<table></table>".to_string();
    }
    let mut html = String::from("<table class=\"transform-table\"><thead><tr>");
    for cell in &rows[0] {
        html.push_str(&format!("<th>{cell}</th>"));
    }
    html.push_str("</tr></thead><tbody>");
    for row in rows.iter().skip(1) {
        html.push_str("<tr>");
        for cell in row {
            html.push_str(&format!("<td>{cell}</td>"));
        }
        html.push_str("</tr>");
    }
    html.push_str("</tbody></table>");
    html
}

pub(crate) fn evaluate_markdown_table_formulas(
    markdown: &str,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    markdown
        .lines()
        .enumerate()
        .map(|(line_index, line)| {
            let trimmed = line.trim();
            if !is_markdown_table_row(trimmed) || is_markdown_table_separator(trimmed) {
                return line.to_string();
            }

            let mut changed = false;
            let cells = split_table_row(trimmed)
                .into_iter()
                .map(|cell| {
                    let Some(expression) = cell.strip_prefix('=') else {
                        return cell;
                    };
                    changed = true;
                    match evaluate_table_formula(expression) {
                        Ok(value) => value,
                        Err(error) => {
                            diagnostics.push(diag(
                                "error",
                                format!(
                                    "Markdown table formula error on row {}: {error}",
                                    line_index + 1
                                ),
                                None,
                                Some(line_index + 1),
                                Some("Use numeric formulas such as =10+15 or =SUM(1,2)."),
                            ));
                            "#ERROR".to_string()
                        }
                    }
                })
                .collect::<Vec<_>>();

            if changed {
                format!("| {} |", cells.join(" | "))
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn collect_table_summaries(text: &str) -> Vec<TableSummary> {
    let lines = text.lines().collect::<Vec<_>>();
    let mut tables = Vec::new();
    let mut index = 0;
    while index + 1 < lines.len() {
        let header = lines[index].trim();
        let separator = lines[index + 1].trim();
        if is_markdown_table_row(header) && is_markdown_table_separator(separator) {
            let columns = split_table_row(header);
            let mut row_count = 0usize;
            let mut numeric_columns = columns
                .iter()
                .map(|column| (column.clone(), 0.0))
                .collect::<BTreeMap<_, _>>();
            index += 2;
            while index < lines.len() && is_markdown_table_row(lines[index].trim()) {
                let cells = split_table_row(lines[index].trim());
                for (column_index, cell) in cells.iter().enumerate() {
                    if let Some(column) = columns.get(column_index) {
                        if let Ok(value) = cell.replace([',', '$', '%'], "").parse::<f64>() {
                            *numeric_columns.entry(column.clone()).or_insert(0.0) += value;
                        }
                    }
                }
                row_count += 1;
                index += 1;
            }
            numeric_columns.retain(|_, value| *value != 0.0);
            tables.push(TableSummary {
                line: index.saturating_sub(row_count + 1),
                columns,
                rows: row_count,
                numeric_columns,
            });
        } else {
            index += 1;
        }
    }
    tables
}

fn render_table_cell(
    cell: &str,
    line: usize,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let Some(expression) = cell.strip_prefix('=') else {
        return escape_html(cell);
    };
    match evaluate_table_formula(expression) {
        Ok(value) => escape_html(&value),
        Err(error) => {
            let diagnostic = diag(
                "error",
                format!("Table formula error on row {line}: {error}"),
                None,
                Some(line),
                Some("Use numeric formulas such as =SUM(1,2) in CSV/TSV cells."),
            );
            artifact_diags.push(diagnostic.clone());
            diagnostics.push(diagnostic);
            "#ERROR".to_string()
        }
    }
}

fn evaluate_table_formula(expression: &str) -> Result<String, String> {
    eval_expression(expression, &HashMap::new()).map(|value| format_value(value, "round"))
}

fn is_markdown_table_row(line: &str) -> bool {
    line.starts_with('|') && line.ends_with('|') && line.matches('|').count() >= 2
}

fn is_markdown_table_separator(line: &str) -> bool {
    is_markdown_table_row(line)
        && line
            .trim_matches('|')
            .split('|')
            .all(|cell| cell.trim().chars().all(|ch| matches!(ch, '-' | ':' | ' ')))
}

fn split_table_row(line: &str) -> Vec<String> {
    line.trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}
