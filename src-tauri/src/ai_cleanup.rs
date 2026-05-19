use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct AiCleanupRequest {
    pub(crate) text: String,
    pub(crate) add_provenance: bool,
    pub(crate) mark_as_draft: bool,
    pub(crate) insert_citation_todos: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct AiCleanupResponse {
    pub(crate) cleaned_markdown: String,
    pub(crate) issues: Vec<String>,
    pub(crate) provenance_block: Option<String>,
}

#[tauri::command]
pub(crate) fn cleanup_ai_paste(request: AiCleanupRequest) -> AiCleanupResponse {
    let mut issues = Vec::new();
    let mut cleaned = request.text.replace("\r\n", "\n");
    cleaned = remove_chat_labels(&cleaned, &mut issues);
    cleaned = normalize_rich_html_clipboard(&cleaned, &mut issues);
    cleaned = normalize_markdown_lists(&cleaned, &mut issues);
    cleaned = normalize_markdown_tables(&cleaned, &mut issues);
    if request.insert_citation_todos {
        cleaned = insert_citation_todos(&cleaned, &mut issues);
    }
    if request.mark_as_draft && !cleaned.contains("ai-assisted:") {
        cleaned = format!(
            "<!-- ai-assisted: status=needs-review | reviewedBy= | reviewedAt= | source=AI paste cleanup | promptSummary=AI paste cleanup review required -->\n\n{cleaned}"
        );
        issues.push("Marked inserted content as draft.".to_string());
    }
    let provenance_block = if request.add_provenance {
        Some(format!(
            "```ai-source\nprovider: unknown\nmodel: unknown\ndate: {}\npromptSummary: AI paste cleanup\nreviewedBy: \nstatus: needs-review\n```",
            Utc::now().date_naive()
        ))
    } else {
        None
    };
    if let Some(block) = &provenance_block {
        cleaned = format!("{cleaned}\n\n{block}\n");
    }
    AiCleanupResponse {
        cleaned_markdown: cleaned.trim().to_string(),
        issues,
        provenance_block,
    }
}

fn remove_chat_labels(text: &str, issues: &mut Vec<String>) -> String {
    let mut removed = Vec::new();
    let mut in_code_fence = false;
    let output = text
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("```") {
                in_code_fence = !in_code_fence;
                return Some(line.to_string());
            }
            if in_code_fence {
                return Some(line.to_string());
            }
            if let Some((label, rest)) = strip_chat_label(trimmed) {
                removed.push(label.to_string());
                if rest.trim().is_empty() {
                    None
                } else {
                    let indent = &line[..line.len() - trimmed.len()];
                    Some(format!("{indent}{}", rest.trim_start()))
                }
            } else {
                Some(line.to_string())
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if !removed.is_empty() {
        removed.sort();
        removed.dedup();
        issues.push(format!("Removed chat labels: {}.", removed.join(", ")));
    }
    output
}

fn strip_chat_label(line: &str) -> Option<(&'static str, &str)> {
    const LABELS: &[&str] = &[
        "ChatGPT said:",
        "Claude said:",
        "DeepSeek said:",
        "Gemini said:",
        "Google Gemini said:",
        "Copilot said:",
        "Microsoft Copilot said:",
        "Perplexity said:",
        "Assistant:",
        "AI:",
        "User:",
        "You:",
        "Human:",
    ];
    let lower = line.to_ascii_lowercase();
    LABELS.iter().find_map(|label| {
        let normalized_label = label.to_ascii_lowercase();
        lower
            .starts_with(&normalized_label)
            .then(|| (*label, &line[label.len()..]))
    })
}

fn normalize_rich_html_clipboard(text: &str, issues: &mut Vec<String>) -> String {
    if !looks_like_rich_html_clipboard(text) {
        return text.to_string();
    }

    let mut output = String::new();
    let mut input = text.replace("\r\n", "\n").replace('\r', "\n");
    input = strip_html_comments(&input);
    let mut index = 0usize;
    let mut link_stack = Vec::new();

    while index < input.len() {
        let remainder = &input[index..];
        let Some(tag_start) = remainder.find('<') else {
            output.push_str(remainder);
            break;
        };
        output.push_str(&remainder[..tag_start]);
        let tag_absolute_start = index + tag_start;
        let tag_remainder = &input[tag_absolute_start..];
        let Some(tag_end) = tag_remainder.find('>') else {
            output.push_str(tag_remainder);
            break;
        };
        let raw_tag = &tag_remainder[1..tag_end];
        push_markdown_for_html_tag(raw_tag, &mut output, &mut link_stack);
        index = tag_absolute_start + tag_end + 1;
    }

    issues.push("Converted rich HTML clipboard content to Markdown-like text.".to_string());
    normalize_blank_lines(&decode_html_entities(&output))
}

fn looks_like_rich_html_clipboard(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    [
        "<p", "</p", "<div", "</div", "<br", "<h1", "<h2", "<h3", "<ul", "<ol", "<li", "<table",
        "<tr", "<td", "<th", "<a ",
    ]
    .iter()
    .any(|tag| lower.contains(tag))
}

fn strip_html_comments(text: &str) -> String {
    let mut output = String::new();
    let mut remainder = text;
    while let Some(start) = remainder.find("<!--") {
        output.push_str(&remainder[..start]);
        let after_start = &remainder[start + 4..];
        let Some(end) = after_start.find("-->") else {
            return output;
        };
        remainder = &after_start[end + 3..];
    }
    output.push_str(remainder);
    output
}

fn push_markdown_for_html_tag(raw_tag: &str, output: &mut String, link_stack: &mut Vec<String>) {
    let tag = raw_tag.trim().trim_end_matches('/').trim();
    if tag.is_empty() || tag.starts_with('!') {
        return;
    }
    let closing = tag.starts_with('/');
    let name = tag
        .trim_start_matches('/')
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();

    match (closing, name.as_str()) {
        (false, "br") => output.push('\n'),
        (false, "a") => {
            if let Some(href) = html_attribute(tag, "href").filter(|value| !value.trim().is_empty())
            {
                output.push('[');
                link_stack.push(href);
            }
        }
        (true, "a") => {
            if let Some(href) = link_stack.pop() {
                output.push_str("](");
                output.push_str(&href);
                output.push(')');
            }
        }
        (false, "h1") => start_block_with(output, "# "),
        (false, "h2") => start_block_with(output, "## "),
        (false, "h3") => start_block_with(output, "### "),
        (false, "h4") => start_block_with(output, "#### "),
        (false, "h5") => start_block_with(output, "##### "),
        (false, "h6") => start_block_with(output, "###### "),
        (true, "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "div" | "section" | "article") => {
            output.push_str("\n\n")
        }
        (false, "li") => start_block_with(output, "- "),
        (true, "li") => output.push('\n'),
        (true, "tr") => output.push('\n'),
        (true, "td" | "th") => output.push('\t'),
        (true, "table" | "thead" | "tbody") => output.push('\n'),
        _ => {}
    }
}

fn html_attribute(tag: &str, key: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let marker = format!("{key}=");
    let marker_start = lower.find(&marker)?;
    let after_marker = &tag[marker_start + marker.len()..];
    let mut chars = after_marker.chars();
    let first = chars.next()?;
    if first == '"' || first == '\'' {
        let value_start = first.len_utf8();
        let value_end = after_marker[value_start..].find(first)? + value_start;
        return Some(after_marker[value_start..value_end].to_string());
    }
    Some(
        after_marker
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_end_matches('/')
            .to_string(),
    )
}

fn start_block_with(output: &mut String, prefix: &str) {
    let trimmed = output.trim_end_matches([' ', '\t']);
    let newline_count = trimmed.chars().rev().take_while(|ch| *ch == '\n').count();
    if !trimmed.is_empty() && newline_count == 0 {
        output.push('\n');
    }
    output.push_str(prefix);
}

fn decode_html_entities(text: &str) -> String {
    let mut output = String::new();
    let mut remainder = text;
    while let Some(start) = remainder.find('&') {
        output.push_str(&remainder[..start]);
        let entity_remainder = &remainder[start..];
        let Some(end) = entity_remainder.find(';') else {
            output.push_str(entity_remainder);
            return output;
        };
        let entity = &entity_remainder[1..end];
        if let Some(decoded) = decode_html_entity(entity) {
            output.push(decoded);
        } else {
            output.push_str(&entity_remainder[..=end]);
        }
        remainder = &entity_remainder[end + 1..];
    }
    output.push_str(remainder);
    output
}

fn decode_html_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" | "#39" => Some('\''),
        "nbsp" => Some(' '),
        _ if entity.starts_with("#x") || entity.starts_with("#X") => {
            u32::from_str_radix(&entity[2..], 16)
                .ok()
                .and_then(char::from_u32)
        }
        _ if entity.starts_with('#') => entity[1..].parse::<u32>().ok().and_then(char::from_u32),
        _ => None,
    }
}

fn normalize_blank_lines(text: &str) -> String {
    let mut output = Vec::new();
    let mut blank_count = 0usize;
    for line in text.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                output.push(String::new());
            }
        } else {
            blank_count = 0;
            output.push(
                line.trim_start_matches('\t')
                    .trim_end_matches('\t')
                    .to_string(),
            );
        }
    }
    output.join("\n").trim().to_string()
}

fn normalize_markdown_lists(text: &str, issues: &mut Vec<String>) -> String {
    let mut changed = false;
    let mut in_code_fence = false;
    let output = text
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("```") {
                in_code_fence = !in_code_fence;
                return line.to_string();
            }
            if !in_code_fence {
                let indent = &line[..line.len() - trimmed.len()];
                if let Some(rest) = strip_ai_bullet_prefix(trimmed) {
                    changed = true;
                    return format!("{indent}- {rest}");
                }
                if let Some((number, rest)) = strip_ai_number_prefix(trimmed) {
                    changed = true;
                    return format!("{indent}{number}. {rest}");
                }
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    if changed {
        issues.push("Normalized bullet characters to Markdown list markers.".to_string());
    }
    output
}

fn strip_ai_bullet_prefix(line: &str) -> Option<&str> {
    ["• ", "◦ ", "‣ ", "▪ ", "▫ ", "– ", "— "]
        .iter()
        .find_map(|prefix| line.strip_prefix(prefix))
}

fn strip_ai_number_prefix(line: &str) -> Option<(&str, &str)> {
    let marker_end = line
        .char_indices()
        .take_while(|(_, ch)| ch.is_ascii_digit())
        .last()
        .map(|(index, ch)| index + ch.len_utf8())?;
    if marker_end == 0 || !line[marker_end..].starts_with(") ") {
        return None;
    }
    Some((&line[..marker_end], line[marker_end + 2..].trim_start()))
}

fn normalize_markdown_tables(text: &str, issues: &mut Vec<String>) -> String {
    let lines = text.lines().collect::<Vec<_>>();
    let mut output = Vec::new();
    let mut index = 0;
    let mut changed = false;
    let mut in_code_fence = false;
    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code_fence = !in_code_fence;
            output.push(line.to_string());
        } else if !in_code_fence && line.contains('\t') {
            let cells = trimmed_tab_cells(line);
            if cells.is_empty() {
                index += 1;
                continue;
            }
            output.push(format!("| {} |", cells.join(" | ")));
            if index + 1 < lines.len() && lines[index + 1].contains('\t') {
                output.push(format!(
                    "| {} |",
                    cells.iter().map(|_| "---").collect::<Vec<_>>().join(" | ")
                ));
            }
            changed = true;
        } else {
            output.push(line.to_string());
        }
        index += 1;
    }
    if changed {
        issues.push("Converted tab-separated rows to Markdown table rows.".to_string());
    }
    output.join("\n")
}

fn trimmed_tab_cells(line: &str) -> Vec<&str> {
    let cells = line.split('\t').map(str::trim).collect::<Vec<_>>();
    let Some(start) = cells.iter().position(|cell| !cell.is_empty()) else {
        return Vec::new();
    };
    let end = cells
        .iter()
        .rposition(|cell| !cell.is_empty())
        .unwrap_or(start);
    cells[start..=end].to_vec()
}

fn insert_citation_todos(text: &str, issues: &mut Vec<String>) -> String {
    let mut in_code_fence = false;
    let mut inserted = 0usize;
    let output = text
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("```") {
                in_code_fence = !in_code_fence;
                return line.to_string();
            }
            if in_code_fence || !line_needs_citation_todo(trimmed) {
                return line.to_string();
            }
            inserted += 1;
            format!("{line} <!-- TODO: citation needed -->")
        })
        .collect::<Vec<_>>()
        .join("\n");
    if inserted > 0 {
        issues.push(format!("Inserted {inserted} citation TODO marker(s)."));
    }
    output
}

fn line_needs_citation_todo(line: &str) -> bool {
    if line.is_empty()
        || line.starts_with('#')
        || line.starts_with('|')
        || line.starts_with("<!--")
        || line.starts_with('>')
        || line.contains("[@")
        || line.contains("http://")
        || line.contains("https://")
        || line.contains("TODO: citation")
    {
        return false;
    }
    let lower = line.to_ascii_lowercase();
    let factual_signal = line.chars().any(|ch| ch.is_ascii_digit())
        || [
            "according to",
            "research",
            "study",
            "report",
            "market",
            "revenue",
            "growth",
            "customers",
            "users",
            "increased",
            "decreased",
        ]
        .iter()
        .any(|signal| lower.contains(signal));
    factual_signal && matches!(line.chars().last(), Some('.' | ')' | '%'))
}
