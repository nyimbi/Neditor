pub(crate) fn is_markdown_table_row(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    let cells = split_markdown_table_row(trimmed);
    cells.len() >= 2
        || (trimmed.starts_with('|')
            && has_unescaped_trailing_pipe(trimmed)
            && unescaped_pipe_count(trimmed) >= 2
            && !cells.is_empty())
}

pub(crate) fn is_markdown_table_separator(line: &str) -> bool {
    is_markdown_table_row(line)
        && split_markdown_table_row(line)
            .iter()
            .all(|cell| is_markdown_table_separator_cell(cell))
}

pub(crate) fn split_markdown_table_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let inner = strip_optional_outer_pipes(trimmed);
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

fn strip_optional_outer_pipes(line: &str) -> &str {
    let inner = line.strip_prefix('|').unwrap_or(line);
    if has_unescaped_trailing_pipe(inner) {
        &inner[..inner.len() - 1]
    } else {
        inner
    }
}

fn has_unescaped_trailing_pipe(line: &str) -> bool {
    if !line.ends_with('|') {
        return false;
    }
    let mut slash_count = 0;
    for ch in line[..line.len() - 1].chars().rev() {
        if ch == '\\' {
            slash_count += 1;
        } else {
            break;
        }
    }
    slash_count % 2 == 0
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

pub(crate) fn markdown_table_separator_row(cells: &[String]) -> String {
    markdown_table_row(
        &cells
            .iter()
            .map(|cell| {
                let compact = cell.replace(char::is_whitespace, "");
                match (compact.starts_with(':'), compact.ends_with(':')) {
                    (true, true) => ":---:".to_string(),
                    (false, true) => "---:".to_string(),
                    _ => "---".to_string(),
                }
            })
            .collect::<Vec<_>>(),
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
