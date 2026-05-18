use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct ReviewComment {
    pub(crate) line: usize,
    pub(crate) author: String,
    pub(crate) created_at: String,
    pub(crate) state: String,
    pub(crate) text: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ChangeNote {
    pub(crate) line: usize,
    pub(crate) author: String,
    pub(crate) created_at: String,
    pub(crate) text: String,
}

pub(crate) fn collect_comments(text: &str) -> Vec<ReviewComment> {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let content = line
                .trim()
                .strip_prefix("<!-- comment:")?
                .strip_suffix("-->")?;
            Some(parse_review_comment(index + 1, content))
        })
        .collect()
}

pub(crate) fn collect_change_notes(text: &str) -> Vec<ChangeNote> {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let content = line
                .trim()
                .strip_prefix("<!-- change:")?
                .strip_suffix("-->")?;
            Some(parse_change_note(index + 1, content))
        })
        .collect()
}

fn parse_review_comment(line: usize, content: &str) -> ReviewComment {
    let mut author = "local".to_string();
    let mut created_at = String::new();
    let mut state = if content.contains("resolved") {
        "resolved"
    } else {
        "unresolved"
    }
    .to_string();
    let mut text_parts = Vec::new();

    for part in content
        .split('|')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        if part == "resolved" || part == "unresolved" {
            state = part.to_string();
        } else if let Some(value) = part
            .strip_prefix("author:")
            .or_else(|| part.strip_prefix("author="))
        {
            author = value.trim().to_string();
        } else if let Some(value) = part
            .strip_prefix("at:")
            .or_else(|| part.strip_prefix("at="))
            .or_else(|| part.strip_prefix("createdAt:"))
            .or_else(|| part.strip_prefix("createdAt="))
        {
            created_at = value.trim().to_string();
        } else {
            text_parts.push(part.to_string());
        }
    }

    ReviewComment {
        line,
        author,
        created_at,
        state,
        text: text_parts.join(" | "),
    }
}

fn parse_change_note(line: usize, content: &str) -> ChangeNote {
    let mut author = "local".to_string();
    let mut created_at = String::new();
    let mut text_parts = Vec::new();

    for part in content
        .split('|')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        if let Some(value) = part
            .strip_prefix("author:")
            .or_else(|| part.strip_prefix("author="))
        {
            author = value.trim().to_string();
        } else if let Some(value) = part
            .strip_prefix("at:")
            .or_else(|| part.strip_prefix("at="))
            .or_else(|| part.strip_prefix("createdAt:"))
            .or_else(|| part.strip_prefix("createdAt="))
        {
            created_at = value.trim().to_string();
        } else {
            text_parts.push(part.to_string());
        }
    }

    ChangeNote {
        line,
        author,
        created_at,
        text: text_parts.join(" | "),
    }
}
