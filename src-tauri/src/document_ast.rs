use crate::{
    markdown_tables::{
        is_markdown_table_row, is_markdown_table_separator, split_markdown_table_row,
    },
    review::{parse_change_note, parse_review_comment},
    transforms::TransformArtifact,
};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub(crate) struct DocumentAst {
    pub(crate) blocks: Vec<DocumentBlock>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct AstSourceRange {
    pub(crate) source_file: String,
    pub(crate) source_line: usize,
    pub(crate) end_source_line: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct FootnoteEntry {
    pub(crate) number: usize,
    pub(crate) key: String,
    pub(crate) text: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct TaskListItem {
    pub(crate) checked: bool,
    pub(crate) text: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AstReviewComment {
    pub(crate) author: String,
    pub(crate) created_at: String,
    pub(crate) state: String,
    pub(crate) text: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AstChangeNote {
    pub(crate) author: String,
    pub(crate) created_at: String,
    pub(crate) text: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AstAiSource {
    pub(crate) provider: String,
    pub(crate) model: String,
    pub(crate) date: String,
    pub(crate) prompt_summary: String,
    pub(crate) reviewed_by: String,
    pub(crate) reviewed_at: String,
    pub(crate) status: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum InlineNode {
    Text {
        text: String,
    },
    Strong {
        text: String,
    },
    Emphasis {
        text: String,
    },
    Code {
        text: String,
    },
    Link {
        text: String,
        url: String,
    },
    Citation {
        key: String,
        keys: Vec<String>,
        raw: String,
    },
    CrossReference {
        key: String,
        raw: String,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum DocumentBlock {
    Heading {
        level: usize,
        text: String,
        anchor: String,
        line: usize,
        end_line: usize,
        source: Option<AstSourceRange>,
    },
    Paragraph {
        text: String,
        inlines: Vec<InlineNode>,
        line: usize,
        end_line: usize,
        source: Option<AstSourceRange>,
    },
    List {
        ordered: bool,
        items: Vec<String>,
        line: usize,
        end_line: usize,
        source: Option<AstSourceRange>,
    },
    TaskList {
        items: Vec<TaskListItem>,
        line: usize,
        end_line: usize,
        source: Option<AstSourceRange>,
    },
    BlockQuote {
        text: String,
        line: usize,
        end_line: usize,
        source: Option<AstSourceRange>,
    },
    CodeBlock {
        language: Option<String>,
        code: String,
        line: usize,
        end_line: usize,
        source: Option<AstSourceRange>,
    },
    Table {
        line: usize,
        end_line: usize,
        id: Option<String>,
        caption: Option<String>,
        headers: Vec<String>,
        alignments: Vec<String>,
        rows: Vec<Vec<String>>,
        source: Option<AstSourceRange>,
    },
    Figure {
        line: usize,
        end_line: usize,
        id: Option<String>,
        src: Option<String>,
        alt: Option<String>,
        caption: Option<String>,
        source: Option<AstSourceRange>,
    },
    Equation {
        line: usize,
        end_line: usize,
        id: Option<String>,
        caption: Option<String>,
        text: String,
        source: Option<AstSourceRange>,
    },
    Layout {
        line: usize,
        end_line: usize,
        directive: String,
        options: String,
        source: Option<AstSourceRange>,
    },
    Callout {
        line: usize,
        end_line: usize,
        callout_type: String,
        title: String,
        text: String,
        source: Option<AstSourceRange>,
    },
    Footnotes {
        line: usize,
        end_line: usize,
        entries: Vec<FootnoteEntry>,
        source: Option<AstSourceRange>,
    },
    ReviewComment {
        line: usize,
        end_line: usize,
        comment: AstReviewComment,
        source: Option<AstSourceRange>,
    },
    ChangeNote {
        line: usize,
        end_line: usize,
        note: AstChangeNote,
        source: Option<AstSourceRange>,
    },
    AiSource {
        line: usize,
        end_line: usize,
        provenance: AstAiSource,
        source: Option<AstSourceRange>,
    },
    Transform {
        line: usize,
        end_line: usize,
        name: String,
        output_kind: String,
        text: String,
        html: String,
        source_hash: Option<String>,
        output_hash: Option<String>,
        cache_key: Option<String>,
        execution_kind: Option<String>,
        options: Option<Value>,
        source: Option<AstSourceRange>,
    },
    RawHtml {
        line: usize,
        end_line: usize,
        html: String,
        source: Option<AstSourceRange>,
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

        if is_ast_table_start(&lines, index) {
            let caption = pending_table_caption(&mut paragraph_lines, &mut paragraph_start);
            if let Some((table, next_index)) = parse_ast_table(&lines, index, caption) {
                flush_ast_paragraph(&mut blocks, &mut paragraph_lines, &mut paragraph_start);
                blocks.push(table);
                index = next_index;
                continue;
            }
        }

        if let Some((list, next_index)) = parse_ast_list(&lines, index) {
            flush_ast_paragraph(&mut blocks, &mut paragraph_lines, &mut paragraph_start);
            blocks.push(list);
            index = next_index;
            continue;
        }

        if let Some((code_block, next_index)) = parse_ast_code_block(&lines, index) {
            flush_ast_paragraph(&mut blocks, &mut paragraph_lines, &mut paragraph_start);
            blocks.push(code_block);
            index = next_index;
            continue;
        }

        if let Some((quote, next_index)) = parse_ast_block_quote(&lines, index) {
            flush_ast_paragraph(&mut blocks, &mut paragraph_lines, &mut paragraph_start);
            blocks.push(quote);
            index = next_index;
            continue;
        }

        if is_footnotes_start(trimmed) {
            flush_ast_paragraph(&mut blocks, &mut paragraph_lines, &mut paragraph_start);
            let (footnotes, next_index) = parse_ast_footnotes(&lines, index);
            blocks.push(footnotes);
            index = next_index;
            continue;
        }

        if trimmed.starts_with('<') {
            flush_ast_paragraph(&mut blocks, &mut paragraph_lines, &mut paragraph_start);
            let (html, next_index) = collect_ast_html_block(&lines, index);
            blocks.push(parse_ast_html_block(&html, line_number, next_index));
            index = next_index;
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

fn collect_ast_html_block(lines: &[&str], start_index: usize) -> (String, usize) {
    let first = lines[start_index].trim();
    let Some(close_marker) = html_block_close_marker(first) else {
        return (first.to_string(), start_index + 1);
    };
    let mut html = first.to_string();
    let mut index = start_index + 1;
    while !html.contains(&close_marker) && index < lines.len() {
        let next = lines[index].trim();
        if next.is_empty() {
            break;
        }
        html.push('\n');
        html.push_str(next);
        index += 1;
    }
    (html, index)
}

fn html_block_close_marker(line: &str) -> Option<String> {
    if line.starts_with("<!--") {
        return Some("-->".to_string());
    }
    if line.ends_with("/>") {
        return None;
    }
    let tag = line
        .trim_start_matches('<')
        .trim_start_matches('/')
        .split(|ch: char| ch == '>' || ch.is_whitespace())
        .next()
        .unwrap_or("");
    if matches!(
        tag.to_ascii_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    ) {
        return None;
    }
    (!tag.is_empty()).then(|| format!("</{tag}>"))
}

pub(crate) fn export_body_text_from_ast(ast: &DocumentAst) -> String {
    ast.blocks
        .iter()
        .filter_map(|block| match block {
            DocumentBlock::Heading { level, text, .. } => {
                Some(format!("{} {text}", "#".repeat(*level)))
            }
            DocumentBlock::Paragraph { text, .. } => Some(text.clone()),
            DocumentBlock::List { ordered, items, .. } => Some(
                items
                    .iter()
                    .enumerate()
                    .map(|(index, item)| {
                        if *ordered {
                            format!("{}. {item}", index + 1)
                        } else {
                            format!("- {item}")
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            DocumentBlock::TaskList { items, .. } => Some(
                items
                    .iter()
                    .map(|item| {
                        format!("- [{}] {}", if item.checked { "x" } else { " " }, item.text)
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            DocumentBlock::BlockQuote { text, .. } => Some(
                text.lines()
                    .map(|line| format!("> {line}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            DocumentBlock::CodeBlock { language, code, .. } => Some(format!(
                "```{}\n{}\n```",
                language.as_deref().unwrap_or(""),
                code.trim_end()
            )),
            DocumentBlock::Table {
                id,
                caption,
                headers,
                alignments: _,
                rows,
                ..
            } => {
                let mut lines = Vec::new();
                lines.push(table_export_title(id, caption, headers));
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
            DocumentBlock::Callout {
                callout_type,
                title,
                text,
                ..
            } => {
                let mut parts = vec![format!("Callout: {callout_type}")];
                if !title.is_empty() {
                    parts.push(title.clone());
                }
                if !text.is_empty() {
                    parts.push(text.clone());
                }
                Some(parts.join(": "))
            }
            DocumentBlock::Footnotes { entries, .. } => {
                let mut lines = vec!["Footnotes".to_string()];
                lines.extend(
                    entries
                        .iter()
                        .map(|entry| format!("{}. {}", entry.number, entry.text)),
                );
                Some(lines.join("\n"))
            }
            DocumentBlock::ReviewComment { comment, .. } => Some(format!(
                "Review comment: {} | {} | {}",
                comment.state, comment.author, comment.text
            )),
            DocumentBlock::ChangeNote { note, .. } => {
                Some(format!("Change note: {} | {}", note.author, note.text))
            }
            DocumentBlock::AiSource { provenance, .. } => Some(format!(
                "AI source: {} / {} | {}",
                empty_as(&provenance.provider, "unknown"),
                empty_as(&provenance.model, "unknown"),
                empty_as(&provenance.status, "unreviewed")
            )),
            DocumentBlock::Transform { name, text, .. } => Some(transform_export_text(name, text)),
            DocumentBlock::RawHtml { html, .. } => {
                let text = clean_inline_text(html);
                (!text.is_empty()).then_some(text)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn attach_source_ranges<F>(ast: &mut DocumentAst, mut resolve: F)
where
    F: FnMut(usize, usize) -> Option<AstSourceRange>,
{
    for block in &mut ast.blocks {
        match block {
            DocumentBlock::Heading {
                line,
                end_line,
                source,
                ..
            }
            | DocumentBlock::Paragraph {
                line,
                end_line,
                source,
                ..
            }
            | DocumentBlock::List {
                line,
                end_line,
                source,
                ..
            }
            | DocumentBlock::TaskList {
                line,
                end_line,
                source,
                ..
            }
            | DocumentBlock::BlockQuote {
                line,
                end_line,
                source,
                ..
            }
            | DocumentBlock::CodeBlock {
                line,
                end_line,
                source,
                ..
            }
            | DocumentBlock::Table {
                line,
                end_line,
                source,
                ..
            }
            | DocumentBlock::Figure {
                line,
                end_line,
                source,
                ..
            }
            | DocumentBlock::Equation {
                line,
                end_line,
                source,
                ..
            }
            | DocumentBlock::Layout {
                line,
                end_line,
                source,
                ..
            }
            | DocumentBlock::Callout {
                line,
                end_line,
                source,
                ..
            }
            | DocumentBlock::Footnotes {
                line,
                end_line,
                source,
                ..
            }
            | DocumentBlock::ReviewComment {
                line,
                end_line,
                source,
                ..
            }
            | DocumentBlock::ChangeNote {
                line,
                end_line,
                source,
                ..
            }
            | DocumentBlock::AiSource {
                line,
                end_line,
                source,
                ..
            }
            | DocumentBlock::Transform {
                line,
                end_line,
                source,
                ..
            }
            | DocumentBlock::RawHtml {
                line,
                end_line,
                source,
                ..
            } => {
                *source = resolve(*line, *end_line);
            }
        }
    }
}

pub(crate) fn attach_transform_artifacts(ast: &mut DocumentAst, artifacts: &[TransformArtifact]) {
    let mut search_start = 0usize;
    for block in &mut ast.blocks {
        let DocumentBlock::Transform {
            name,
            output_kind,
            source_hash,
            output_hash,
            cache_key,
            execution_kind,
            options,
            ..
        } = block
        else {
            continue;
        };
        let Some((artifact_index, artifact)) = artifacts
            .iter()
            .enumerate()
            .skip(search_start)
            .find(|(_, artifact)| artifact.name == *name)
        else {
            continue;
        };
        *output_kind = artifact.output_kind.clone();
        *source_hash = Some(artifact.source_hash.clone());
        *output_hash = Some(artifact.output_hash.clone());
        *cache_key = Some(artifact.cache_key.clone());
        *execution_kind = Some(artifact.execution_kind.clone());
        *options = Some(artifact.options.clone());
        search_start = artifact_index + 1;
    }
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
        let inlines = parse_inline_nodes(&paragraph_lines.join(" "));
        blocks.push(DocumentBlock::Paragraph {
            text,
            inlines,
            line: paragraph_start.unwrap_or(1),
            end_line: paragraph_start.unwrap_or(1) + paragraph_lines.len().saturating_sub(1),
            source: None,
        });
    }
    paragraph_lines.clear();
    *paragraph_start = None;
}

fn parse_inline_nodes(text: &str) -> Vec<InlineNode> {
    let mut nodes = Vec::new();
    let mut index = 0usize;
    while index < text.len() {
        let rest = &text[index..];
        let Some((start, end, node)) = earliest_inline_node(rest) else {
            push_inline_text(&mut nodes, rest);
            break;
        };
        push_inline_text(&mut nodes, &rest[..start]);
        push_inline_node(&mut nodes, node);
        index += end;
    }
    nodes
}

fn earliest_inline_node(text: &str) -> Option<(usize, usize, InlineNode)> {
    let mut candidates = Vec::new();
    if let Some(strong) = parse_wrapped_inline(text, "**", "**") {
        candidates.push((
            strong.start,
            strong.end,
            InlineNode::Strong { text: strong.text },
        ));
    }
    if let Some(code) = parse_wrapped_inline(text, "`", "`") {
        candidates.push((code.start, code.end, InlineNode::Code { text: code.text }));
    }
    if let Some(link) = parse_markdown_link_inline(text) {
        candidates.push((
            link.start,
            link.end,
            InlineNode::Link {
                text: link.text,
                url: link.url,
            },
        ));
    }
    if let Some(citation) = parse_html_citation_inline(text) {
        candidates.push((
            citation.start,
            citation.end,
            InlineNode::Citation {
                key: citation.key,
                keys: citation.keys,
                raw: citation.raw,
            },
        ));
    }
    if let Some(citation) = parse_citation_inline(text) {
        candidates.push((
            citation.start,
            citation.end,
            InlineNode::Citation {
                key: citation.key,
                keys: citation.keys,
                raw: citation.raw,
            },
        ));
    }
    if let Some(reference) = parse_cross_reference_inline(text) {
        candidates.push((
            reference.start,
            reference.end,
            InlineNode::CrossReference {
                key: reference.key,
                raw: reference.raw,
            },
        ));
    }
    if let Some(emphasis) = parse_wrapped_inline(text, "*", "*") {
        candidates.push((
            emphasis.start,
            emphasis.end,
            InlineNode::Emphasis {
                text: emphasis.text,
            },
        ));
    }
    candidates.into_iter().min_by_key(|(start, _, _)| *start)
}

struct WrappedInline {
    start: usize,
    end: usize,
    text: String,
}

struct CitationInline {
    start: usize,
    end: usize,
    key: String,
    keys: Vec<String>,
    raw: String,
}

struct LinkInline {
    start: usize,
    end: usize,
    text: String,
    url: String,
}

fn parse_wrapped_inline(text: &str, open: &str, close: &str) -> Option<WrappedInline> {
    let start = text.find(open)?;
    if open == "*" && text[start..].starts_with("**") {
        return None;
    }
    let content_start = start + open.len();
    let content_end = text[content_start..].find(close)? + content_start;
    if content_end == content_start {
        return None;
    }
    Some(WrappedInline {
        start,
        end: content_end + close.len(),
        text: clean_inline_text(&text[content_start..content_end]),
    })
}

fn parse_citation_inline(text: &str) -> Option<CitationInline> {
    let start = text.find("[@")?;
    let end = text[start..].find(']')? + start + 1;
    let raw = text[start..end].to_string();
    let keys = citation_keys_from_inline(raw.trim_start_matches('[').trim_end_matches(']'));
    let key = keys.first().cloned().unwrap_or_default();
    (!key.is_empty()).then_some(CitationInline {
        start,
        end,
        key,
        keys,
        raw,
    })
}

fn parse_html_citation_inline(text: &str) -> Option<CitationInline> {
    let marker = "data-citation-keys=\"";
    let marker_start = text.find(marker)?;
    let start = text[..marker_start].rfind("<span").unwrap_or(marker_start);
    let key_start = marker_start + marker.len();
    let key_end = text[key_start..].find('"')? + key_start;
    let keys = text[key_start..key_end]
        .split_whitespace()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let key = keys.first().cloned().unwrap_or_default();
    let close_start = text[key_end..].find("</span>")? + key_end;
    let end = close_start + "</span>".len();
    (!key.is_empty()).then_some(CitationInline {
        start,
        end,
        key,
        keys,
        raw: text[start..end].to_string(),
    })
}

fn citation_keys_from_inline(content: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut rest = content;
    while let Some(marker_start) = rest.find('@') {
        let after_marker = &rest[marker_start + 1..];
        let key = after_marker
            .trim_start_matches(['-', '+'])
            .chars()
            .take_while(|ch| !matches!(ch, ',' | ';' | ']' | ')' | '(') && !ch.is_whitespace())
            .collect::<String>();
        if !key.is_empty() {
            keys.push(key);
        }
        rest = after_marker;
    }
    keys
}

fn parse_cross_reference_inline(text: &str) -> Option<CitationInline> {
    let start = text.find("{@")?;
    let end = text[start..].find('}')? + start + 1;
    let raw = text[start..end].to_string();
    let key = raw
        .trim_start_matches("{@")
        .trim_end_matches('}')
        .trim()
        .to_string();
    (!key.is_empty()).then_some(CitationInline {
        start,
        end,
        keys: vec![key.clone()],
        key,
        raw,
    })
}

fn parse_markdown_link_inline(text: &str) -> Option<LinkInline> {
    let start = text.find('[')?;
    if text[start..].starts_with("[@") {
        return None;
    }
    let label_end = text[start..].find(']')? + start;
    let url_start = label_end + 1;
    if !text[url_start..].starts_with('(') {
        return None;
    }
    let url_end = text[url_start + 1..].find(')')? + url_start + 1;
    let label = clean_inline_text(&text[start + 1..label_end]);
    let url = decode_html_entities(&text[url_start + 1..url_end]);
    (!label.is_empty() && !url.trim().is_empty()).then_some(LinkInline {
        start,
        end: url_end + 1,
        text: label,
        url,
    })
}

fn push_inline_text(nodes: &mut Vec<InlineNode>, text: &str) {
    let text = clean_inline_text(text);
    if !text.is_empty() {
        push_inline_node(nodes, InlineNode::Text { text });
    }
}

fn push_inline_node(nodes: &mut Vec<InlineNode>, node: InlineNode) {
    nodes.push(node);
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
        end_line: line_number,
        source: None,
    })
}

fn parse_ast_table(
    lines: &[&str],
    index: usize,
    caption: Option<TableCaption>,
) -> Option<(DocumentBlock, usize)> {
    if index + 1 >= lines.len() {
        return None;
    }
    let header = lines[index].trim();
    let separator = lines[index + 1].trim();
    if !is_markdown_table_row(header) || !is_markdown_table_separator(separator) {
        return None;
    }

    let headers = split_markdown_table_row(header)
        .iter()
        .map(|cell| clean_inline_text(cell))
        .collect::<Vec<_>>();
    let alignments = split_markdown_table_row(separator)
        .iter()
        .map(|cell| table_alignment(cell))
        .collect::<Vec<_>>();
    let mut rows = Vec::new();
    let mut next_index = index + 2;
    while next_index < lines.len() && is_markdown_table_row(lines[next_index].trim()) {
        let row = split_markdown_table_row(lines[next_index].trim())
            .iter()
            .map(|cell| clean_inline_text(cell))
            .collect::<Vec<_>>();
        rows.push(row);
        next_index += 1;
    }

    Some((
        DocumentBlock::Table {
            line: index + 1,
            end_line: next_index,
            id: caption.as_ref().and_then(|caption| caption.id.clone()),
            caption: caption.and_then(|caption| caption.caption),
            headers,
            alignments,
            rows,
            source: None,
        },
        next_index,
    ))
}

fn parse_ast_list(lines: &[&str], index: usize) -> Option<(DocumentBlock, usize)> {
    if let Some(task_list) = parse_ast_task_list(lines, index) {
        return Some(task_list);
    }
    let (ordered, first_item) = parse_ast_list_item(lines.get(index)?.trim())?;
    let mut items = vec![first_item];
    let mut next_index = index + 1;
    while next_index < lines.len() {
        let Some((candidate_ordered, item)) = parse_ast_list_item(lines[next_index].trim()) else {
            break;
        };
        if candidate_ordered != ordered {
            break;
        }
        items.push(item);
        next_index += 1;
    }
    Some((
        DocumentBlock::List {
            ordered,
            items,
            line: index + 1,
            end_line: next_index,
            source: None,
        },
        next_index,
    ))
}

fn parse_ast_task_list(lines: &[&str], index: usize) -> Option<(DocumentBlock, usize)> {
    let first_item = parse_ast_task_list_item(lines.get(index)?.trim())?;
    let mut items = vec![first_item];
    let mut next_index = index + 1;
    while next_index < lines.len() {
        let Some(item) = parse_ast_task_list_item(lines[next_index].trim()) else {
            break;
        };
        items.push(item);
        next_index += 1;
    }
    Some((
        DocumentBlock::TaskList {
            items,
            line: index + 1,
            end_line: next_index,
            source: None,
        },
        next_index,
    ))
}

fn parse_ast_task_list_item(line: &str) -> Option<TaskListItem> {
    let item = line
        .strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))
        .or_else(|| line.strip_prefix("+ "))?;
    let marker = item.get(..3)?;
    let checked = match marker {
        "[ ]" => false,
        "[x]" | "[X]" => true,
        _ => return None,
    };
    let text = clean_inline_text(item.get(3..)?.trim_start());
    (!text.is_empty()).then_some(TaskListItem { checked, text })
}

fn parse_ast_list_item(line: &str) -> Option<(bool, String)> {
    if let Some(item) = line
        .strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))
        .or_else(|| line.strip_prefix("+ "))
    {
        let item = clean_inline_text(item);
        return (!item.is_empty()).then_some((false, item));
    }

    let marker_end = line
        .char_indices()
        .take_while(|(_, ch)| ch.is_ascii_digit())
        .last()
        .map(|(index, ch)| index + ch.len_utf8())?;
    if marker_end == 0 || !matches!(line.as_bytes().get(marker_end), Some(b'.' | b')')) {
        return None;
    }
    if line.as_bytes().get(marker_end + 1).copied() != Some(b' ') {
        return None;
    }
    let item = clean_inline_text(&line[marker_end + 2..]);
    (!item.is_empty()).then_some((true, item))
}

fn parse_ast_code_block(lines: &[&str], index: usize) -> Option<(DocumentBlock, usize)> {
    let opener = lines.get(index)?.trim();
    let marker = if opener.starts_with("```") {
        "```"
    } else if opener.starts_with("~~~") {
        "~~~"
    } else {
        return None;
    };
    let language = opener[marker.len()..]
        .split_whitespace()
        .next()
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    let mut code_lines = Vec::new();
    let mut next_index = index + 1;
    while next_index < lines.len() {
        let line = lines[next_index];
        if line.trim_start().starts_with(marker) {
            if language.as_deref() == Some("ai-source") {
                let code = code_lines.join("\n");
                return Some((
                    DocumentBlock::AiSource {
                        provenance: parse_ast_ai_source(&code),
                        line: index + 1,
                        end_line: next_index + 1,
                        source: None,
                    },
                    next_index + 1,
                ));
            }
            return Some((
                DocumentBlock::CodeBlock {
                    language,
                    code: code_lines.join("\n"),
                    line: index + 1,
                    end_line: next_index + 1,
                    source: None,
                },
                next_index + 1,
            ));
        }
        code_lines.push(line.to_string());
        next_index += 1;
    }
    if language.as_deref() == Some("ai-source") {
        let code = code_lines.join("\n");
        return Some((
            DocumentBlock::AiSource {
                provenance: parse_ast_ai_source(&code),
                line: index + 1,
                end_line: next_index,
                source: None,
            },
            next_index,
        ));
    }
    Some((
        DocumentBlock::CodeBlock {
            language,
            code: code_lines.join("\n"),
            line: index + 1,
            end_line: next_index,
            source: None,
        },
        next_index,
    ))
}

fn parse_ast_block_quote(lines: &[&str], index: usize) -> Option<(DocumentBlock, usize)> {
    if !lines.get(index)?.trim_start().starts_with('>') {
        return None;
    }
    let mut quote_lines = Vec::new();
    let mut next_index = index;
    while next_index < lines.len() {
        let line = lines[next_index].trim_start();
        if !line.starts_with('>') {
            break;
        }
        quote_lines.push(clean_inline_text(line.trim_start_matches('>').trim_start()));
        next_index += 1;
    }
    let text = quote_lines.join("\n").trim().to_string();
    if text.is_empty() {
        return None;
    }
    Some((
        DocumentBlock::BlockQuote {
            text,
            line: index + 1,
            end_line: next_index,
            source: None,
        },
        next_index,
    ))
}

fn is_ast_table_start(lines: &[&str], index: usize) -> bool {
    index + 1 < lines.len()
        && is_markdown_table_row(lines[index].trim())
        && is_markdown_table_separator(lines[index + 1].trim())
}

struct TableCaption {
    id: Option<String>,
    caption: Option<String>,
}

fn pending_table_caption(
    paragraph_lines: &mut Vec<String>,
    paragraph_start: &mut Option<usize>,
) -> Option<TableCaption> {
    let caption = parse_table_caption(paragraph_lines.last()?)?;
    paragraph_lines.pop();
    if paragraph_lines.is_empty() {
        *paragraph_start = None;
    }
    Some(caption)
}

fn parse_table_caption(line: &str) -> Option<TableCaption> {
    let trimmed = line.trim();
    if !trimmed.to_ascii_lowercase().starts_with("table:") {
        return None;
    }
    let id = extract_table_caption_id(trimmed);
    let caption = extract_table_caption_text(trimmed);
    if id.is_none() && caption.is_none() {
        return None;
    }
    Some(TableCaption { id, caption })
}

fn extract_table_caption_id(line: &str) -> Option<String> {
    let (_, after) = line.split_once("{#")?;
    let id = after
        .split(['}', ' ', '\t'])
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    (!id.is_empty()).then_some(id)
}

fn extract_table_caption_text(line: &str) -> Option<String> {
    if let Some(caption) = extract_quoted_attribute(line, "caption") {
        let caption = clean_inline_text(&caption);
        return (!caption.is_empty()).then_some(caption);
    }
    let without_prefix = line.trim_start_matches(|ch: char| ch != ':');
    let without_prefix = without_prefix.trim_start_matches(':').trim();
    let before_attrs = without_prefix.split("{#").next().unwrap_or("").trim();
    (!before_attrs.is_empty()).then(|| clean_inline_text(before_attrs))
}

fn table_export_title(id: &Option<String>, caption: &Option<String>, headers: &[String]) -> String {
    let mut parts = vec!["Table".to_string()];
    if let Some(id) = id {
        parts.push(id.clone());
    }
    if let Some(caption) = caption {
        parts.push(caption.clone());
    }
    if parts.len() == 1 {
        parts.push(headers.join(" | "));
    }
    parts.join(": ")
}

fn transform_export_text(name: &str, text: &str) -> String {
    let label = format!("Transform: {name}");
    if text.is_empty() {
        label
    } else {
        format!("{label}: {text}")
    }
}

fn parse_ast_html_block(line: &str, line_number: usize, end_line: usize) -> DocumentBlock {
    if let Some(table) = parse_ast_transform_table(line, line_number, end_line) {
        return table;
    }

    if let Some(transform) = parse_ast_transform_block(line, line_number, end_line) {
        return transform;
    }

    if let Some(content) = line
        .trim()
        .strip_prefix("<!-- comment:")
        .and_then(|value| value.strip_suffix("-->"))
    {
        return DocumentBlock::ReviewComment {
            line: line_number,
            end_line,
            comment: {
                let parsed = parse_review_comment(line_number, content);
                AstReviewComment {
                    author: parsed.author,
                    created_at: parsed.created_at,
                    state: parsed.state,
                    text: parsed.text,
                }
            },
            source: None,
        };
    }

    if let Some(content) = line
        .trim()
        .strip_prefix("<!-- change:")
        .and_then(|value| value.strip_suffix("-->"))
    {
        return DocumentBlock::ChangeNote {
            line: line_number,
            end_line,
            note: {
                let parsed = parse_change_note(line_number, content);
                AstChangeNote {
                    author: parsed.author,
                    created_at: parsed.created_at,
                    text: parsed.text,
                }
            },
            source: None,
        };
    }

    if line.contains("class=\"figure\"") {
        return DocumentBlock::Figure {
            line: line_number,
            end_line,
            id: extract_quoted_attribute(line, "id"),
            src: extract_quoted_attribute(line, "src"),
            alt: extract_quoted_attribute(line, "alt").map(|value| decode_html_entities(&value)),
            caption: extract_between(line, "<figcaption>", "</figcaption>")
                .map(|value| clean_inline_text(&value)),
            source: None,
        };
    }

    if line.contains("class=\"equation\"") {
        return DocumentBlock::Equation {
            line: line_number,
            end_line,
            id: extract_quoted_attribute(line, "id"),
            caption: extract_between(line, "<figcaption>", "</figcaption>")
                .map(|value| clean_inline_text(&value)),
            text: extract_between(line, "<code>", "</code>")
                .map(|value| clean_inline_text(&value))
                .unwrap_or_default(),
            source: None,
        };
    }

    if line.contains("data-layout=\"") {
        return DocumentBlock::Layout {
            line: line_number,
            end_line,
            directive: extract_quoted_attribute(line, "data-layout").unwrap_or_default(),
            options: extract_quoted_attribute(line, "data-options")
                .map(|value| decode_html_entities(&value))
                .unwrap_or_default(),
            source: None,
        };
    }

    if line.contains("class=\"callout") {
        return DocumentBlock::Callout {
            line: line_number,
            end_line,
            callout_type: extract_quoted_attribute(line, "data-callout").unwrap_or_default(),
            title: extract_between(line, "<strong>", "</strong>")
                .map(|value| clean_inline_text(&value))
                .unwrap_or_default(),
            text: extract_between(line, "<p>", "</p>")
                .map(|value| clean_inline_text(&value))
                .unwrap_or_default(),
            source: None,
        };
    }

    DocumentBlock::RawHtml {
        line: line_number,
        end_line,
        html: line.to_string(),
        source: None,
    }
}

fn parse_ast_transform_block(
    html: &str,
    line_number: usize,
    end_line: usize,
) -> Option<DocumentBlock> {
    let class_attr = extract_quoted_attribute(html, "class")?;
    if !class_attr
        .split_whitespace()
        .any(|class| class == "transform")
    {
        return None;
    }
    let name = class_attr
        .split_whitespace()
        .filter_map(|class| class.strip_prefix("transform-"))
        .find(|name| !matches!(*name, "error" | "table"))
        .unwrap_or("unknown")
        .to_string();
    let output_kind = if html.contains("<svg") { "svg" } else { "html" }.to_string();
    Some(DocumentBlock::Transform {
        line: line_number,
        end_line,
        name,
        output_kind,
        text: clean_inline_text(html),
        html: html.to_string(),
        source_hash: None,
        output_hash: None,
        cache_key: None,
        execution_kind: None,
        options: None,
        source: None,
    })
}

fn parse_ast_transform_table(
    html: &str,
    line_number: usize,
    end_line: usize,
) -> Option<DocumentBlock> {
    if !html.contains("<table") || !html.contains("transform-table") {
        return None;
    }
    let header_section = html_between(html, "<thead", "</thead>")?;
    let headers = html_table_cells(header_section, "th");
    if headers.is_empty() {
        return None;
    }
    let body_section = html_between(html, "<tbody", "</tbody>").unwrap_or("");
    let mut rows = Vec::new();
    let mut rest = body_section;
    while let Some((row_html, next)) = next_html_tag_block(rest, "tr") {
        let row = html_table_cells(row_html, "td");
        if !row.is_empty() {
            rows.push(
                (0..headers.len())
                    .map(|index| row.get(index).cloned().unwrap_or_default())
                    .collect(),
            );
        }
        rest = next;
    }
    Some(DocumentBlock::Table {
        line: line_number,
        end_line,
        id: None,
        caption: None,
        headers: headers.clone(),
        alignments: headers.iter().map(|_| "left".to_string()).collect(),
        rows,
        source: None,
    })
}

fn html_between<'a>(html: &'a str, open_prefix: &str, close_tag: &str) -> Option<&'a str> {
    let open_start = html.find(open_prefix)?;
    let open_end = html[open_start..].find('>')? + open_start + 1;
    let close_start = html[open_end..].find(close_tag)? + open_end;
    Some(&html[open_end..close_start])
}

fn next_html_tag_block<'a>(html: &'a str, tag: &str) -> Option<(&'a str, &'a str)> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let open_start = html.find(&open)?;
    let open_end = html[open_start..].find('>')? + open_start + 1;
    let close_start = html[open_end..].find(&close)? + open_end;
    let close_end = close_start + close.len();
    Some((&html[open_end..close_start], &html[close_end..]))
}

fn html_table_cells(row_html: &str, tag: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut rest = row_html;
    while let Some((cell_html, next)) = next_html_tag_block(rest, tag) {
        let text = clean_inline_text(cell_html).trim().to_string();
        cells.push(text);
        rest = next;
    }
    cells
}

fn parse_ast_ai_source(content: &str) -> AstAiSource {
    let mut provenance = AstAiSource {
        provider: String::new(),
        model: String::new(),
        date: String::new(),
        prompt_summary: String::new(),
        reviewed_by: String::new(),
        reviewed_at: String::new(),
        status: "needs-review".to_string(),
    };

    for line in content.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim().to_string();
        match key.trim() {
            "provider" => provenance.provider = value,
            "model" => provenance.model = value,
            "date" => provenance.date = value,
            "promptSummary" | "prompt_summary" | "prompt" => provenance.prompt_summary = value,
            "reviewedBy" | "reviewed_by" | "reviewer" => provenance.reviewed_by = value,
            "reviewedAt" | "reviewed_at" | "reviewDate" => provenance.reviewed_at = value,
            "status" => provenance.status = value,
            _ => {}
        }
    }

    provenance
}

fn is_footnotes_start(line: &str) -> bool {
    line.contains("class=\"footnotes\"") || line.contains("role=\"doc-endnotes\"")
}

fn parse_ast_footnotes(lines: &[&str], start_index: usize) -> (DocumentBlock, usize) {
    let mut entries = Vec::new();
    let mut index = start_index;
    while index < lines.len() {
        let line = lines[index].trim();
        if line.starts_with("<li ") || line.starts_with("<li>") {
            entries.push(parse_footnote_entry(line, entries.len() + 1));
        }
        index += 1;
        if line.contains("</section>") {
            break;
        }
    }
    (
        DocumentBlock::Footnotes {
            line: start_index + 1,
            end_line: index,
            entries,
            source: None,
        },
        index,
    )
}

fn parse_footnote_entry(line: &str, fallback_number: usize) -> FootnoteEntry {
    let number = extract_quoted_attribute(line, "value")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(fallback_number);
    let key = extract_quoted_attribute(line, "id")
        .and_then(|value| value.strip_prefix("fn:").map(ToString::to_string))
        .unwrap_or_else(|| number.to_string());
    let text = clean_inline_text(line)
        .trim_end_matches(" back")
        .trim()
        .to_string();
    FootnoteEntry { number, key, text }
}

fn table_alignment(cell: &str) -> String {
    let compact = cell.replace(' ', "");
    if compact.starts_with(':') && compact.ends_with(':') {
        "center".to_string()
    } else if compact.ends_with(':') {
        "right".to_string()
    } else {
        "left".to_string()
    }
}

pub(crate) fn extract_label(text: &str) -> Option<String> {
    text.split("{#")
        .nth(1)
        .and_then(|rest| rest.split_once('}'))
        .map(|(label, _)| label.split_whitespace().next().unwrap_or("").to_string())
        .filter(|label| !label.is_empty())
}

pub(crate) fn extract_quoted_attribute(text: &str, key: &str) -> Option<String> {
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
    decode_html_entities(&without_tags.replace("**", "").replace(['*', '`'], ""))
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

fn empty_as<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() {
        fallback
    } else {
        value
    }
}

fn extract_between(text: &str, start: &str, end: &str) -> Option<String> {
    let after_start = text.split(start).nth(1)?;
    let (value, _) = after_start.split_once(end)?;
    Some(value.to_string())
}

pub(crate) fn slugify(text: &str) -> String {
    text.to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
