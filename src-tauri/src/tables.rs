use crate::{
    calculations::eval_expression,
    compiler_support::fenced_code_marker,
    diagnostics::{diag, DocumentDiagnostic},
    document_ast::{DocumentAst, DocumentBlock},
    escape_html, format_value,
    markdown_tables::{
        is_markdown_table_row, is_markdown_table_separator, markdown_table_row,
        split_markdown_table_row,
    },
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
    let mut rows = parse_delimited_rows(body, delimiter);
    if rows.is_empty() {
        return "<table></table>".to_string();
    }
    evaluate_delimited_table_formula_rows(&mut rows, artifact_diags, diagnostics);
    let mut html = String::from("<table class=\"transform-table\"><thead><tr>");
    for cell in &rows[0] {
        html.push_str(&format!("<th>{}</th>", escape_html(cell)));
    }
    html.push_str("</tr></thead><tbody>");
    for row in rows.iter().skip(1) {
        html.push_str("<tr>");
        for cell in row {
            html.push_str(&format!("<td>{}</td>", escape_html(cell)));
        }
        html.push_str("</tr>");
    }
    html.push_str("</tbody></table>");
    html
}

pub(crate) fn delimited_rows_for_export(body: &str, delimiter: char) -> Vec<Vec<String>> {
    let mut rows = parse_delimited_rows(body, delimiter);
    let mut artifact_diags = Vec::new();
    let mut diagnostics = Vec::new();
    evaluate_delimited_table_formula_rows(&mut rows, &mut artifact_diags, &mut diagnostics);
    rows
}

fn evaluate_delimited_table_formula_rows(
    rows: &mut [Vec<String>],
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    if rows.is_empty() {
        return;
    }
    let named_tables = HashMap::new();
    evaluate_table_formula_rows(
        rows,
        TableFormulaEvaluation {
            data_start: 1,
            display_line_for_row: |row_index| row_index + 1,
            diagnostic_prefix: "Table formula error",
            suggestion: "Use numeric formulas such as =SUM(1,2) or =SUM(A1:A3) in CSV/TSV cells.",
            named_tables: &named_tables,
            artifact_diags,
            diagnostics,
        },
    );
}

pub(crate) fn parse_delimited_rows(body: &str, delimiter: char) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut cell = String::new();
    let mut quoted = false;
    let mut chars = body.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '"' && quoted && chars.peek() == Some(&'"') {
            cell.push('"');
            chars.next();
        } else if ch == '"' {
            quoted = !quoted;
        } else if ch == delimiter && !quoted {
            row.push(cell.trim().to_string());
            cell.clear();
        } else if matches!(ch, '\n' | '\r') && !quoted {
            if ch == '\r' && chars.peek() == Some(&'\n') {
                chars.next();
            }
            push_delimited_row(&mut rows, &mut row, &mut cell);
        } else {
            cell.push(ch);
        }
    }
    push_delimited_row(&mut rows, &mut row, &mut cell);
    rows
}

fn push_delimited_row(rows: &mut Vec<Vec<String>>, row: &mut Vec<String>, cell: &mut String) {
    row.push(cell.trim().to_string());
    cell.clear();
    if row.iter().any(|value| !value.trim().is_empty()) {
        rows.push(std::mem::take(row));
    } else {
        row.clear();
    }
}

pub(crate) fn evaluate_markdown_table_formulas(
    markdown: &str,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let lines = markdown.lines().collect::<Vec<_>>();
    let mut output = Vec::new();
    let mut index = 0;
    let mut named_tables = HashMap::new();
    let mut fence_marker = None;
    while index < lines.len() {
        let line = lines[index];
        if let Some(marker) = fence_marker {
            output.push(line.to_string());
            if line.trim_start().starts_with(marker) {
                fence_marker = None;
            }
            index += 1;
            continue;
        }
        if let Some(marker) = fenced_code_marker(line) {
            output.push(line.to_string());
            fence_marker = Some(marker);
            index += 1;
            continue;
        }
        if index + 1 >= lines.len()
            || !is_markdown_table_row(line.trim())
            || !is_markdown_table_separator(lines[index + 1].trim())
        {
            output.push(line.to_string());
            index += 1;
            continue;
        }

        let table_start = index;
        let table_id = output.last().and_then(|line| table_id_from_caption(line));
        let header = lines[index].to_string();
        let separator = lines[index + 1].to_string();
        index += 2;
        let mut row_lines = Vec::new();
        while index < lines.len() && is_markdown_table_row(lines[index].trim()) {
            row_lines.push((index, lines[index].to_string()));
            index += 1;
        }

        let mut rows = Vec::with_capacity(row_lines.len() + 1);
        rows.push(split_markdown_table_row(header.trim()));
        rows.extend(
            row_lines
                .iter()
                .map(|(_, row)| split_markdown_table_row(row.trim())),
        );
        let mut artifact_diags = Vec::new();
        let changed = evaluate_table_formula_rows(
            &mut rows,
            TableFormulaEvaluation {
                data_start: 1,
                display_line_for_row: |row_index| table_start + row_index + 1,
                diagnostic_prefix: "Markdown table formula error",
                suggestion: "Use numeric formulas such as =10+15, =SUM(1,2), or =SUM(A1:A3).",
                named_tables: &named_tables,
                artifact_diags: &mut artifact_diags,
                diagnostics,
            },
        );

        output.push(header);
        output.push(separator);
        if changed {
            output.extend(rows.iter().skip(1).map(|row| markdown_table_row(row)));
        } else {
            output.extend(row_lines.into_iter().map(|(_, row)| row));
        }
        register_named_table(&mut named_tables, table_id.as_deref(), &rows);
    }
    output.join("\n")
}

pub(crate) fn collect_table_summaries(document_ast: &DocumentAst) -> Vec<TableSummary> {
    let mut tables = Vec::new();
    for block in &document_ast.blocks {
        if let DocumentBlock::Table {
            line,
            headers,
            rows,
            ..
        } = block
        {
            let mut numeric_columns = headers
                .iter()
                .map(|column| (column.clone(), 0.0))
                .collect::<BTreeMap<_, _>>();
            for row in rows {
                for (column_index, cell) in row.iter().enumerate() {
                    if let Some(column) = headers.get(column_index) {
                        if let Ok(value) = cell.replace([',', '$', '%'], "").parse::<f64>() {
                            *numeric_columns.entry(column.clone()).or_insert(0.0) += value;
                        }
                    }
                }
            }
            numeric_columns.retain(|_, value| *value != 0.0);
            tables.push(TableSummary {
                line: *line,
                columns: headers.clone(),
                rows: rows.len(),
                numeric_columns,
            });
        }
    }
    tables
}

type CellReference = (usize, usize);
type ParsedCellReference = Option<(CellReference, usize)>;

struct TableFormulaEvaluation<'a, F>
where
    F: Fn(usize) -> usize,
{
    data_start: usize,
    display_line_for_row: F,
    diagnostic_prefix: &'a str,
    suggestion: &'static str,
    named_tables: &'a HashMap<String, Vec<Vec<String>>>,
    artifact_diags: &'a mut Vec<DocumentDiagnostic>,
    diagnostics: &'a mut Vec<DocumentDiagnostic>,
}

fn evaluate_table_formula_rows<F>(
    rows: &mut [Vec<String>],
    options: TableFormulaEvaluation<'_, F>,
) -> bool
where
    F: Fn(usize) -> usize,
{
    let mut changed = false;
    let source_rows = rows.to_vec();
    let mut context = TableFormulaContext::new(&source_rows, options.named_tables);
    for (row_index, row) in rows.iter_mut().enumerate().skip(options.data_start) {
        for (column_index, cell) in row.iter_mut().enumerate() {
            if !cell.starts_with('=') {
                continue;
            }
            changed = true;
            match table_cell_value(&mut context, (row_index, column_index)) {
                Ok(value) => *cell = format_value(value, "round"),
                Err(error) => {
                    let line = (options.display_line_for_row)(row_index);
                    let diagnostic = diag(
                        "error",
                        format!("{} on row {line}: {error}", options.diagnostic_prefix),
                        None,
                        Some(line),
                        Some(options.suggestion),
                    );
                    options.artifact_diags.push(diagnostic.clone());
                    options.diagnostics.push(diagnostic);
                    *cell = "#ERROR".to_string();
                }
            }
        }
    }
    changed
}

struct TableFormulaContext<'a> {
    rows: &'a [Vec<String>],
    named_tables: &'a HashMap<String, Vec<Vec<String>>>,
    cache: HashMap<(usize, usize), Result<f64, String>>,
    stack: Vec<(usize, usize)>,
}

impl<'a> TableFormulaContext<'a> {
    fn new(rows: &'a [Vec<String>], named_tables: &'a HashMap<String, Vec<Vec<String>>>) -> Self {
        Self {
            rows,
            named_tables,
            cache: HashMap::new(),
            stack: Vec::new(),
        }
    }
}

fn evaluate_table_formula_number(
    expression: &str,
    context: &mut TableFormulaContext<'_>,
) -> Result<f64, String> {
    let expanded = expand_table_references(expression, context)?;
    eval_expression(&expanded, &HashMap::new())
}

fn expand_table_references(
    expression: &str,
    context: &mut TableFormulaContext<'_>,
) -> Result<String, String> {
    let chars = expression.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0;
    while index < chars.len() {
        if let Some((replacement, next_index)) =
            expand_named_table_reference(&chars, index, context)?
        {
            output.push_str(&replacement);
            index = next_index;
            continue;
        }

        if !chars[index].is_ascii_alphabetic() {
            output.push(chars[index]);
            index += 1;
            continue;
        }

        let start = index;
        while index < chars.len() && chars[index].is_ascii_alphabetic() {
            index += 1;
        }
        let letters_end = index;
        while index < chars.len() && chars[index].is_ascii_digit() {
            index += 1;
        }
        if letters_end == index {
            output.extend(chars[start..index].iter());
            continue;
        }

        let first_ref = cell_ref_from_parts(&chars[start..letters_end], &chars[letters_end..index])
            .ok_or_else(|| "#REF?".to_string())?;
        if chars.get(index) == Some(&':') {
            let range_start = index + 1;
            let mut range_index = range_start;
            while range_index < chars.len() && chars[range_index].is_ascii_alphabetic() {
                range_index += 1;
            }
            let range_letters_end = range_index;
            while range_index < chars.len() && chars[range_index].is_ascii_digit() {
                range_index += 1;
            }
            if range_letters_end == range_index {
                return Err("#REF?".to_string());
            }
            let second_ref = cell_ref_from_parts(
                &chars[range_start..range_letters_end],
                &chars[range_letters_end..range_index],
            )
            .ok_or_else(|| "#REF?".to_string())?;
            output.push_str(&table_range_values(context, first_ref, second_ref)?.join(","));
            index = range_index;
        } else {
            output.push_str(&table_cell_value(context, first_ref)?.to_string());
        }
    }
    Ok(output)
}

fn expand_named_table_reference(
    chars: &[char],
    index: usize,
    context: &mut TableFormulaContext<'_>,
) -> Result<Option<(String, usize)>, String> {
    if !(chars[index].is_ascii_alphabetic() || chars[index] == '_') {
        return Ok(None);
    }
    let mut name_end = index;
    while name_end < chars.len()
        && (chars[name_end].is_ascii_alphanumeric() || matches!(chars[name_end], '_' | '-' | ':'))
    {
        name_end += 1;
    }
    if chars.get(name_end) != Some(&'!') {
        return Ok(None);
    }
    let name = chars[index..name_end].iter().collect::<String>();
    let rows = context
        .named_tables
        .get(&name)
        .ok_or_else(|| format!("#NAME? {name}"))?;
    let (first_ref, mut next_index) =
        parse_cell_reference_at(chars, name_end + 1)?.ok_or_else(|| "#REF?".to_string())?;
    if chars.get(next_index) == Some(&':') {
        let (second_ref, range_end) =
            parse_cell_reference_at(chars, next_index + 1)?.ok_or_else(|| "#REF?".to_string())?;
        next_index = range_end;
        return Ok(Some((
            table_range_values_from_rows(rows, first_ref, second_ref)?.join(","),
            next_index,
        )));
    }
    Ok(Some((
        table_cell_value_from_rows(rows, first_ref)?.to_string(),
        next_index,
    )))
}

fn parse_cell_reference_at(
    chars: &[char],
    mut index: usize,
) -> Result<ParsedCellReference, String> {
    let start = index;
    while index < chars.len() && chars[index].is_ascii_alphabetic() {
        index += 1;
    }
    let letters_end = index;
    while index < chars.len() && chars[index].is_ascii_digit() {
        index += 1;
    }
    if letters_end == start || letters_end == index {
        return Ok(None);
    }
    let reference = cell_ref_from_parts(&chars[start..letters_end], &chars[letters_end..index])
        .ok_or_else(|| "#REF?".to_string())?;
    Ok(Some((reference, index)))
}

fn cell_ref_from_parts(column: &[char], row: &[char]) -> Option<(usize, usize)> {
    let column_index = column.iter().try_fold(0usize, |acc, ch| {
        let upper = ch.to_ascii_uppercase();
        if !upper.is_ascii_uppercase() {
            return None;
        }
        Some(acc * 26 + (upper as u8 - b'A' + 1) as usize)
    })?;
    let row_index = row.iter().collect::<String>().parse::<usize>().ok()?;
    if column_index == 0 || row_index == 0 {
        return None;
    }
    Some((row_index, column_index - 1))
}

fn table_range_values(
    context: &mut TableFormulaContext<'_>,
    first: (usize, usize),
    second: (usize, usize),
) -> Result<Vec<String>, String> {
    let row_start = first.0.min(second.0);
    let row_end = first.0.max(second.0);
    let column_start = first.1.min(second.1);
    let column_end = first.1.max(second.1);
    let mut values = Vec::new();
    for row in row_start..=row_end {
        for column in column_start..=column_end {
            values.push(table_cell_value(context, (row, column))?.to_string());
        }
    }
    Ok(values)
}

fn table_range_values_from_rows(
    rows: &[Vec<String>],
    first: (usize, usize),
    second: (usize, usize),
) -> Result<Vec<String>, String> {
    let row_start = first.0.min(second.0);
    let row_end = first.0.max(second.0);
    let column_start = first.1.min(second.1);
    let column_end = first.1.max(second.1);
    let mut values = Vec::new();
    for row in row_start..=row_end {
        for column in column_start..=column_end {
            values.push(table_cell_value_from_rows(rows, (row, column))?.to_string());
        }
    }
    Ok(values)
}

fn table_cell_value(
    context: &mut TableFormulaContext<'_>,
    reference: (usize, usize),
) -> Result<f64, String> {
    if let Some(value) = context.cache.get(&reference).cloned() {
        return value;
    }
    if context.stack.contains(&reference) {
        return Err(table_cycle_error(reference, &context.stack));
    }
    let row = context
        .rows
        .get(reference.0)
        .ok_or_else(|| "#REF?".to_string())?;
    let cell = row.get(reference.1).ok_or_else(|| "#REF?".to_string())?;
    let result = if let Some(expression) = cell.trim().strip_prefix('=') {
        context.stack.push(reference);
        let result = evaluate_table_formula_number(expression, context);
        context.stack.pop();
        result
    } else {
        parse_table_number(cell).ok_or_else(|| "#VALUE?".to_string())
    };
    context.cache.insert(reference, result.clone());
    result
}

fn table_cell_value_from_rows(
    rows: &[Vec<String>],
    reference: (usize, usize),
) -> Result<f64, String> {
    let row = rows.get(reference.0).ok_or_else(|| "#REF?".to_string())?;
    let cell = row.get(reference.1).ok_or_else(|| "#REF?".to_string())?;
    parse_table_number(cell).ok_or_else(|| "#VALUE?".to_string())
}

fn table_cycle_error(reference: (usize, usize), stack: &[(usize, usize)]) -> String {
    let start = stack
        .iter()
        .position(|candidate| *candidate == reference)
        .unwrap_or(0);
    let mut path = stack[start..]
        .iter()
        .map(|candidate| table_cell_label(*candidate))
        .collect::<Vec<_>>();
    path.push(table_cell_label(reference));
    format!("#CYCLE? {}", path.join(" -> "))
}

fn table_cell_label(reference: (usize, usize)) -> String {
    format!("{}{}", table_column_label(reference.1), reference.0)
}

fn table_column_label(mut column: usize) -> String {
    column += 1;
    let mut label = String::new();
    while column > 0 {
        let remainder = (column - 1) % 26;
        label.insert(0, (b'A' + remainder as u8) as char);
        column = (column - 1) / 26;
    }
    label
}

fn parse_table_number(value: &str) -> Option<f64> {
    value
        .trim()
        .trim_start_matches('=')
        .replace([',', '$', '%'], "")
        .parse::<f64>()
        .ok()
}

fn register_named_table(
    named_tables: &mut HashMap<String, Vec<Vec<String>>>,
    table_id: Option<&str>,
    rows: &[Vec<String>],
) {
    let Some(table_id) = table_id else {
        return;
    };
    named_tables.insert(table_id.to_string(), rows.to_vec());
    if let Some((_, short_name)) = table_id.split_once(':') {
        if !short_name.is_empty() {
            named_tables.insert(short_name.to_string(), rows.to_vec());
        }
    }
}

fn table_id_from_caption(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.to_ascii_lowercase().starts_with("table:") {
        return None;
    }
    let (_, after) = trimmed.split_once("{#")?;
    let id = after
        .split(['}', ' ', '\t'])
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    (!id.is_empty()).then_some(id)
}
