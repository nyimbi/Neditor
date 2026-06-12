use serde::{Deserialize, Serialize};
use std::{fs, io::{BufRead, BufReader, Read, Seek, SeekFrom, Write}, path::PathBuf};
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub event: String,
    pub document_path: Option<String>,
    pub document_title: Option<String>,
    pub author: Option<String>,
    pub detail: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecordAuditRequest {
    pub workspace_root: String,
    pub event: String,
    pub document_path: Option<String>,
    pub document_title: Option<String>,
    pub author: Option<String>,
    pub detail: Option<String>,
    pub status: Option<String>,
    /// Maximum file size in bytes. When the file meets or exceeds this threshold
    /// the oldest half of entries is dropped before the new entry is appended.
    /// None means unbounded (matches old behaviour).
    pub max_bytes: Option<u64>,
}

#[tauri::command]
pub(crate) fn record_audit_event(request: RecordAuditRequest) -> Result<(), String> {
    let audit_dir = PathBuf::from(&request.workspace_root).join(".neditor");
    fs::create_dir_all(&audit_dir).map_err(|e| e.to_string())?;
    let audit_path = audit_dir.join("audit.jsonl");

    // Enforce size cap: when max_bytes is set and the file meets/exceeds the
    // threshold, rewrite the file keeping only the newest half of entries.
    if let Some(max_bytes) = request.max_bytes {
        if let Ok(meta) = fs::metadata(&audit_path) {
            if meta.len() >= max_bytes {
                let file = fs::File::open(&audit_path).map_err(|e| e.to_string())?;
                let reader = BufReader::new(file);
                let lines: Vec<String> = reader
                    .lines()
                    .filter_map(|l| l.ok())
                    .filter(|l| !l.is_empty())
                    .collect();
                // Keep the newest half so the file doesn't oscillate on every write.
                let keep_from = lines.len() / 2;
                let trimmed = lines[keep_from..].join("\n");
                fs::write(&audit_path, format!("{trimmed}\n")).map_err(|e| e.to_string())?;
            }
        }
    }

    let entry = AuditEntry {
        timestamp: Utc::now().to_rfc3339(),
        event: request.event,
        document_path: request.document_path,
        document_title: request.document_title,
        author: request.author,
        detail: request.detail,
        status: request.status,
    };
    let line = serde_json::to_string(&entry).map_err(|e| e.to_string())?;
    let mut file = fs::OpenOptions::new().create(true).append(true).open(&audit_path).map_err(|e| e.to_string())?;
    writeln!(file, "{line}").map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub(crate) fn read_audit_log(workspace_root: String, limit: usize) -> Result<Vec<AuditEntry>, String> {
    let audit_path = PathBuf::from(&workspace_root).join(".neditor").join("audit.jsonl");
    if !audit_path.exists() { return Ok(Vec::new()); }

    let limit = limit.min(1000);

    // Guard: refuse to scan files larger than 10 MB to prevent OOM.
    const MAX_READ_BYTES: u64 = 10 * 1024 * 1024;
    // Tail window: read at most this many bytes from the end of the file.
    // 1000 entries × ~500 bytes each = ~500 KB; 1 MB is a generous buffer.
    const TAIL_BYTES: u64 = 1024 * 1024;

    let meta = fs::metadata(&audit_path).map_err(|e| e.to_string())?;
    let file_len = meta.len();

    if file_len > MAX_READ_BYTES {
        return Err(format!(
            "audit.jsonl is {file_len} bytes which exceeds the {MAX_READ_BYTES}-byte read limit. \
             Rotate or trim the file, or lower auditMaxBytes in settings."
        ));
    }

    let mut file = fs::File::open(&audit_path).map_err(|e| e.to_string())?;

    // Seek to a position near the tail so we only read the last TAIL_BYTES.
    // If the file is smaller than TAIL_BYTES, read from the start; the first
    // line may be a partial line so we skip it and let the filter drop it.
    let seek_pos = file_len.saturating_sub(TAIL_BYTES);
    file.seek(SeekFrom::Start(seek_pos)).map_err(|e| e.to_string())?;

    let mut tail_buf = Vec::with_capacity((file_len - seek_pos) as usize);
    file.read_to_end(&mut tail_buf).map_err(|e| e.to_string())?;

    // When we sought into the middle of the file, the first "line" is likely a
    // partial fragment — drop it so we only parse complete JSON objects.
    let tail_str = String::from_utf8_lossy(&tail_buf);
    let mut lines_iter = tail_str.lines();
    if seek_pos > 0 {
        lines_iter.next(); // discard the potentially-truncated first line
    }

    let entries: Vec<AuditEntry> = lines_iter
        .filter(|l| !l.is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();

    Ok(entries.into_iter().rev().take(limit).collect())
}
