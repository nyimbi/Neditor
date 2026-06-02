use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct MailMergeRequest {
    pub template_path: String,
    pub data_path: String,   // CSV or JSON file path
    pub output_dir: String,
    pub filename_field: String, // which CSV column to use as output filename
    pub workspace_root: Option<String>, // when set, all paths must be contained within it
}

/// RFC 4180-compliant CSV/TSV field parser.
fn parse_delimited_line(line: &str, sep: char) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if !in_quotes => { in_quotes = true; }
            '"' if in_quotes => {
                if chars.peek() == Some(&'"') {
                    chars.next(); // escaped double-quote inside quoted field
                    current.push('"');
                } else {
                    in_quotes = false;
                }
            }
            c if c == sep && !in_quotes => {
                fields.push(current.trim().to_string());
                current = String::new();
            }
            _ => { current.push(c); }
        }
    }
    fields.push(current.trim().to_string());
    fields
}

fn safe_path(path: &str, workspace_root: &Option<String>) -> Result<PathBuf, String> {
    let p = PathBuf::from(path);
    let canonical = p.canonicalize().unwrap_or_else(|_| p.clone());
    if let Some(root) = workspace_root {
        let root_canon = PathBuf::from(root).canonicalize().unwrap_or_else(|_| PathBuf::from(root));
        if !canonical.starts_with(&root_canon) {
            return Err(format!("Path '{}' is outside the workspace root.", path));
        }
    }
    Ok(p)
}

#[derive(Debug, Serialize)]
pub struct MailMergeResult {
    pub generated: usize,
    pub output_paths: Vec<String>,
    pub errors: Vec<String>,
}

#[tauri::command]
pub(crate) fn run_mail_merge(request: MailMergeRequest) -> Result<MailMergeResult, String> {
    let template_path = safe_path(&request.template_path, &request.workspace_root)?;
    let data_path_safe = safe_path(&request.data_path, &request.workspace_root)?;
    let out_dir_safe = safe_path(&request.output_dir, &request.workspace_root)?;

    let template = fs::read_to_string(&template_path)
        .map_err(|e| format!("Cannot read template: {e}"))?;
    let data_path = PathBuf::from(&data_path_safe);
    let ext = data_path.extension().and_then(|e| e.to_str()).unwrap_or_default();
    let records: Vec<std::collections::HashMap<String, String>> = if ext == "csv" || ext == "tsv" {
        let sep = if ext == "tsv" { '\t' } else { ',' };
        let content = fs::read_to_string(&data_path).map_err(|e| e.to_string())?;
        let mut lines = content.lines();
        let headers = parse_delimited_line(lines.next().unwrap_or_default(), sep);
        lines.map(|line| {
            let values = parse_delimited_line(line, sep);
            headers.iter().zip(values.into_iter().chain(std::iter::repeat(String::new())))
                .map(|(k, v)| (k.clone(), v)).collect()
        }).collect()
    } else if ext == "json" {
        let content = fs::read_to_string(&data_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())?
    } else {
        return Err(format!("Unsupported data format: .{ext}. Use CSV or JSON."));
    };
    let out_dir = out_dir_safe;
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let mut output_paths = Vec::new();
    let mut errors = Vec::new();
    for record in &records {
        let mut merged = template.clone();
        for (key, value) in record {
            merged = merged.replace(&format!("{{{{{key}}}}}"), value);
        }
        let filename = record.get(&request.filename_field)
            .map(|s| s.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_"))
            .unwrap_or_else(|| format!("document_{}", output_paths.len() + 1));
        let out_path = out_dir.join(format!("{filename}.md"));
        match fs::write(&out_path, &merged) {
            Ok(_) => output_paths.push(out_path.to_string_lossy().to_string()),
            Err(e) => errors.push(format!("{filename}: {e}")),
        }
    }
    Ok(MailMergeResult { generated: output_paths.len(), output_paths, errors })
}
