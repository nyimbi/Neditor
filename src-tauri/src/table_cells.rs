use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct TableCell {
    pub(crate) text: String,
    pub(crate) colspan: usize,
    pub(crate) rowspan: usize,
    pub(crate) covered: bool,
    pub(crate) continues_rowspan: bool,
}

pub(crate) fn table_cell_from_markdown(raw: &str, clean: impl Fn(&str) -> String) -> TableCell {
    let (text, colspan, rowspan) = split_table_cell_span_attributes(raw);
    TableCell {
        text: clean(&text),
        colspan,
        rowspan,
        covered: false,
        continues_rowspan: false,
    }
}

pub(crate) fn html_table_cell(
    text: String,
    open_tag: &str,
    colspan_name: &str,
    rowspan_name: &str,
) -> TableCell {
    TableCell {
        text,
        colspan: html_span_attribute(open_tag, colspan_name).unwrap_or(1),
        rowspan: html_span_attribute(open_tag, rowspan_name).unwrap_or(1),
        covered: false,
        continues_rowspan: false,
    }
}

pub(crate) fn normalize_table_cell_rows(raw_rows: &[Vec<TableCell>]) -> Vec<Vec<TableCell>> {
    let mut rows = Vec::new();
    let mut active_rowspans: Vec<usize> = Vec::new();
    for raw_row in raw_rows {
        let mut row = Vec::new();
        let mut column_index = 0usize;
        for raw_cell in raw_row.iter().cloned() {
            while active_rowspans
                .get(column_index)
                .is_some_and(|remaining| *remaining > 0)
            {
                row.push(covered_table_cell(true));
                active_rowspans[column_index] = active_rowspans[column_index].saturating_sub(1);
                column_index += 1;
            }
            let colspan = raw_cell.colspan.max(1);
            let rowspan = raw_cell.rowspan.max(1);
            if active_rowspans.len() < column_index + colspan {
                active_rowspans.resize(column_index + colspan, 0);
            }
            if rowspan > 1 {
                for offset in 0..colspan {
                    active_rowspans[column_index + offset] = rowspan - 1;
                }
            }
            row.push(raw_cell);
            for _ in 1..colspan {
                row.push(covered_table_cell(false));
            }
            column_index += colspan;
        }
        while active_rowspans
            .get(column_index)
            .is_some_and(|remaining| *remaining > 0)
        {
            row.push(covered_table_cell(true));
            active_rowspans[column_index] = active_rowspans[column_index].saturating_sub(1);
            column_index += 1;
        }
        rows.push(row);
    }
    rows
}

pub(crate) fn plain_table_cells(cells: &[String]) -> Vec<TableCell> {
    cells
        .iter()
        .map(|cell| TableCell {
            text: cell.clone(),
            colspan: 1,
            rowspan: 1,
            covered: false,
            continues_rowspan: false,
        })
        .collect()
}

pub(crate) fn table_cell_texts(cells: &[TableCell]) -> Vec<String> {
    cells
        .iter()
        .map(|cell| {
            if cell.covered {
                String::new()
            } else {
                cell.text.clone()
            }
        })
        .collect()
}

pub(crate) fn markdown_table_rows(
    headers: &[String],
    alignments: &[String],
    header_cells: &[TableCell],
    rows: &[Vec<String>],
    row_cells: &[Vec<TableCell>],
) -> Vec<String> {
    if headers.is_empty() {
        return Vec::new();
    }
    let header_cells = populated_table_cells(header_cells, headers);
    let mut output = vec![
        markdown_table_row(&header_cells),
        markdown_alignment_row(alignments, header_cells.len()),
    ];
    let row_cells = populated_table_row_cells(row_cells, rows);
    output.extend(row_cells.iter().map(|row| markdown_table_row(row)));
    output
}

fn populated_table_cells(cells: &[TableCell], fallback: &[String]) -> Vec<TableCell> {
    if cells.is_empty() {
        plain_table_cells(fallback)
    } else {
        cells.to_vec()
    }
}

fn populated_table_row_cells(
    cells: &[Vec<TableCell>],
    fallback: &[Vec<String>],
) -> Vec<Vec<TableCell>> {
    if cells.is_empty() {
        fallback.iter().map(|row| plain_table_cells(row)).collect()
    } else {
        cells.to_vec()
    }
}

fn markdown_table_row(cells: &[TableCell]) -> String {
    let cells = cells
        .iter()
        .map(markdown_table_cell)
        .collect::<Vec<_>>()
        .join(" | ");
    format!("| {cells} |")
}

fn markdown_alignment_row(alignments: &[String], width: usize) -> String {
    let cells = (0..width)
        .map(|index| match alignments.get(index).map(String::as_str) {
            Some("center") => ":---:",
            Some("right") => "---:",
            _ => "---",
        })
        .collect::<Vec<_>>()
        .join(" | ");
    format!("| {cells} |")
}

fn markdown_table_cell(cell: &TableCell) -> String {
    if cell.covered {
        return String::new();
    }
    let mut text = cell
        .text
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('\n', " ");
    let mut attrs = Vec::new();
    if cell.colspan > 1 {
        attrs.push(format!("colspan={}", cell.colspan));
    }
    if cell.rowspan > 1 {
        attrs.push(format!("rowspan={}", cell.rowspan));
    }
    if !attrs.is_empty() {
        text.push_str(" {");
        text.push_str(&attrs.join(" "));
        text.push('}');
    }
    text
}

fn covered_table_cell(continues_rowspan: bool) -> TableCell {
    TableCell {
        text: String::new(),
        colspan: 1,
        rowspan: 1,
        covered: true,
        continues_rowspan,
    }
}

fn split_table_cell_span_attributes(raw: &str) -> (String, usize, usize) {
    let trimmed = raw.trim();
    let Some(open_index) = trimmed.rfind('{') else {
        return (trimmed.to_string(), 1, 1);
    };
    if !trimmed.ends_with('}') {
        return (trimmed.to_string(), 1, 1);
    }
    let attrs = &trimmed[open_index + 1..trimmed.len() - 1];
    if !attrs.contains("colspan") && !attrs.contains("rowspan") {
        return (trimmed.to_string(), 1, 1);
    }
    let colspan = table_span_attribute(attrs, "colspan").unwrap_or(1);
    let rowspan = table_span_attribute(attrs, "rowspan").unwrap_or(1);
    (trimmed[..open_index].trim().to_string(), colspan, rowspan)
}

fn table_span_attribute(attrs: &str, name: &str) -> Option<usize> {
    attrs
        .split_whitespace()
        .find_map(|part| part.strip_prefix(&format!("{name}=")))
        .and_then(|value| value.trim_matches('"').parse::<usize>().ok())
        .filter(|value| *value > 1)
}

fn html_span_attribute(open_tag: &str, name: &str) -> Option<usize> {
    html_quoted_attribute(open_tag, name)
        .or_else(|| {
            let marker = format!("{name}=");
            let value = open_tag.split(&marker).nth(1)?;
            Some(
                value
                    .split(|ch: char| ch == '>' || ch.is_whitespace())
                    .next()
                    .unwrap_or("")
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string(),
            )
        })
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 1)
}

fn html_quoted_attribute(text: &str, key: &str) -> Option<String> {
    let marker = format!("{key}=\"");
    let after_marker = text.split(&marker).nth(1)?;
    let (value, _) = after_marker.split_once('"')?;
    Some(value.to_string())
}
