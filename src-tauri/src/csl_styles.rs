use serde::Serialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct CslStyleInfo {
	pub id: String,
	pub title: String,
	pub filename: String,
	pub path: String,
}

fn extract_csl_title(content: &str) -> Option<String> {
	// CSL XML: look for <title>...</title> inside <info>
	let info_start = content.find("<info")?;
	let info_end = content[info_start..].find("</info>").map(|i| info_start + i)?;
	let info = &content[info_start..info_end];
	let title_start = info.find("<title>")? + "<title>".len();
	let title_end = info[title_start..].find("</title>").map(|i| title_start + i)?;
	Some(info[title_start..title_end].trim().to_string())
}

#[tauri::command]
pub(crate) fn list_installed_csl_styles(csl_dir: Option<String>) -> Result<Vec<CslStyleInfo>, String> {
	let search_dirs: Vec<PathBuf> = if let Some(dir) = csl_dir {
		vec![PathBuf::from(dir)]
	} else {
		// Default Pandoc CSL locations
		let mut dirs = Vec::new();
		if let Some(home) = std::env::var_os("HOME") {
			dirs.push(PathBuf::from(&home).join(".pandoc").join("csl"));
			dirs.push(PathBuf::from(&home).join("Library").join("Application Support").join("pandoc").join("csl"));
		}
		if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
			dirs.push(PathBuf::from(xdg).join("pandoc").join("csl"));
		}
		dirs
	};

	let mut styles = Vec::new();
	for dir in &search_dirs {
		let Ok(entries) = fs::read_dir(dir) else { continue };
		for entry in entries.flatten() {
			let path = entry.path();
			if path.extension().and_then(|e| e.to_str()) != Some("csl") { continue; }
			let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
			let id = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
			let content = fs::read_to_string(&path).unwrap_or_default();
			let title = extract_csl_title(&content).unwrap_or_else(|| id.replace('-', " "));
			styles.push(CslStyleInfo {
				id: id.clone(),
				title,
				filename,
				path: path.to_string_lossy().to_string(),
			});
		}
	}
	styles.sort_by(|a, b| a.title.cmp(&b.title));
	Ok(styles)
}
