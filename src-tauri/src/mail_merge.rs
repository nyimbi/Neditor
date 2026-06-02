use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct MailMergeRequest {
    pub template_path: String,
    pub data_path: String,   // CSV or JSON file path
    pub output_dir: String,
    pub filename_field: String, // which CSV column to use as output filename
}

#[derive(Debug, Serialize)]
pub struct MailMergeResult {
    pub generated: usize,
    pub output_paths: Vec<String>,
    pub errors: Vec<String>,
}

#[tauri::command]
pub(crate) fn run_mail_merge(request: MailMergeRequest) -> Result<MailMergeResult, String> {
    let template = fs::read_to_string(&request.template_path)
        .map_err(|e| format!("Cannot read template: {e}"))?;
    let data_path = PathBuf::from(&request.data_path);
    let ext = data_path.extension().and_then(|e| e.to_str()).unwrap_or_default();
    let records: Vec<std::collections::HashMap<String, String>> = if ext == "csv" || ext == "tsv" {
        let sep = if ext == "tsv" { b'\t' } else { b',' };
        let content = fs::read_to_string(&data_path).map_err(|e| e.to_string())?;
        let mut lines = content.lines();
        let headers: Vec<String> = lines.next().unwrap_or_default().split(sep as char).map(|s| s.trim().to_string()).collect();
        lines.map(|line| {
            headers.iter().zip(line.split(sep as char).map(|s| s.trim().to_string()))
                .map(|(k, v)| (k.clone(), v)).collect()
        }).collect()
    } else if ext == "json" {
        let content = fs::read_to_string(&data_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())?
    } else {
        return Err(format!("Unsupported data format: .{ext}. Use CSV or JSON."));
    };
    let out_dir = PathBuf::from(&request.output_dir);
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
