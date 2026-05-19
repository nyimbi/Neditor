use crate::Heading;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub(crate) struct AiSource {
    pub(crate) line: usize,
    pub(crate) provider: String,
    pub(crate) model: String,
    pub(crate) date: String,
    pub(crate) prompt_summary: String,
    pub(crate) reviewed_by: String,
    pub(crate) reviewed_at: String,
    pub(crate) status: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AiAssistedSection {
    pub(crate) line: usize,
    pub(crate) heading: String,
    pub(crate) status: String,
    pub(crate) reviewed_by: String,
    pub(crate) reviewed_at: String,
    pub(crate) source: String,
    pub(crate) prompt_summary: String,
}

pub(crate) fn collect_ai_sources(text: &str) -> Vec<AiSource> {
    collect_ai_fence_bodies_with_lines(text, "ai-source")
        .into_iter()
        .map(|(line, body)| {
            let map = body
                .lines()
                .filter_map(|line| line.split_once(':'))
                .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
                .collect::<HashMap<_, _>>();
            AiSource {
                line,
                provider: map.get("provider").cloned().unwrap_or_default(),
                model: map.get("model").cloned().unwrap_or_default(),
                date: map.get("date").cloned().unwrap_or_default(),
                prompt_summary: map
                    .get("promptSummary")
                    .or_else(|| map.get("prompt_summary"))
                    .or_else(|| map.get("prompt"))
                    .cloned()
                    .unwrap_or_default(),
                reviewed_by: map
                    .get("reviewedBy")
                    .or_else(|| map.get("reviewer"))
                    .or_else(|| map.get("reviewed_by"))
                    .cloned()
                    .unwrap_or_default(),
                reviewed_at: map
                    .get("reviewedAt")
                    .or_else(|| map.get("reviewed_at"))
                    .or_else(|| map.get("reviewDate"))
                    .cloned()
                    .unwrap_or_default(),
                status: map
                    .get("status")
                    .cloned()
                    .unwrap_or_else(|| "unreviewed".to_string()),
            }
        })
        .collect()
}

pub(crate) fn collect_ai_assisted_sections(
    text: &str,
    headings: &[Heading],
) -> Vec<AiAssistedSection> {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line_number = index + 1;
            let trimmed = line.trim();
            if let Some(content) = trimmed
                .strip_prefix("<!-- ai-assisted:")
                .and_then(|content| content.strip_suffix("-->"))
            {
                return Some(parse_ai_assisted_section(line_number, content, headings));
            }
            if trimmed == "<!-- draft: AI paste cleanup review required -->" {
                return Some(AiAssistedSection {
                    line: line_number,
                    heading: ai_section_heading(line_number, headings),
                    status: "needs-review".to_string(),
                    reviewed_by: String::new(),
                    reviewed_at: String::new(),
                    source: "AI paste cleanup".to_string(),
                    prompt_summary: "AI paste cleanup review required".to_string(),
                });
            }
            None
        })
        .collect()
}

fn collect_ai_fence_bodies_with_lines(text: &str, target: &str) -> Vec<(usize, String)> {
    let mut bodies = Vec::new();
    let mut lines = text.lines().enumerate();
    while let Some((line_index, line)) = lines.next() {
        if line
            .trim()
            .strip_prefix("```")
            .map(|info| info.split_whitespace().next().unwrap_or("") == target)
            .unwrap_or(false)
        {
            let mut body = String::new();
            for (_, body_line) in lines.by_ref() {
                if body_line.trim() == "```" {
                    break;
                }
                body.push_str(body_line);
                body.push('\n');
            }
            bodies.push((line_index + 1, body));
        }
    }
    bodies
}

fn parse_ai_assisted_section(
    line: usize,
    content: &str,
    headings: &[Heading],
) -> AiAssistedSection {
    let mut status = "needs-review".to_string();
    let mut reviewed_by = String::new();
    let mut reviewed_at = String::new();
    let mut source = String::new();
    let mut prompt_summary = String::new();

    for part in content
        .split('|')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        if matches!(part, "human-reviewed" | "needs-review" | "unreviewed") {
            status = part.to_string();
        } else if let Some((key, value)) = part.split_once(':').or_else(|| part.split_once('=')) {
            let key = key.trim();
            let value = value.trim().to_string();
            match key {
                "status" => status = value,
                "reviewedBy" | "reviewed_by" | "reviewer" => reviewed_by = value,
                "reviewedAt" | "reviewed_at" | "reviewDate" => reviewed_at = value,
                "source" => source = value,
                "promptSummary" | "prompt_summary" | "prompt" => prompt_summary = value,
                _ => {}
            }
        }
    }

    AiAssistedSection {
        line,
        heading: ai_section_heading(line, headings),
        status,
        reviewed_by,
        reviewed_at,
        source,
        prompt_summary,
    }
}

fn ai_section_heading(line: usize, headings: &[Heading]) -> String {
    headings
        .iter()
        .min_by_key(|heading| heading.line.abs_diff(line))
        .map(|heading| heading.text.clone())
        .unwrap_or_else(|| "Document body".to_string())
}
