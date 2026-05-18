use crate::escape_html;
use std::collections::{BTreeMap, HashMap};

pub(crate) fn render_footnotes(markdown: &str) -> String {
    let mut body_lines = Vec::new();
    let mut definitions = BTreeMap::new();
    let lines = markdown.lines().collect::<Vec<_>>();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        if let Some((key, text)) = parse_footnote_definition(line) {
            let mut parts = vec![text];
            index += 1;
            while index < lines.len() {
                if let Some(continuation) = footnote_continuation_line(lines[index]) {
                    parts.push(continuation);
                    index += 1;
                } else {
                    break;
                }
            }
            definitions.insert(key, parts.join(" "));
        } else {
            body_lines.push(line.to_string());
            index += 1;
        }
    }
    if definitions.is_empty() {
        return markdown.to_string();
    }

    let mut order = Vec::new();
    let mut numbers = HashMap::new();
    let body = body_lines
        .into_iter()
        .map(|line| replace_footnote_references(&line, &definitions, &mut order, &mut numbers))
        .collect::<Vec<_>>()
        .join("\n");
    if order.is_empty() {
        return body;
    }

    let entries = order
        .iter()
        .filter_map(|key| definitions.get(key).map(|text| (key, text)))
        .enumerate()
        .map(|(index, (key, text))| {
            format!(
                "<li value=\"{}\" id=\"fn:{}\">{} <a href=\"#fnref:{}\" aria-label=\"Back to footnote reference\">back</a></li>",
                index + 1,
                escape_html(key),
                escape_html(text),
                escape_html(key)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("{body}\n\n<section class=\"footnotes\" role=\"doc-endnotes\"><ol>\n{entries}\n</ol></section>")
}

fn parse_footnote_definition(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("[^")?;
    let (key, text) = rest.split_once("]:")?;
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    Some((key.to_string(), text.trim().to_string()))
}

fn footnote_continuation_line(line: &str) -> Option<String> {
    line.strip_prefix("    ")
        .or_else(|| line.strip_prefix('\t'))
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
}

fn replace_footnote_references(
    line: &str,
    definitions: &BTreeMap<String, String>,
    order: &mut Vec<String>,
    numbers: &mut HashMap<String, usize>,
) -> String {
    let mut output = String::new();
    let mut rest = line;
    while let Some(start) = rest.find("[^") {
        output.push_str(&rest[..start]);
        let after_start = &rest[start + 2..];
        let Some(end) = after_start.find(']') else {
            output.push_str(&rest[start..]);
            return output;
        };
        let key = &after_start[..end];
        if definitions.contains_key(key) {
            let number = *numbers.entry(key.to_string()).or_insert_with(|| {
                order.push(key.to_string());
                order.len()
            });
            output.push_str(&format!(
                "<sup id=\"fnref:{}\"><a href=\"#fn:{}\">{number}</a></sup>",
                escape_html(key),
                escape_html(key)
            ));
            rest = &after_start[end + 1..];
        } else {
            output.push_str(&rest[..start + 2 + end + 1]);
            rest = &after_start[end + 1..];
        }
    }
    output.push_str(rest);
    output
}
