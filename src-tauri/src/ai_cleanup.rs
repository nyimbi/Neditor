use crate::compiler_support::fenced_code_marker;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct AiCleanupRequest {
    pub(crate) text: String,
    pub(crate) add_provenance: bool,
    pub(crate) mark_as_draft: bool,
    pub(crate) insert_citation_todos: bool,
    #[serde(default)]
    pub(crate) preserve_headings: bool,
    #[serde(default = "default_true")]
    pub(crate) convert_numbered_lists: bool,
    #[serde(default = "default_true")]
    pub(crate) convert_tables: bool,
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
    cleaned = normalize_ai_code_fences(&cleaned, &mut issues);
    if !request.preserve_headings {
        cleaned = remove_duplicate_markdown_headings(&cleaned, &mut issues);
    }
    cleaned = normalize_markdown_lists(&cleaned, &mut issues, request.convert_numbered_lists);
    if request.convert_tables {
        cleaned = normalize_markdown_tables(&cleaned, &mut issues);
    }
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

fn default_true() -> bool {
    true
}

fn remove_duplicate_markdown_headings(text: &str, issues: &mut Vec<String>) -> String {
    let mut in_code_fence = false;
    let mut last_heading = None::<String>;
    let mut only_blank_since_heading = false;
    let mut removed = 0usize;
    let mut output = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code_fence = !in_code_fence;
            last_heading = None;
            only_blank_since_heading = false;
            output.push(line.to_string());
            continue;
        }
        if in_code_fence {
            output.push(line.to_string());
            continue;
        }
        if let Some(heading) = normalized_markdown_heading(trimmed) {
            if only_blank_since_heading && last_heading.as_deref() == Some(heading.as_str()) {
                removed += 1;
                continue;
            }
            last_heading = Some(heading);
            only_blank_since_heading = true;
            output.push(line.to_string());
        } else if trimmed.trim().is_empty() {
            output.push(line.to_string());
        } else {
            last_heading = None;
            only_blank_since_heading = false;
            output.push(line.to_string());
        }
    }

    if removed > 0 {
        issues.push(format!("Removed {removed} duplicated heading marker(s)."));
    }
    output.join("\n")
}

fn normalized_markdown_heading(line: &str) -> Option<String> {
    let marker_len = line.chars().take_while(|ch| *ch == '#').count();
    if !(1..=6).contains(&marker_len) || !line[marker_len..].starts_with(' ') {
        return None;
    }
    let heading = line[marker_len..].trim().trim_end_matches('#').trim();
    if heading.is_empty() {
        return None;
    }
    Some(
        heading
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_ascii_lowercase(),
    )
}

fn remove_chat_labels(text: &str, issues: &mut Vec<String>) -> String {
    let mut removed = Vec::new();
    let mut open_marker: Option<String> = None;
    let output = text
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            if let Some(ref marker) = open_marker.clone() {
                // Inside a fence: close when we see a run of the same character
                // with length >= the opening marker length.
                let fence_char = marker.chars().next().unwrap();
                let run_len = trimmed.chars().take_while(|&c| c == fence_char).count();
                if run_len >= marker.len() {
                    open_marker = None;
                }
                return Some(line.to_string());
            }
            if let Some(marker) = fenced_code_marker(line) {
                open_marker = Some(marker);
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
    input = convert_html_pre_code_blocks(&input, issues);
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

fn convert_html_pre_code_blocks(text: &str, issues: &mut Vec<String>) -> String {
    let mut output = String::new();
    let mut remainder = text;
    let mut converted = 0usize;

    while let Some(pre_start) = find_ascii_case_insensitive(remainder, "<pre") {
        output.push_str(&remainder[..pre_start]);
        let pre_remainder = &remainder[pre_start..];
        let Some(pre_tag_end) = pre_remainder.find('>') else {
            output.push_str(pre_remainder);
            remainder = "";
            break;
        };
        let body_start = pre_start + pre_tag_end + 1;
        let body_remainder = &remainder[body_start..];
        let Some(pre_close) = find_ascii_case_insensitive(body_remainder, "</pre>") else {
            output.push_str(pre_remainder);
            remainder = "";
            break;
        };
        let body = &body_remainder[..pre_close];
        let (language, code_body) = html_pre_code_body(body);
        let plain = strip_html_tags_to_text(code_body)
            .trim_matches('\n')
            .to_string();

        if !output.is_empty() && !output.ends_with("\n\n") {
            if output.ends_with('\n') {
                output.push('\n');
            } else {
                output.push_str("\n\n");
            }
        }
        output.push_str("```");
        output.push_str(&language);
        output.push('\n');
        output.push_str(&plain);
        output.push_str("\n```\n\n");

        converted += 1;
        remainder = &body_remainder[pre_close + "</pre>".len()..];
    }

    output.push_str(remainder);
    if converted > 0 {
        issues.push(format!(
            "Converted {converted} rich HTML code block(s) to Markdown fences."
        ));
    }
    output
}

fn html_pre_code_body(body: &str) -> (String, &str) {
    let Some(code_start) = find_ascii_case_insensitive(body, "<code") else {
        return (String::new(), body);
    };
    let code_remainder = &body[code_start..];
    let Some(code_tag_end) = code_remainder.find('>') else {
        return (String::new(), body);
    };
    let code_tag = &code_remainder[1..code_tag_end];
    let code_body_start = code_start + code_tag_end + 1;
    let code_body_remainder = &body[code_body_start..];
    let Some(code_close) = find_ascii_case_insensitive(code_body_remainder, "</code>") else {
        return (language_from_code_tag(code_tag), code_body_remainder);
    };
    (
        language_from_code_tag(code_tag),
        &code_body_remainder[..code_close],
    )
}

fn language_from_code_tag(tag: &str) -> String {
    for key in ["class", "data-language", "lang"] {
        if let Some(value) = html_attribute(tag, key) {
            if let Some(language) = value.split_whitespace().find_map(normalize_fence_language) {
                return language;
            }
        }
    }
    String::new()
}

fn strip_html_tags_to_text(text: &str) -> String {
    let mut output = String::new();
    let mut index = 0usize;
    while index < text.len() {
        let remainder = &text[index..];
        let Some(tag_start) = remainder.find('<') else {
            output.push_str(remainder);
            break;
        };
        output.push_str(&remainder[..tag_start]);
        let tag_absolute_start = index + tag_start;
        let tag_remainder = &text[tag_absolute_start..];
        let Some(tag_end) = tag_remainder.find('>') else {
            output.push_str(tag_remainder);
            break;
        };
        let raw_tag = tag_remainder[1..tag_end].trim().to_ascii_lowercase();
        if raw_tag.starts_with("br") {
            output.push('\n');
        }
        index = tag_absolute_start + tag_end + 1;
    }
    output
}

fn find_ascii_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    haystack
        .to_ascii_lowercase()
        .find(&needle.to_ascii_lowercase())
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

fn normalize_markdown_lists(
    text: &str,
    issues: &mut Vec<String>,
    convert_numbered_lists: bool,
) -> String {
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
                if convert_numbered_lists {
                    if let Some((number, rest)) = strip_ai_number_prefix(trimmed) {
                        changed = true;
                        return format!("{indent}{number}. {rest}");
                    }
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

fn normalize_ai_code_fences(text: &str, issues: &mut Vec<String>) -> String {
    let mut output = Vec::new();
    let mut in_code_fence = false;
    let mut normalized = 0usize;
    let mut removed_copy_labels = 0usize;

    for line in text.lines() {
        let trimmed = line.trim();
        if !in_code_fence && is_ai_copy_code_label(trimmed) {
            removed_copy_labels += 1;
            continue;
        }
        if let Some((marker, raw_language)) = ai_code_fence_parts(trimmed) {
            if in_code_fence {
                if trimmed != "```" {
                    normalized += 1;
                }
                output.push("```".to_string());
                in_code_fence = false;
                continue;
            }
            let language = raw_language
                .and_then(normalize_fence_language)
                .unwrap_or_default();
            let normalized_line = if language.is_empty() {
                "```".to_string()
            } else {
                format!("```{language}")
            };
            if marker != "```" || trimmed != normalized_line || line != trimmed {
                normalized += 1;
            }
            output.push(normalized_line);
            in_code_fence = true;
            continue;
        }
        output.push(line.to_string());
    }

    if normalized > 0 {
        issues.push(format!("Normalized {normalized} AI code fence marker(s)."));
    }
    if removed_copy_labels > 0 {
        issues.push(format!(
            "Removed {removed_copy_labels} AI code copy label(s)."
        ));
    }
    output.join("\n")
}

fn ai_code_fence_parts(line: &str) -> Option<(&'static str, Option<&str>)> {
    if let Some(rest) = line.strip_prefix("```") {
        return Some(("```", Some(rest.trim()).filter(|value| !value.is_empty())));
    }
    if let Some(rest) = line.strip_prefix("~~~") {
        return Some(("~~~", Some(rest.trim()).filter(|value| !value.is_empty())));
    }
    None
}

fn normalize_fence_language(raw: &str) -> Option<String> {
    let mut value = raw
        .trim()
        .trim_matches('`')
        .trim()
        .trim_start_matches("{.")
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    if value.to_ascii_lowercase().starts_with("language-") {
        value = &value["language-".len()..];
    }
    let token = value.split_whitespace().next().unwrap_or("").trim();
    if token.is_empty()
        || token.eq_ignore_ascii_case("copy")
        || token.eq_ignore_ascii_case("code")
        || token.eq_ignore_ascii_case("copy-code")
    {
        return None;
    }
    Some(token.to_ascii_lowercase())
}

fn is_ai_copy_code_label(line: &str) -> bool {
    matches!(
        line.to_ascii_lowercase().as_str(),
        "copy code" | "copy" | "code"
    )
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
    let mut tab_blocks = 0usize;
    let mut csv_blocks = 0usize;
    let mut in_code_fence = false;
    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code_fence = !in_code_fence;
            output.push(line.to_string());
            index += 1;
            continue;
        }
        if !in_code_fence {
            if let Some((rows, consumed)) = delimited_table_block(&lines[index..], '\t') {
                output.extend(markdown_table_rows(&rows));
                tab_blocks += 1;
                index += consumed;
                continue;
            }
            if let Some((rows, consumed)) = delimited_table_block(&lines[index..], ',') {
                output.extend(markdown_table_rows(&rows));
                csv_blocks += 1;
                index += consumed;
                continue;
            }
        }
        output.push(line.to_string());
        index += 1;
    }
    if tab_blocks > 0 {
        issues.push(format!(
            "Converted {tab_blocks} tab-separated table block(s) to Markdown."
        ));
    }
    if csv_blocks > 0 {
        issues.push(format!(
            "Converted {csv_blocks} comma-separated table block(s) to Markdown."
        ));
    }
    output.join("\n")
}

fn delimited_table_block(lines: &[&str], delimiter: char) -> Option<(Vec<Vec<String>>, usize)> {
    let first = lines.first()?;
    let first_cells = trimmed_delimited_cells(first, delimiter)?;
    let width = first_cells.len();
    if width < 2 {
        return None;
    }
    let mut rows = vec![first_cells];
    let mut consumed = 1usize;
    for line in lines.iter().skip(1) {
        if line.trim().is_empty() || line.trim_start().starts_with("```") {
            break;
        }
        let Some(cells) = trimmed_delimited_cells(line, delimiter) else {
            break;
        };
        if cells.len() != width {
            break;
        }
        rows.push(cells);
        consumed += 1;
    }
    if delimiter == ',' && !looks_like_csv_table_block(&rows) {
        return None;
    }
    Some((rows, consumed))
}

fn trimmed_delimited_cells(line: &str, delimiter: char) -> Option<Vec<String>> {
    if !line.contains(delimiter) {
        return None;
    }
    let cells: Vec<String> = if delimiter == '\t' {
        line.split('\t')
            .map(str::trim)
            .map(str::to_string)
            .collect()
    } else {
        parse_csv_line(line)?
            .into_iter()
            .map(|cell| cell.trim().to_string())
            .collect()
    };
    let start = cells.iter().position(|cell| !cell.is_empty())?;
    let end = cells
        .iter()
        .rposition(|cell| !cell.is_empty())
        .unwrap_or(start);
    Some(cells[start..=end].to_vec())
}

fn parse_csv_line(line: &str) -> Option<Vec<String>> {
    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '"' if quoted && chars.peek() == Some(&'"') => {
                cell.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => {
                cells.push(cell.trim().to_string());
                cell.clear();
            }
            _ => cell.push(ch),
        }
    }
    if quoted {
        return None;
    }
    cells.push(cell.trim().to_string());
    Some(cells)
}

fn looks_like_csv_table_block(rows: &[Vec<String>]) -> bool {
    if rows.len() < 2 {
        return false;
    }
    let header_like = rows[0]
        .iter()
        .all(|cell| !cell.is_empty() && cell.len() <= 48 && !cell.ends_with('.'));
    let data_like = rows.iter().skip(1).any(|row| {
        row.iter()
            .any(|cell| parse_tableish_number(cell).is_some() || looks_like_short_date(cell))
    });
    header_like && data_like
}

fn parse_tableish_number(value: &str) -> Option<f64> {
    value
        .trim()
        .trim_end_matches('%')
        .replace(['$', ','], "")
        .parse::<f64>()
        .ok()
}

fn looks_like_short_date(value: &str) -> bool {
    let normalized = value.trim();
    normalized.len() >= 8
        && normalized.len() <= 12
        && normalized.chars().any(|ch| ch.is_ascii_digit())
        && (normalized.contains('-') || normalized.contains('/'))
}

fn markdown_table_rows(rows: &[Vec<String>]) -> Vec<String> {
    let mut output = Vec::new();
    output.push(markdown_table_row(&rows[0]));
    if rows.len() > 1 {
        output.push(format!(
            "| {} |",
            rows[0]
                .iter()
                .map(|_| "---")
                .collect::<Vec<_>>()
                .join(" | ")
        ));
        output.extend(rows.iter().skip(1).map(|row| markdown_table_row(row)));
    }
    output
}

fn markdown_table_row(cells: &[String]) -> String {
    format!(
        "| {} |",
        cells
            .iter()
            .map(|cell| cell.replace('|', "\\|"))
            .collect::<Vec<_>>()
            .join(" | ")
    )
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
