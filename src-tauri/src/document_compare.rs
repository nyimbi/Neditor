use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize)]
pub struct CompareRequest {
    pub path_a: String,
    pub path_b: String,
    pub label_a: Option<String>,
    pub label_b: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DiffLine {
    pub kind: String,  // "added", "removed", "equal"
    pub line_a: Option<usize>,
    pub line_b: Option<usize>,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct CompareResponse {
    pub label_a: String,
    pub label_b: String,
    pub diff: Vec<DiffLine>,
    pub added: usize,
    pub removed: usize,
    pub unchanged: usize,
}

#[tauri::command]
pub(crate) fn compare_documents(request: CompareRequest) -> Result<CompareResponse, String> {
    let text_a = fs::read_to_string(&request.path_a).map_err(|e| format!("Cannot read file A: {e}"))?;
    let text_b = fs::read_to_string(&request.path_b).map_err(|e| format!("Cannot read file B: {e}"))?;
    let label_a = request.label_a.unwrap_or_else(|| request.path_a.split('/').last().unwrap_or("A").to_string());
    let label_b = request.label_b.unwrap_or_else(|| request.path_b.split('/').last().unwrap_or("B").to_string());

    let lines_a: Vec<&str> = text_a.lines().collect();
    let lines_b: Vec<&str> = text_b.lines().collect();

    // Simple LCS-based diff
    let diff = lcs_diff(&lines_a, &lines_b);
    let added = diff.iter().filter(|d| d.kind == "added").count();
    let removed = diff.iter().filter(|d| d.kind == "removed").count();
    let unchanged = diff.iter().filter(|d| d.kind == "equal").count();

    Ok(CompareResponse { label_a, label_b, diff, added, removed, unchanged })
}

fn lcs_diff(a: &[&str], b: &[&str]) -> Vec<DiffLine> {
    // Simple Myers diff approximation using patience diff approach
    let m = a.len(); let n = b.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in (0..m).rev() {
        for j in (0..n).rev() {
            dp[i][j] = if a[i] == b[j] { dp[i+1][j+1] + 1 } else { dp[i+1][j].max(dp[i][j+1]) };
        }
    }
    let mut result = Vec::new();
    let mut i = 0; let mut j = 0;
    let mut la = 1usize; let mut lb = 1usize;
    while i < m || j < n {
        if i < m && j < n && a[i] == b[j] {
            result.push(DiffLine { kind: "equal".to_string(), line_a: Some(la), line_b: Some(lb), text: a[i].to_string() });
            i += 1; j += 1; la += 1; lb += 1;
        } else if j < n && (i >= m || dp[i][j+1] >= dp[i+1][j]) {
            result.push(DiffLine { kind: "added".to_string(), line_a: None, line_b: Some(lb), text: b[j].to_string() });
            j += 1; lb += 1;
        } else {
            result.push(DiffLine { kind: "removed".to_string(), line_a: Some(la), line_b: None, text: a[i].to_string() });
            i += 1; la += 1;
        }
    }
    result
}
