use crate::diagnostics::DocumentDiagnostic;
use crate::{
    compiler_support::{collect_fence_bodies_with_lines, fenced_code_marker},
    diagnostics::diag,
    escape_html, path_to_string,
};
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize)]
pub(crate) struct BibliographyEntry {
    pub(crate) key: String,
    pub(crate) title: String,
    pub(crate) author: Option<String>,
    pub(crate) issued: Option<String>,
    pub(crate) raw: String,
    pub(crate) source_file: Option<String>,
    pub(crate) line: Option<usize>,
    pub(crate) column: Option<usize>,
    pub(crate) end_column: Option<usize>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CitationReference {
    pub(crate) key: String,
    pub(crate) locator: Option<String>,
    pub(crate) raw: String,
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) end_column: usize,
}

pub(crate) fn collect_bibliography(
    text: &str,
    metadata: &Value,
    root_path: Option<&Path>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> Vec<BibliographyEntry> {
    let mut sources = collect_fence_bodies_with_lines(text, "bibtex")
        .into_iter()
        .map(|(line, body)| BibliographySource {
            body,
            source_file: None,
            start_line: line + 1,
        })
        .collect::<Vec<_>>();
    sources.extend(
        collect_fence_bodies_with_lines(text, "hayagriva")
            .into_iter()
            .map(|(line, body)| BibliographySource {
                body,
                source_file: None,
                start_line: line + 1,
            }),
    );
    sources.extend(
        collect_fence_bodies_with_lines(text, "bibliography")
            .into_iter()
            .map(|(line, body)| BibliographySource {
                body,
                source_file: None,
                start_line: line + 1,
            }),
    );
    for path in bibliography_source_paths(metadata) {
        let base = root_path
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let bibliography_path = base.join(path);
        match fs::read_to_string(&bibliography_path) {
            Ok(contents) => sources.push(BibliographySource {
                body: contents,
                source_file: Some(path_to_string(&bibliography_path)),
                start_line: 1,
            }),
            Err(err) => diagnostics.push(diag(
                "error",
                format!(
                    "Missing bibliography file {}: {err}",
                    bibliography_path.display()
                ),
                Some(path_to_string(&bibliography_path)),
                None,
                Some("Create the bibliography file or update front matter bibliography paths."),
            )),
        }
    }

    sources
        .into_iter()
        .flat_map(|source| {
            parse_bibliography_source_with_origin(
                &source.body,
                source.source_file.as_deref(),
                source.start_line,
            )
        })
        .collect()
}

fn bibliography_source_paths(metadata: &Value) -> Vec<&str> {
    let mut paths = Vec::new();
    for key in [
        "bibliography",
        "bibliographies",
        "citationSources",
        "citation_sources",
    ] {
        collect_bibliography_paths(metadata.get(key), &mut paths);
    }
    paths
}

fn collect_bibliography_paths<'a>(value: Option<&'a Value>, paths: &mut Vec<&'a str>) {
    match value {
        Some(Value::String(path)) => paths.push(path),
        Some(Value::Array(values)) => {
            for value in values {
                collect_bibliography_paths(Some(value), paths);
            }
        }
        Some(Value::Object(object)) => {
            if let Some(path) = object
                .get("path")
                .or_else(|| object.get("file"))
                .and_then(Value::as_str)
            {
                paths.push(path);
            }
        }
        _ => {}
    }
}

struct BibliographySource {
    body: String,
    source_file: Option<String>,
    start_line: usize,
}

pub(crate) fn parse_bibliography_source(body: &str) -> Vec<BibliographyEntry> {
    parse_bibliography_source_with_origin(body, None, 1)
}

fn parse_bibliography_source_with_origin(
    body: &str,
    source_file: Option<&str>,
    start_line: usize,
) -> Vec<BibliographyEntry> {
    if let Ok(entries) = parse_csl_json_bibliography(body, source_file, start_line) {
        return entries;
    }
    if let Ok(entries) = parse_hayagriva_yaml_bibliography(body, source_file, start_line) {
        return entries;
    }
    let mut entries = Vec::new();
    for entry in bibtex_entry_slices(body) {
        let key = entry.raw_key.trim();
        if !key.is_empty() {
            let key_offset = entry.key_offset + leading_whitespace_len(entry.raw_key);
            let (line, column) = source_position_for_offset(body, start_line, key_offset);
            let title = bibtex_field(entry.raw, "title").unwrap_or_else(|| entry.kind.to_string());
            entries.push(BibliographyEntry {
                key: key.to_string(),
                title,
                author: bibtex_field(entry.raw, "author"),
                issued: bibtex_issued_year(entry.raw),
                raw: entry.raw.to_string(),
                source_file: source_file.map(ToString::to_string),
                line: Some(line),
                column: Some(column),
                end_column: Some(column + key.len()),
            });
        }
    }
    entries
}

pub(crate) fn duplicate_bibliography_keys(bibliography: &[BibliographyEntry]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut duplicates = BTreeSet::new();
    for entry in bibliography {
        if !seen.insert(entry.key.as_str()) {
            duplicates.insert(entry.key.clone());
        }
    }
    duplicates.into_iter().collect()
}

struct BibtexEntrySlice<'a> {
    kind: &'a str,
    raw_key: &'a str,
    raw: &'a str,
    key_offset: usize,
}

fn bibtex_entry_slices(body: &str) -> Vec<BibtexEntrySlice<'_>> {
    let mut entries = Vec::new();
    let mut search_from = 0usize;
    while let Some(relative_at) = body[search_from..].find('@') {
        let at = search_from + relative_at;
        let mut kind_start = at + 1;
        while let Some(ch) = body[kind_start..].chars().next() {
            if !ch.is_whitespace() {
                break;
            }
            kind_start += ch.len_utf8();
        }
        let kind_end = body[kind_start..]
            .char_indices()
            .find_map(|(index, ch)| (!is_bibtex_identifier_char(ch)).then_some(kind_start + index))
            .unwrap_or(body.len());
        let kind = body[kind_start..kind_end].trim();
        if kind.is_empty() {
            search_from = at + 1;
            continue;
        }
        let mut open_index = kind_end;
        while let Some(ch) = body[open_index..].chars().next() {
            if !ch.is_whitespace() {
                break;
            }
            open_index += ch.len_utf8();
        }
        let Some(open) = body[open_index..].chars().next() else {
            break;
        };
        if !matches!(open, '{' | '(') {
            search_from = open_index + open.len_utf8();
            continue;
        }
        let content_start = open_index + open.len_utf8();
        let Some(close_index) = find_bibtex_entry_close(body, open_index, open) else {
            break;
        };
        let content = &body[content_start..close_index];
        if !matches!(
            kind.to_ascii_lowercase().as_str(),
            "comment" | "preamble" | "string"
        ) {
            if let Some(comma_index) = find_top_level_bibtex_comma(content) {
                entries.push(BibtexEntrySlice {
                    kind,
                    raw_key: &content[..comma_index],
                    raw: &content[comma_index + 1..],
                    key_offset: content_start,
                });
            }
        }
        search_from = close_index + 1;
    }
    entries
}

fn find_bibtex_entry_close(body: &str, open_index: usize, open: char) -> Option<usize> {
    let mut delimiter_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut in_quote = false;
    let mut escaped = false;
    for (index, ch) in body[open_index..].char_indices() {
        let absolute_index = open_index + index;
        if in_quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_quote = false;
            }
            continue;
        }
        if ch == '"' {
            in_quote = true;
            continue;
        }
        if ch == '{' {
            brace_depth += 1;
            if open == '{' {
                delimiter_depth += 1;
            }
            continue;
        }
        if ch == '}' {
            brace_depth = brace_depth.saturating_sub(1);
            if open == '{' {
                delimiter_depth = delimiter_depth.saturating_sub(1);
                if delimiter_depth == 0 {
                    return Some(absolute_index);
                }
            }
            continue;
        }
        if open == '(' && ch == '(' {
            delimiter_depth += 1;
            continue;
        }
        if open == '(' && ch == ')' && brace_depth == 0 {
            delimiter_depth = delimiter_depth.saturating_sub(1);
            if delimiter_depth == 0 {
                return Some(absolute_index);
            }
        }
    }
    None
}

fn find_top_level_bibtex_comma(content: &str) -> Option<usize> {
    let mut brace_depth = 0usize;
    let mut in_quote = false;
    let mut escaped = false;
    for (index, ch) in content.char_indices() {
        if in_quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_quote = false;
            }
            continue;
        }
        match ch {
            '"' => in_quote = true,
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            ',' if brace_depth == 0 => return Some(index),
            _ => {}
        }
    }
    None
}

fn parse_hayagriva_yaml_bibliography(
    body: &str,
    source_file: Option<&str>,
    start_line: usize,
) -> Result<Vec<BibliographyEntry>, serde_yaml::Error> {
    let spans = hayagriva_entry_spans(body, start_line);
    if !spans.is_empty() {
        return Ok(spans
            .into_iter()
            .map(|span| {
                let entry = serde_yaml::from_str::<Value>(&span.body).unwrap_or(Value::Null);
                BibliographyEntry {
                    key: span.key.clone(),
                    title: entry
                        .get("title")
                        .and_then(Value::as_str)
                        .unwrap_or(&span.key)
                        .to_string(),
                    author: yaml_author(&entry),
                    issued: yaml_issued_year(&entry),
                    raw: span.body,
                    source_file: source_file.map(ToString::to_string),
                    line: Some(span.line),
                    column: Some(span.column),
                    end_column: Some(span.column + span.key.len()),
                }
            })
            .collect());
    }

    let value = serde_yaml::from_str::<Value>(body)?;
    let entries = value
        .as_object()
        .into_iter()
        .flat_map(|entries| entries.iter())
        .map(|(key, entry)| {
            let title = entry
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or(key)
                .to_string();
            let location = bibliography_key_location(body, key, start_line);
            BibliographyEntry {
                key: key.to_string(),
                title,
                author: yaml_author(entry),
                issued: yaml_issued_year(entry),
                raw: serde_yaml::to_string(entry).unwrap_or_default(),
                source_file: source_file.map(ToString::to_string),
                line: location.map(|location| location.0),
                column: location.map(|location| location.1),
                end_column: location.map(|location| location.1 + key.len()),
            }
        })
        .collect();
    Ok(entries)
}

fn parse_csl_json_bibliography(
    body: &str,
    source_file: Option<&str>,
    start_line: usize,
) -> Result<Vec<BibliographyEntry>, serde_json::Error> {
    let value = serde_json::from_str::<Value>(body)?;
    let mut occurrence_counts = BTreeMap::<String, usize>::new();
    let entries = csl_json_entries(&value)
        .into_iter()
        .filter_map(|entry| {
            let key = entry
                .get("id")
                .or_else(|| entry.get("citation-key"))
                .and_then(Value::as_str)?;
            let occurrence = occurrence_counts.entry(key.to_string()).or_insert(0);
            let location =
                bibliography_key_location_for_occurrence(body, key, start_line, *occurrence);
            *occurrence += 1;
            let title = entry
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or(key)
                .to_string();
            Some(BibliographyEntry {
                key: key.to_string(),
                title,
                author: csl_author(entry),
                issued: csl_issued_year(entry),
                raw: entry.to_string(),
                source_file: source_file.map(ToString::to_string),
                line: location.map(|location| location.0),
                column: location.map(|location| location.1),
                end_column: location.map(|location| location.1 + key.len()),
            })
        })
        .collect();
    Ok(entries)
}

fn csl_json_entries(value: &Value) -> Vec<&Value> {
    if let Some(entries) = value.as_array() {
        return entries.iter().collect();
    }
    if let Some(object) = value.as_object() {
        for key in ["items", "references", "bibliography", "data"] {
            if let Some(entries) = object.get(key).and_then(Value::as_array) {
                return entries.iter().collect();
            }
        }
        if object.contains_key("id") || object.contains_key("citation-key") {
            return vec![value];
        }
    }
    Vec::new()
}

struct HayagrivaEntrySpan {
    key: String,
    body: String,
    line: usize,
    column: usize,
}

fn hayagriva_entry_spans(body: &str, start_line: usize) -> Vec<HayagrivaEntrySpan> {
    let mut starts = Vec::new();
    let mut byte_offset = 0usize;
    for (line_index, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if !line
            .chars()
            .next()
            .map(|ch| ch.is_whitespace())
            .unwrap_or(false)
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && trimmed.ends_with(':')
        {
            let key = trimmed.trim_end_matches(':').trim();
            if !key.is_empty()
                && key
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | ':' | '.'))
            {
                let column = line.find(key).map(|index| index + 1).unwrap_or(1);
                starts.push((byte_offset, line_index, key.to_string(), column));
            }
        }
        byte_offset += line.len() + 1;
    }
    starts
        .iter()
        .enumerate()
        .map(|(index, (offset, line_index, key, column))| {
            let next_offset = starts
                .get(index + 1)
                .map(|(next_offset, _, _, _)| *next_offset)
                .unwrap_or(body.len());
            let raw = &body[*offset..next_offset];
            let body_start = raw
                .find('\n')
                .map(|newline| newline + 1)
                .unwrap_or(raw.len());
            HayagrivaEntrySpan {
                key: key.clone(),
                body: raw[body_start..].trim_end().to_string(),
                line: start_line + line_index,
                column: *column,
            }
        })
        .collect()
}

fn leading_whitespace_len(text: &str) -> usize {
    text.len() - text.trim_start().len()
}

fn source_position_for_offset(body: &str, start_line: usize, offset: usize) -> (usize, usize) {
    let mut line = start_line;
    let mut column = 1usize;
    for (index, ch) in body.char_indices() {
        if index >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

fn bibliography_key_location(body: &str, key: &str, start_line: usize) -> Option<(usize, usize)> {
    bibliography_key_location_for_occurrence(body, key, start_line, 0)
}

fn bibliography_key_location_for_occurrence(
    body: &str,
    key: &str,
    start_line: usize,
    occurrence: usize,
) -> Option<(usize, usize)> {
    let mut seen = 0usize;
    for (index, line) in body.lines().enumerate() {
        if let Some(column) = line.find(key) {
            if seen == occurrence {
                return Some((start_line + index, column + 1));
            }
            seen += 1;
        }
    }
    None
}

fn yaml_author(entry: &Value) -> Option<String> {
    entry.get("author").and_then(|author| match author {
        Value::String(value) => Some(value.clone()),
        Value::Object(author) => author_name_from_object(author),
        Value::Array(values) => values.first().and_then(|value| match value {
            Value::String(value) => Some(value.clone()),
            Value::Object(author) => author_name_from_object(author),
            _ => None,
        }),
        _ => None,
    })
}

fn yaml_issued_year(entry: &Value) -> Option<String> {
    ["date", "year", "issued"]
        .into_iter()
        .find_map(|field| entry.get(field))
        .and_then(year_from_value)
}

fn bibtex_field(raw: &str, field: &str) -> Option<String> {
    let field_start = find_bibtex_field(raw, field)?;
    let after_field = field_start + field.len();
    let equals_offset = raw[after_field..].find('=')? + after_field;
    let value_start = raw[equals_offset + 1..]
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(index, _)| equals_offset + 1 + index)?;
    let value = parse_bibtex_value(&raw[value_start..]);
    Some(clean_bibliography_value(&value)).filter(|value| !value.is_empty())
}

fn find_bibtex_field(raw: &str, field: &str) -> Option<usize> {
    for (index, _) in raw.char_indices() {
        let Some(candidate) = raw.get(index..index + field.len()) else {
            continue;
        };
        if !candidate.eq_ignore_ascii_case(field) {
            continue;
        }
        let before = raw[..index].chars().next_back();
        if before.is_some_and(is_bibtex_identifier_char) {
            continue;
        }
        let after = &raw[index + field.len()..];
        if after
            .chars()
            .find(|ch| !ch.is_whitespace())
            .is_some_and(|ch| ch == '=')
        {
            return Some(index);
        }
    }
    None
}

fn parse_bibtex_value(value: &str) -> String {
    let Some(first) = value.chars().next() else {
        return String::new();
    };
    if first == '{' {
        let mut depth = 0usize;
        let mut end = value.len();
        for (index, ch) in value.char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        end = index + ch.len_utf8();
                        break;
                    }
                }
                _ => {}
            }
        }
        return value[..end].to_string();
    }
    if first == '"' {
        let mut escaped = false;
        for (index, ch) in value.char_indices().skip(1) {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == '"' {
                return value[..index + ch.len_utf8()].to_string();
            }
        }
        return value.to_string();
    }
    value
        .split([',', '\n'])
        .next()
        .unwrap_or_default()
        .to_string()
}

fn is_bibtex_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')
}

fn clean_bibliography_value(value: &str) -> String {
    let trimmed = value
        .trim()
        .trim_matches(&['{', '}', ',', '"'][..])
        .trim()
        .to_string();
    trimmed
        .replace("\\&", "&")
        .replace(['{', '}'], "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn bibtex_issued_year(raw: &str) -> Option<String> {
    bibtex_field(raw, "year").or_else(|| {
        bibtex_field(raw, "date").and_then(|date| {
            let year = date.chars().take(4).collect::<String>();
            (year.len() == 4 && year.chars().all(|ch| ch.is_ascii_digit())).then_some(year)
        })
    })
}

fn csl_author(entry: &Value) -> Option<String> {
    entry
        .get("author")
        .and_then(Value::as_array)
        .and_then(|authors| authors.first())
        .and_then(|author| {
            author
                .as_object()
                .and_then(author_name_from_object)
                .or_else(|| {
                    author
                        .get("literal")
                        .and_then(Value::as_str)
                        .or_else(|| author.get("family").and_then(Value::as_str))
                        .or_else(|| author.get("name").and_then(Value::as_str))
                        .map(ToString::to_string)
                })
        })
}

fn csl_issued_year(entry: &Value) -> Option<String> {
    entry.get("issued").and_then(year_from_value)
}

fn author_name_from_object(author: &serde_json::Map<String, Value>) -> Option<String> {
    if let Some(literal) = author.get("literal").and_then(Value::as_str) {
        return Some(literal.to_string());
    }
    if let Some(name) = author.get("name").and_then(Value::as_str) {
        return Some(name.to_string());
    }
    let given = author
        .get("given")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let family = author
        .get("family")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    match (given.is_empty(), family.is_empty()) {
        (false, false) => Some(format!("{given} {family}")),
        (false, true) => Some(given.to_string()),
        (true, false) => Some(family.to_string()),
        (true, true) => None,
    }
}

fn year_from_value(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => first_year(value),
        Value::Number(value) => Some(value.to_string()).filter(|year| is_year(year)),
        Value::Object(object) => object
            .get("date-parts")
            .and_then(Value::as_array)
            .and_then(|date_parts| date_parts.first())
            .and_then(Value::as_array)
            .and_then(|first_date| first_date.first())
            .and_then(year_from_value)
            .or_else(|| {
                ["raw", "literal", "date", "year"]
                    .into_iter()
                    .filter_map(|key| object.get(key))
                    .find_map(year_from_value)
            }),
        _ => None,
    }
}

fn first_year(value: &str) -> Option<String> {
    value
        .split(|ch: char| !ch.is_ascii_digit())
        .find(|part| is_year(part))
        .map(ToString::to_string)
}

fn is_year(value: &str) -> bool {
    value.len() == 4 && value.chars().all(|ch| ch.is_ascii_digit())
}

pub(crate) fn collect_citation_references(text: &str) -> Vec<CitationReference> {
    let mut citations = Vec::new();
    let mut fence_marker = None;
    for (index, line) in text.lines().enumerate() {
        if let Some(marker) = fence_marker {
            if line.trim_start().starts_with(marker) {
                fence_marker = None;
            }
            continue;
        }
        if let Some(marker) = fenced_code_marker(line) {
            fence_marker = Some(marker);
            continue;
        }
        let mut search_from = 0usize;
        while let Some(relative_start) = line[search_from..].find('[') {
            let start = search_from + relative_start;
            let inside_start = start + 1;
            let Some(relative_end) = line[inside_start..].find(']') else {
                break;
            };
            let inside = &line[inside_start..inside_start + relative_end];
            if inside.contains('@') {
                citations.extend(citation_references_from_bracket(
                    inside,
                    index + 1,
                    inside_start + 1,
                ));
            }
            search_from = inside_start + relative_end + 1;
        }
    }
    citations
}

pub(crate) fn citation_keys_from_references(references: &[CitationReference]) -> Vec<String> {
    let mut citations = BTreeSet::new();
    for reference in references {
        citations.insert(reference.key.clone());
    }
    citations.into_iter().collect()
}

fn citation_references_from_bracket(
    text: &str,
    line: usize,
    bracket_content_column: usize,
) -> Vec<CitationReference> {
    let mut references = Vec::new();
    let mut rest = text;
    let mut consumed = 0usize;
    while let Some(index) = rest.find('@') {
        let after_at = &rest[index + 1..];
        let key = after_at
            .chars()
            .take_while(|ch| is_citation_key_char(*ch))
            .collect::<String>();
        let key_len = key.len();
        if !key.is_empty() {
            let column = bracket_content_column + consumed + index;
            let after_key = &after_at[key_len..];
            let locator_end = after_key.find('@').unwrap_or(after_key.len());
            let locator = after_key[..locator_end]
                .trim()
                .trim_start_matches([',', ';'])
                .trim()
                .trim_end_matches(';')
                .trim();
            references.push(CitationReference {
                key,
                locator: (!locator.is_empty()).then(|| locator.to_string()),
                raw: text.to_string(),
                line,
                column,
                end_column: column + key_len + 1,
            });
        }
        let advance = index + 1 + key_len;
        consumed += advance;
        rest = &rest[advance..];
    }
    references
}

fn is_citation_key_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | ':' | '.' | '/')
}

pub(crate) fn render_citations(
    markdown: &str,
    bibliography: &[BibliographyEntry],
    style: &str,
) -> String {
    let context = CitationRenderContext::new(bibliography);
    let mut lines = Vec::new();
    let mut fence_marker = None;
    for line in markdown.lines() {
        if let Some(marker) = fence_marker {
            lines.push(line.to_string());
            if line.trim_start().starts_with(marker) {
                fence_marker = None;
            }
            continue;
        }
        if let Some(marker) = fenced_code_marker(line) {
            lines.push(line.to_string());
            fence_marker = Some(marker);
            continue;
        }
        lines.push(render_citation_line(line, &context, style));
    }
    let mut output = lines.join("\n");
    if markdown.ends_with('\n') {
        output.push('\n');
    }
    output
}

struct CitationRenderContext<'a> {
    entries: HashMap<&'a str, &'a BibliographyEntry>,
    numbers: HashMap<&'a str, usize>,
}

impl<'a> CitationRenderContext<'a> {
    fn new(bibliography: &'a [BibliographyEntry]) -> Self {
        Self {
            entries: bibliography
                .iter()
                .map(|entry| (entry.key.as_str(), entry))
                .collect(),
            numbers: bibliography
                .iter()
                .enumerate()
                .map(|(index, entry)| (entry.key.as_str(), index + 1))
                .collect(),
        }
    }
}

fn render_citation_line(line: &str, context: &CitationRenderContext<'_>, style: &str) -> String {
    let mut output = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(start) = rest.find('[') {
        output.push_str(&rest[..start]);
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find(']') else {
            output.push_str(&rest[start..]);
            return output;
        };
        let inside = &after_start[..end];
        if inside.contains('@') {
            let references = citation_references_from_bracket(inside, 0, 1);
            output.push_str(&render_citation_span(&references, context, style));
        } else {
            output.push('[');
            output.push_str(inside);
            output.push(']');
        }
        rest = &after_start[end + 1..];
    }
    output.push_str(rest);
    output
}

fn render_citation_span(
    references: &[CitationReference],
    context: &CitationRenderContext<'_>,
    style: &str,
) -> String {
    if references.is_empty() {
        return String::new();
    }
    let keys = references
        .iter()
        .map(|reference| reference.key.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let label = references
        .iter()
        .map(|reference| {
            let mut label = citation_label(reference, context, style);
            if let Some(locator) = &reference.locator {
                label.push_str(", ");
                label.push_str(locator);
            }
            label
        })
        .collect::<Vec<_>>()
        .join("; ");
    let details = references
        .iter()
        .map(|reference| {
            let title = context
                .entries
                .get(reference.key.as_str())
                .map(|entry| entry.title.as_str())
                .unwrap_or("missing bibliography entry");
            match &reference.locator {
                Some(locator) => format!("@{} ({locator}): {title}", reference.key),
                None => format!("@{}: {title}", reference.key),
            }
        })
        .collect::<Vec<_>>()
        .join("; ");
    let display_label = if style == "numeric" {
        format!("[{label}]")
    } else {
        format!("({label})")
    };
    format!(
        "<span class=\"citation\" tabindex=\"0\" title=\"{}\" aria-label=\"Citation: {}\" data-citation-keys=\"{}\" data-citation-detail=\"{}\">{}</span>",
        escape_html(&details),
        escape_html(&details),
        escape_html(&keys),
        escape_html(&details),
        escape_html(&display_label)
    )
}

fn citation_label(
    reference: &CitationReference,
    context: &CitationRenderContext<'_>,
    style: &str,
) -> String {
    if style == "numeric" {
        return context
            .numbers
            .get(reference.key.as_str())
            .map(|number| number.to_string())
            .unwrap_or_else(|| reference.key.clone());
    }
    let entry = context.entries.get(reference.key.as_str()).copied();
    let Some(entry) = entry else {
        return reference.key.clone();
    };
    match style {
        "key" => format!("@{}", reference.key),
        "author-year" => match (&entry.author, &entry.issued) {
            (Some(author), Some(year)) => format!("{author} {year}"),
            (Some(author), None) => author.clone(),
            (None, Some(year)) => year.clone(),
            (None, None) => entry.title.clone(),
        },
        _ => entry.title.clone(),
    }
}
