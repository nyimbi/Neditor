use serde::Serialize;

#[derive(Clone, Debug, Default, Serialize)]
pub(crate) struct LayoutSettings {
    pub(crate) columns: Option<usize>,
    pub(crate) page_size: Option<String>,
    pub(crate) orientation: Option<String>,
    pub(crate) margins: Option<String>,
    pub(crate) break_before: Option<String>,
    pub(crate) break_after: Option<String>,
    pub(crate) keep_with_next: bool,
    pub(crate) keep_together: bool,
    pub(crate) header: Option<String>,
    pub(crate) footer: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) layout: Option<String>,
    pub(crate) notes: Option<String>,
}

impl LayoutSettings {
    pub(crate) fn from_options(options: &str) -> Self {
        Self {
            columns: layout_columns(options),
            page_size: layout_page_size_option(options),
            orientation: layout_orientation_option(options),
            margins: layout_margins_option(options),
            break_before: layout_break_before(options),
            break_after: layout_break_after(options),
            keep_with_next: layout_bool_option_any(
                options,
                &["keepWithNext", "keep-with-next", "keep_with_next"],
            ),
            keep_together: layout_bool_option_any(
                options,
                &[
                    "keepTogether",
                    "keep-together",
                    "keep_together",
                    "avoidBreakInside",
                    "avoid-break-inside",
                    "avoid_break_inside",
                ],
            ),
            header: layout_option_text(options, "header"),
            footer: layout_option_text(options, "footer"),
            title: layout_option_text(options, "title"),
            layout: layout_option_text_any(options, &["layout", "type", "kind"]),
            notes: layout_option_text_any(options, &["notes", "speakerNotes", "speaker_notes"]),
        }
    }

    pub(crate) fn has_pagination_controls(&self) -> bool {
        self.break_before.is_some()
            || self.break_after.is_some()
            || self.keep_with_next
            || self.keep_together
    }

    pub(crate) fn has_page_model_controls(&self) -> bool {
        self.page_size.is_some() || self.orientation.is_some() || self.margins.is_some()
    }
}

pub(crate) fn layout_css_style(options: &str) -> String {
    let settings = LayoutSettings::from_options(options);
    let mut styles = Vec::new();
    if let Some(columns) = settings.columns {
        styles.push(format!("column-count:{columns}"));
        styles.push("column-gap:32px".to_string());
    }
    if let Some(orientation) = &settings.orientation {
        styles.push(format!("page:neditor-{orientation}"));
    }
    if let Some(page_size) = &settings.page_size {
        styles.push(format!("--neditor-page-size:{page_size}"));
    }
    if let Some(margins) = &settings.margins {
        styles.push(format!("--neditor-page-margins:{margins}"));
    }
    if matches_layout_break(settings.break_before.as_deref()) {
        styles.push("break-before:page".to_string());
        styles.push("page-break-before:always".to_string());
    }
    if matches_layout_break(settings.break_after.as_deref()) {
        styles.push("break-after:page".to_string());
        styles.push("page-break-after:always".to_string());
    } else if settings.keep_with_next {
        styles.push("break-after:avoid".to_string());
        styles.push("page-break-after:avoid".to_string());
    }
    if settings.keep_together {
        styles.push("break-inside:avoid".to_string());
        styles.push("page-break-inside:avoid".to_string());
    }
    styles.join(";")
}

pub(crate) fn layout_option_text_any(text: &str, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| layout_option_text(text, key))
}

pub(crate) fn layout_option_text(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        if let Some((candidate, value)) = line.split_once(':') {
            if candidate.trim() == key {
                return Some(value.trim().trim_matches('"').to_string());
            }
        }
    }
    let prefix = format!("{key}=");
    let start = text.find(&prefix)? + prefix.len();
    let value = &text[start..];
    if let Some(quoted) = value.strip_prefix('"') {
        return quoted.split_once('"').map(|(title, _)| title.to_string());
    }
    value
        .split_whitespace()
        .next()
        .filter(|value| !value.is_empty())
        .map(|value| value.trim_matches('"').to_string())
}

pub(crate) fn layout_columns(options: &str) -> Option<usize> {
    for line in options.lines() {
        if let Some((key, value)) = line.split_once(':') {
            if key.trim() == "columns" {
                return parse_layout_columns(value);
            }
        }
    }
    for part in options.split_whitespace() {
        if let Some((key, value)) = part.split_once('=') {
            if key.trim() == "columns" {
                return parse_layout_columns(value);
            }
        }
    }
    None
}

pub(crate) fn layout_page_size_option(options: &str) -> Option<String> {
    layout_option_text_any(
        options,
        &["pageSize", "page-size", "page_size", "paper", "size"],
    )
    .and_then(|value| normalize_page_size_value(&value))
}

pub(crate) fn layout_orientation_option(options: &str) -> Option<String> {
    layout_option_text_any(
        options,
        &["orientation", "pageOrientation", "page_orientation"],
    )
    .and_then(|value| normalize_orientation_value(&value))
}

pub(crate) fn layout_margins_option(options: &str) -> Option<String> {
    layout_option_text_any(
        options,
        &["margins", "margin", "pageMargins", "page_margins"],
    )
    .and_then(|value| normalize_margins_value(&value))
}

pub(crate) fn layout_break_before(options: &str) -> Option<String> {
    normalize_break_value(layout_option_text_any(
        options,
        &[
            "breakBefore",
            "break-before",
            "break_before",
            "pageBreakBefore",
        ],
    ))
}

pub(crate) fn layout_break_after(options: &str) -> Option<String> {
    normalize_break_value(layout_option_text_any(
        options,
        &["breakAfter", "break-after", "break_after", "pageBreakAfter"],
    ))
}

pub(crate) fn matches_layout_break(value: Option<&str>) -> bool {
    matches!(value, Some("page" | "always"))
}

fn parse_layout_columns(value: &str) -> Option<usize> {
    let columns = value.trim().trim_matches('"').parse::<usize>().ok()?;
    (columns > 0).then_some(columns)
}

fn layout_bool_option_any(text: &str, keys: &[&str]) -> bool {
    keys.iter().any(|key| {
        layout_option_text(text, key)
            .map(|value| matches_bool_true(&value))
            .unwrap_or(false)
    })
}

fn matches_bool_true(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on" | "avoid"
    )
}

fn normalize_break_value(value: Option<String>) -> Option<String> {
    value.and_then(|value| match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "page" | "always" => Some("page".to_string()),
        "slide" => Some("slide".to_string()),
        "column" => Some("column".to_string()),
        _ => None,
    })
}

fn normalize_page_size_value(value: &str) -> Option<String> {
    match value
        .trim()
        .trim_matches('"')
        .to_ascii_lowercase()
        .replace([' ', '-'], "")
        .as_str()
    {
        "letter" | "usletter" => Some("letter".to_string()),
        "legal" | "uslegal" => Some("legal".to_string()),
        "a4" => Some("a4".to_string()),
        _ => None,
    }
}

fn normalize_orientation_value(value: &str) -> Option<String> {
    match value
        .trim()
        .trim_matches('"')
        .to_ascii_lowercase()
        .replace([' ', '-'], "")
        .as_str()
    {
        "landscape" => Some("landscape".to_string()),
        "portrait" => Some("portrait".to_string()),
        _ => None,
    }
}

fn normalize_margins_value(value: &str) -> Option<String> {
    match value
        .trim()
        .trim_matches('"')
        .to_ascii_lowercase()
        .replace([' ', '-'], "")
        .as_str()
    {
        "narrow" | "compact" => Some("narrow".to_string()),
        "normal" => Some("normal".to_string()),
        "wide" => Some("wide".to_string()),
        _ => None,
    }
}
