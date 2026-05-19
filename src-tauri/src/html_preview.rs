use crate::escape_html;
use pulldown_cmark::{html, Options, Parser};
use std::collections::BTreeMap;

pub(crate) fn markdown_to_html(
    markdown: &str,
    heading_anchors: &[&str],
    glossary: &BTreeMap<String, String>,
) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    let html_with_heading_ids = add_heading_ids(&html_output, heading_anchors);
    annotate_glossary_terms(&html_with_heading_ids, glossary)
}

fn add_heading_ids(html: &str, heading_anchors: &[&str]) -> String {
    if heading_anchors.is_empty() {
        return html.to_string();
    }
    let mut output = String::with_capacity(html.len());
    let mut rest = html;
    let mut heading_index = 0usize;
    while let Some(start) = rest.find("<h") {
        output.push_str(&rest[..start]);
        let candidate = &rest[start..];
        let Some(level) = candidate
            .as_bytes()
            .get(2)
            .and_then(|byte| char::from(*byte).to_digit(10))
            .map(|digit| digit as usize)
        else {
            output.push_str("<h");
            rest = &candidate[2..];
            continue;
        };
        if !(1..=6).contains(&level) {
            output.push_str("<h");
            rest = &candidate[2..];
            continue;
        }
        let Some(tag_end) = candidate.find('>') else {
            output.push_str(candidate);
            return output;
        };
        let tag = &candidate[..=tag_end];
        if tag.contains(" id=") {
            output.push_str(tag);
        } else if let Some(anchor) = heading_anchors.get(heading_index) {
            output.push_str(&tag[..tag.len() - 1]);
            output.push_str(&format!(" id=\"{}\">", escape_html(anchor)));
        } else {
            output.push_str(tag);
        }
        heading_index += 1;
        rest = &candidate[tag_end + 1..];
    }
    output.push_str(rest);
    output
}

fn annotate_glossary_terms(html: &str, glossary: &BTreeMap<String, String>) -> String {
    if glossary.is_empty() {
        return html.to_string();
    }
    let terms = glossary
        .iter()
        .filter(|(term, _)| !term.trim().is_empty())
        .map(|(term, definition)| (term.as_str(), definition.as_str()))
        .collect::<Vec<_>>();
    if terms.is_empty() {
        return html.to_string();
    }

    let mut output = String::with_capacity(html.len());
    let mut text_segment = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        if ch == '<' {
            if !text_segment.is_empty() {
                output.push_str(&annotate_glossary_text_segment(&text_segment, &terms));
                text_segment.clear();
            }
            in_tag = true;
            output.push(ch);
        } else if ch == '>' {
            in_tag = false;
            output.push(ch);
        } else if in_tag {
            output.push(ch);
        } else {
            text_segment.push(ch);
        }
    }
    if !text_segment.is_empty() {
        output.push_str(&annotate_glossary_text_segment(&text_segment, &terms));
    }
    output
}

fn annotate_glossary_text_segment(segment: &str, terms: &[(&str, &str)]) -> String {
    let mut output = String::with_capacity(segment.len());
    let mut index = 0;
    while index < segment.len() {
        if let Some((term, definition)) = terms
            .iter()
            .filter(|(term, _)| segment[index..].starts_with(*term))
            .filter(|(term, _)| glossary_term_has_boundaries(segment, index, index + term.len()))
            .max_by_key(|(term, _)| term.len())
        {
            let matched = &segment[index..index + term.len()];
            output.push_str(&format!(
                "<span class=\"glossary-term\" tabindex=\"0\" title=\"{}\" data-definition=\"{}\">{}</span>",
                escape_html(definition),
                escape_html(definition),
                matched
            ));
            index += term.len();
        } else if let Some(ch) = segment[index..].chars().next() {
            output.push(ch);
            index += ch.len_utf8();
        } else {
            break;
        }
    }
    output
}

fn glossary_term_has_boundaries(segment: &str, start: usize, end: usize) -> bool {
    let before = segment[..start].chars().next_back();
    let after = segment[end..].chars().next();
    !before.is_some_and(is_word_char) && !after.is_some_and(is_word_char)
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}
