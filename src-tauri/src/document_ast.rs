use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct DocumentAst {
    pub(crate) blocks: Vec<DocumentBlock>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum DocumentBlock {
    Heading {
        level: usize,
        text: String,
        anchor: String,
        line: usize,
    },
    Paragraph {
        text: String,
        line: usize,
    },
    Table {
        line: usize,
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Figure {
        line: usize,
        id: Option<String>,
        src: Option<String>,
        alt: Option<String>,
        caption: Option<String>,
    },
    Equation {
        line: usize,
        id: Option<String>,
        caption: Option<String>,
        text: String,
    },
    Layout {
        line: usize,
        directive: String,
        options: String,
    },
    RawHtml {
        line: usize,
        html: String,
    },
}

pub(crate) fn build_document_ast(markdown: &str) -> DocumentAst {
    let lines = markdown.lines().collect::<Vec<_>>();
    let mut blocks = Vec::new();
    let mut paragraph_lines = Vec::new();
    let mut paragraph_start = None;
    let mut index = 0usize;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        let line_number = index + 1;

        if trimmed.is_empty() {
            flush_ast_paragraph(&mut blocks, &mut paragraph_lines, &mut paragraph_start);
            index += 1;
            continue;
        }

        if let Some(heading) = parse_ast_heading(trimmed, line_number) {
            flush_ast_paragraph(&mut blocks, &mut paragraph_lines, &mut paragraph_start);
            blocks.push(heading);
            index += 1;
            continue;
        }

        if let Some((table, next_index)) = parse_ast_table(&lines, index) {
            flush_ast_paragraph(&mut blocks, &mut paragraph_lines, &mut paragraph_start);
            blocks.push(table);
            index = next_index;
            continue;
        }

        if trimmed.starts_with('<') && trimmed.ends_with('>') {
            flush_ast_paragraph(&mut blocks, &mut paragraph_lines, &mut paragraph_start);
            blocks.push(parse_ast_html_block(trimmed, line_number));
            index += 1;
            continue;
        }

        if paragraph_start.is_none() {
            paragraph_start = Some(line_number);
        }
        paragraph_lines.push(trimmed.to_string());
        index += 1;
    }

    flush_ast_paragraph(&mut blocks, &mut paragraph_lines, &mut paragraph_start);
    DocumentAst { blocks }
}

pub(crate) fn export_body_text_from_ast(ast: &DocumentAst) -> String {
    ast.blocks
        .iter()
        .filter_map(|block| match block {
            DocumentBlock::Heading { level, text, .. } => {
                Some(format!("{} {text}", "#".repeat(*level)))
            }
            DocumentBlock::Paragraph { text, .. } => Some(text.clone()),
            DocumentBlock::Table { headers, rows, .. } => {
                let mut lines = Vec::new();
                lines.push(format!("Table: {}", headers.join(" | ")));
                for row in rows {
                    lines.push(format!("- {}", row.join(" | ")));
                }
                Some(lines.join("\n"))
            }
            DocumentBlock::Figure {
                id, src, caption, ..
            } => {
                let mut parts = vec!["Figure".to_string()];
                if let Some(id) = id {
                    parts.push(id.clone());
                }
                if let Some(caption) = caption {
                    parts.push(caption.clone());
                }
                if let Some(src) = src {
                    parts.push(format!("({src})"));
                }
                Some(parts.join(": "))
            }
            DocumentBlock::Equation {
                id, caption, text, ..
            } => {
                let mut parts = vec!["Equation".to_string()];
                if let Some(id) = id {
                    parts.push(id.clone());
                }
                if !text.is_empty() {
                    parts.push(text.clone());
                }
                if let Some(caption) = caption {
                    parts.push(caption.clone());
                }
                Some(parts.join(": "))
            }
            DocumentBlock::Layout {
                directive, options, ..
            } => Some(format!("Layout: {directive} {options}").trim().to_string()),
            DocumentBlock::RawHtml { html, .. } => {
                let text = clean_inline_text(html);
                (!text.is_empty()).then_some(text)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn flush_ast_paragraph(
    blocks: &mut Vec<DocumentBlock>,
    paragraph_lines: &mut Vec<String>,
    paragraph_start: &mut Option<usize>,
) {
    if paragraph_lines.is_empty() {
        return;
    }
    let text = clean_inline_text(&paragraph_lines.join(" "));
    if !text.is_empty() {
        blocks.push(DocumentBlock::Paragraph {
            text,
            line: paragraph_start.unwrap_or(1),
        });
    }
    paragraph_lines.clear();
    *paragraph_start = None;
}

fn parse_ast_heading(line: &str, line_number: usize) -> Option<DocumentBlock> {
    let level = line.chars().take_while(|ch| *ch == '#').count();
    if level == 0 || level > 6 || line.as_bytes().get(level).copied() != Some(b' ') {
        return None;
    }
    let raw_text = line[level..].trim();
    let text = clean_inline_text(strip_markdown_attributes(raw_text));
    if text.is_empty() {
        return None;
    }
    let anchor = extract_label(raw_text).unwrap_or_else(|| slugify(&text));
    Some(DocumentBlock::Heading {
        level,
        text,
        anchor,
        line: line_number,
    })
}

fn parse_ast_table(lines: &[&str], index: usize) -> Option<(DocumentBlock, usize)> {
    if index + 1 >= lines.len() {
        return None;
    }
    let header = lines[index].trim();
    let separator = lines[index + 1].trim();
    if !is_markdown_table_row(header) || !is_markdown_table_separator(separator) {
        return None;
    }

    let headers = split_table_row(header)
        .iter()
        .map(|cell| clean_inline_text(cell))
        .collect::<Vec<_>>();
    let mut rows = Vec::new();
    let mut next_index = index + 2;
    while next_index < lines.len() && is_markdown_table_row(lines[next_index].trim()) {
        let row = split_table_row(lines[next_index].trim())
            .iter()
            .map(|cell| clean_inline_text(cell))
            .collect::<Vec<_>>();
        rows.push(row);
        next_index += 1;
    }

    Some((
        DocumentBlock::Table {
            line: index + 1,
            headers,
            rows,
        },
        next_index,
    ))
}

fn parse_ast_html_block(line: &str, line_number: usize) -> DocumentBlock {
    if line.contains("class=\"figure\"") {
        return DocumentBlock::Figure {
            line: line_number,
            id: extract_quoted_attribute(line, "id"),
            src: extract_quoted_attribute(line, "src"),
            alt: extract_quoted_attribute(line, "alt").map(|value| decode_html_entities(&value)),
            caption: extract_between(line, "<figcaption>", "</figcaption>")
                .map(|value| clean_inline_text(&value)),
        };
    }

    if line.contains("class=\"equation\"") {
        return DocumentBlock::Equation {
            line: line_number,
            id: extract_quoted_attribute(line, "id"),
            caption: extract_between(line, "<figcaption>", "</figcaption>")
                .map(|value| clean_inline_text(&value)),
            text: extract_between(line, "<code>", "</code>")
                .map(|value| clean_inline_text(&value))
                .unwrap_or_default(),
        };
    }

    if line.contains("data-layout=\"") {
        return DocumentBlock::Layout {
            line: line_number,
            directive: extract_quoted_attribute(line, "data-layout").unwrap_or_default(),
            options: extract_quoted_attribute(line, "data-options")
                .map(|value| decode_html_entities(&value))
                .unwrap_or_default(),
        };
    }

    DocumentBlock::RawHtml {
        line: line_number,
        html: line.to_string(),
    }
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

fn extract_label(text: &str) -> Option<String> {
    text.split("{#")
        .nth(1)
        .and_then(|rest| rest.split_once('}'))
        .map(|(label, _)| label.split_whitespace().next().unwrap_or("").to_string())
        .filter(|label| !label.is_empty())
}

fn extract_quoted_attribute(text: &str, key: &str) -> Option<String> {
    let marker = format!("{key}=\"");
    let after_marker = text.split(&marker).nth(1)?;
    let (value, _) = after_marker.split_once('"')?;
    Some(value.to_string())
}

fn strip_markdown_attributes(text: &str) -> &str {
    if let Some(index) = text.rfind("{#") {
        return text[..index].trim_end();
    }
    text
}

fn clean_inline_text(text: &str) -> String {
    let without_attrs = strip_markdown_attributes(text);
    let without_tags = strip_html_tags(without_attrs);
    decode_html_entities(
        &without_tags
            .replace("**", "")
            .replace('*', "")
            .replace('`', ""),
    )
    .trim()
    .to_string()
}

fn strip_html_tags(text: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                output.push(' ');
            }
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    output.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn decode_html_entities(text: &str) -> String {
    text.replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn extract_between(text: &str, start: &str, end: &str) -> Option<String> {
    let after_start = text.split(start).nth(1)?;
    let (value, _) = after_start.split_once(end)?;
    Some(value.to_string())
}

fn slugify(text: &str) -> String {
    text.to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
