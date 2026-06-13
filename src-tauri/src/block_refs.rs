use serde::Serialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct BlockRefResult {
	pub content: String,
	pub source_path: String,
	pub heading: String,
	pub found: bool,
}

fn slug(text: &str) -> String {
	text.chars()
		.map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
		.collect::<String>()
		.split('-')
		.filter(|s| !s.is_empty())
		.collect::<Vec<_>>()
		.join("-")
}

fn heading_level(line: &str) -> Option<usize> {
	let trimmed = line.trim_start_matches('#');
	let level = line.len() - trimmed.len();
	if level >= 1 && level <= 6 && trimmed.starts_with(' ') {
		Some(level)
	} else {
		None
	}
}

#[tauri::command]
pub(crate) fn resolve_block_reference(
	workspace_root: String,
	ref_path: String,
	heading_id: String,
) -> Result<BlockRefResult, String> {
	let root = PathBuf::from(&workspace_root);

	// Reject absolute paths — only workspace-relative paths allowed.
	// Covers POSIX (/), tilde (~), Windows drive letters (C:), and UNC (\\).
	let is_absolute = ref_path.starts_with('/')
		|| ref_path.starts_with('~')
		|| ref_path.starts_with("\\\\")
		|| (ref_path.len() >= 3 && ref_path.chars().nth(1) == Some(':'));
	if is_absolute {
		return Err("Block reference path must be relative to the workspace root".to_string());
	}
	let file_path = root.join(&ref_path);

	// Path containment guard
	let canonical = file_path.canonicalize().map_err(|e| format!("Cannot resolve path: {e}"))?;
	let canonical_root = root.canonicalize().map_err(|e| format!("Cannot resolve root: {e}"))?;
	if !canonical.starts_with(&canonical_root) {
		return Err("Block reference path escapes workspace root".to_string());
	}

	let content = fs::read_to_string(&canonical).map_err(|e| format!("Cannot read file: {e}"))?;
	let target_slug = slug(&heading_id);

	let lines: Vec<&str> = content.lines().collect();
	let mut found_line: Option<usize> = None;
	let mut found_level: usize = 1;
	let mut found_heading = String::new();

	// Find the target heading by slug match
	for (i, line) in lines.iter().enumerate() {
		if let Some(level) = heading_level(line) {
			let heading_text = line.trim_start_matches('#').trim();
			let heading_slug = slug(heading_text);
			if heading_slug == target_slug || heading_text.to_ascii_lowercase() == heading_id.to_ascii_lowercase() {
				found_line = Some(i);
				found_level = level;
				found_heading = heading_text.to_string();
				break;
			}
		}
	}

	let Some(start) = found_line else {
		return Ok(BlockRefResult {
			content: format!("> *Block reference not found: `{ref_path}#{heading_id}`*"),
			source_path: canonical.to_string_lossy().to_string(),
			heading: heading_id.clone(),
			found: false,
		});
	};

	// Collect content until next heading of same or higher level (cap at 1000 lines)
	let mut block_lines = vec![lines[start]];
	for line in lines[start + 1..].iter().take(999) {
		if let Some(level) = heading_level(line) {
			if level <= found_level { break; }
		}
		block_lines.push(line);
	}

	// Remove trailing blank lines
	while block_lines.last().map(|l| l.trim().is_empty()).unwrap_or(false) {
		block_lines.pop();
	}

	Ok(BlockRefResult {
		content: block_lines.join("\n"),
		source_path: canonical.to_string_lossy().to_string(),
		heading: found_heading,
		found: true,
	})
}
