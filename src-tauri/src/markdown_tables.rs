pub(crate) fn is_markdown_table_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed.ends_with('|') && unescaped_pipe_count(trimmed) >= 2
}

pub(crate) fn is_markdown_table_separator(line: &str) -> bool {
    is_markdown_table_row(line)
        && split_markdown_table_row(line)
            .iter()
            .all(|cell| is_markdown_table_separator_cell(cell))
}

pub(crate) fn split_markdown_table_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut escaped = false;

    for ch in inner.chars() {
        if escaped {
            if ch == '|' {
                cell.push('|');
            } else {
                cell.push('\\');
                cell.push(ch);
            }
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
        } else if ch == '|' {
            cells.push(cell.trim().to_string());
            cell.clear();
        } else {
            cell.push(ch);
        }
    }

    if escaped {
        cell.push('\\');
    }
    cells.push(cell.trim().to_string());
    cells
}

pub(crate) fn markdown_table_row(cells: &[String]) -> String {
    format!(
        "| {} |",
        cells
            .iter()
            .map(|cell| escape_markdown_table_cell(cell))
            .collect::<Vec<_>>()
            .join(" | ")
    )
}

pub(crate) fn escape_markdown_table_cell(cell: &str) -> String {
    cell.replace(['\n', '\r'], " ").replace('|', "\\|")
}

fn unescaped_pipe_count(line: &str) -> usize {
    let mut escaped = false;
    let mut count = 0;
    for ch in line.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
        } else if ch == '|' {
            count += 1;
        }
    }
    count
}

fn is_markdown_table_separator_cell(cell: &str) -> bool {
    let compact = cell.replace(' ', "");
    let hyphens = compact.chars().filter(|ch| *ch == '-').count();
    hyphens >= 3
        && compact.chars().all(|ch| matches!(ch, '-' | ':'))
        && compact
            .chars()
            .enumerate()
            .all(|(index, ch)| ch != ':' || index == 0 || index + 1 == compact.len())
}
