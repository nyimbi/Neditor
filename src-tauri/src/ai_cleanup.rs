use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct AiCleanupRequest {
    pub(crate) text: String,
    pub(crate) add_provenance: bool,
    pub(crate) mark_as_draft: bool,
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
    let chat_labels = [
        "ChatGPT said:",
        "Claude said:",
        "Gemini said:",
        "Copilot said:",
        "Assistant:",
        "User:",
    ];
    for label in chat_labels {
        if cleaned.contains(label) {
            cleaned = cleaned.replace(label, "");
            issues.push(format!("Removed chat label '{label}'"));
        }
    }
    cleaned = normalize_markdown_lists(&cleaned, &mut issues);
    cleaned = normalize_markdown_tables(&cleaned, &mut issues);
    if request.mark_as_draft && !cleaned.contains("status: draft") {
        cleaned = format!("<!-- draft: AI paste cleanup review required -->\n\n{cleaned}");
        issues.push("Marked inserted content as draft.".to_string());
    }
    let provenance_block = if request.add_provenance {
        Some(format!(
            "```ai-source\nprovider: unknown\nmodel: unknown\ndate: {}\nreviewedBy: \nstatus: needs-review\n```",
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

fn normalize_markdown_lists(text: &str, issues: &mut Vec<String>) -> String {
    let mut changed = false;
    let output = text
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix("• ") {
                changed = true;
                format!("{}- {}", &line[..line.len() - trimmed.len()], rest)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if changed {
        issues.push("Normalized bullet characters to Markdown list markers.".to_string());
    }
    output
}

fn normalize_markdown_tables(text: &str, issues: &mut Vec<String>) -> String {
    let lines = text.lines().collect::<Vec<_>>();
    let mut output = Vec::new();
    let mut index = 0;
    let mut changed = false;
    while index < lines.len() {
        let line = lines[index];
        if line.contains('\t') {
            let cells = line.split('\t').map(str::trim).collect::<Vec<_>>();
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
