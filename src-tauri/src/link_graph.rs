use serde::Serialize;
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct GraphNode {
	pub id: String,
	pub title: String,
	pub path: String,
	pub link_count: usize,
	pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphEdge {
	pub source: String,
	pub target: String,
	pub link_text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceLinkGraph {
	pub nodes: Vec<GraphNode>,
	pub edges: Vec<GraphEdge>,
}

fn extract_title_and_tags(content: &str) -> (String, Vec<String>) {
	let mut title = String::new();
	let mut tags: Vec<String> = Vec::new();
	let mut in_front_matter = false;
	let mut front_matter_done = false;
	let mut fm_line = 0usize;

	for line in content.lines() {
		if fm_line == 0 && line.trim() == "---" {
			in_front_matter = true;
			fm_line += 1;
			continue;
		}
		if in_front_matter {
			if line.trim() == "---" || line.trim() == "..." {
				in_front_matter = false;
				front_matter_done = true;
				continue;
			}
			if line.starts_with("title:") {
				title = line["title:".len()..].trim().trim_matches('"').trim_matches('\'').to_string();
			}
			if line.starts_with("tags:") {
				let rest = line["tags:".len()..].trim();
				// Inline array: tags: [a, b, c]
				if rest.starts_with('[') {
					let inner = rest.trim_matches(|c| c == '[' || c == ']');
					for t in inner.split(',') {
						let t = t.trim().trim_matches('"').trim_matches('\'');
						if !t.is_empty() { tags.push(t.to_string()); }
					}
				}
			}
			// YAML list item under tags:
			if line.starts_with("  - ") && !tags.is_empty() {
				let t = line[4..].trim().trim_matches('"').trim_matches('\'').to_string();
				if !t.is_empty() { tags.push(t); }
			}
			fm_line += 1;
			continue;
		}
		// Use first H1 as fallback title
		if front_matter_done && title.is_empty() && line.starts_with("# ") {
			title = line[2..].trim().to_string();
		}
	}
	if title.is_empty() && !front_matter_done {
		// No front matter, use first H1
		for line in content.lines() {
			if line.starts_with("# ") {
				title = line[2..].trim().to_string();
				break;
			}
		}
	}
	(title, tags)
}

fn extract_wiki_links(content: &str) -> Vec<String> {
	let mut links = Vec::new();
	let mut i = 0usize;
	let bytes = content.as_bytes();
	while i + 1 < bytes.len() {
		if bytes[i] == b'[' && bytes[i + 1] == b'[' {
			i += 2;
			let start = i;
			while i + 1 < bytes.len() && !(bytes[i] == b']' && bytes[i + 1] == b']') {
				i += 1;
			}
			if i + 1 < bytes.len() {
				if let Ok(link) = std::str::from_utf8(&bytes[start..i]) {
					let target = link.split('|').next().unwrap_or(link);
					let target = target.split('#').next().unwrap_or(target);
					let target = target.trim();
					if !target.is_empty() {
						links.push(target.to_string());
					}
				}
				i += 2; // skip ]]
			}
		} else {
			i += 1;
		}
	}
	links
}

fn scan_workspace(root: &PathBuf) -> HashMap<String, (String, String, Vec<String>, Vec<String>)> {
	// key: relative path → (title, abs_path, tags, wiki_links)
	let mut map = HashMap::new();
	scan_dir(root, root, &mut map);
	map
}

fn scan_dir(
	root: &PathBuf,
	dir: &PathBuf,
	map: &mut HashMap<String, (String, String, Vec<String>, Vec<String>)>,
) {
	let Ok(entries) = fs::read_dir(dir) else { return };
	for entry in entries.flatten() {
		let path = entry.path();
		let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
		if name.starts_with('.') || name == "node_modules" || name == "target" { continue; }
		if path.symlink_metadata().map(|m| m.file_type().is_symlink()).unwrap_or(false) { continue; }
		if path.is_dir() {
			scan_dir(root, &path, map);
			continue;
		}
		if path.extension().and_then(|e| e.to_str()) != Some("md") { continue; }
		let Ok(content) = fs::read_to_string(&path) else { continue };
		let rel = path.strip_prefix(root)
			.map(|p| p.to_string_lossy().to_string())
			.unwrap_or_default();
		let abs = path.to_string_lossy().to_string();
		let (title, tags) = extract_title_and_tags(&content);
		let links = extract_wiki_links(&content);
		map.insert(rel, (title, abs, tags, links));
	}
}

#[tauri::command]
pub(crate) fn build_workspace_link_graph(workspace_root: String) -> Result<WorkspaceLinkGraph, String> {
	let root = PathBuf::from(&workspace_root);
	if !root.exists() {
		return Err(format!("Workspace root does not exist: {workspace_root}"));
	}

	let file_map = scan_workspace(&root);

	// Build stem → rel_path index for link resolution
	let stem_index: HashMap<String, String> = file_map
		.keys()
		.map(|rel| {
			let stem = PathBuf::from(rel)
				.file_stem()
				.and_then(|s| s.to_str())
				.unwrap_or("")
				.to_ascii_lowercase();
			(stem, rel.clone())
		})
		.collect();

	let mut nodes = Vec::new();
	let mut edges = Vec::new();
	let mut link_counts: HashMap<String, usize> = HashMap::new();

	// First pass: collect edges
	for (rel, (_title, _abs, _tags, links)) in &file_map {
		for link_text in links {
			let target_stem = link_text.to_ascii_lowercase();
			if let Some(target_rel) = stem_index.get(&target_stem) {
				if target_rel != rel {
					edges.push(GraphEdge {
						source: rel.clone(),
						target: target_rel.clone(),
						link_text: link_text.clone(),
					});
					*link_counts.entry(rel.clone()).or_insert(0) += 1;
					*link_counts.entry(target_rel.clone()).or_insert(0) += 1;
				}
			}
		}
	}

	// Second pass: build nodes
	for (rel, (title, abs, tags, _)) in &file_map {
		let display_title = if title.is_empty() {
			PathBuf::from(rel).file_stem().and_then(|s| s.to_str()).unwrap_or(rel).to_string()
		} else {
			title.clone()
		};
		nodes.push(GraphNode {
			id: rel.clone(),
			title: display_title,
			path: abs.clone(),
			link_count: link_counts.get(rel).copied().unwrap_or(0),
			tags: tags.clone(),
		});
	}

	// Sort nodes by link count descending for visual priority
	nodes.sort_by(|a, b| b.link_count.cmp(&a.link_count).then(a.title.cmp(&b.title)));

	Ok(WorkspaceLinkGraph { nodes, edges })
}
