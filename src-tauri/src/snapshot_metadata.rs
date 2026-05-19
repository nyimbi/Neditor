use crate::sha256_hex;
use serde_json::Value;

pub(crate) struct SnapshotDocumentMetadata {
    pub(crate) document_version: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) author: Option<String>,
    pub(crate) include_graph_hash: String,
}

pub(crate) fn snapshot_document_metadata(text: &str) -> SnapshotDocumentMetadata {
    let metadata = snapshot_front_matter(text);
    SnapshotDocumentMetadata {
        document_version: metadata
            .as_ref()
            .and_then(|value| metadata_string(value, "version")),
        status: metadata
            .as_ref()
            .and_then(|value| metadata_string(value, "status")),
        author: metadata
            .as_ref()
            .and_then(|value| metadata_string(value, "author")),
        include_graph_hash: include_graph_hash(text),
    }
}

fn snapshot_front_matter(text: &str) -> Option<Value> {
    if !text.starts_with("---\n") {
        return None;
    }
    let lines = text.lines().collect::<Vec<_>>();
    let end_index = lines
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(index, line)| (line.trim() == "---").then_some(index))?;
    let yaml = lines[1..end_index].join("\n");
    serde_yaml::from_str::<Value>(&yaml).ok()
}

fn metadata_string(metadata: &Value, key: &str) -> Option<String> {
    metadata.get(key).and_then(|value| match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    })
}

fn include_graph_hash(text: &str) -> String {
    let mut includes = text
        .lines()
        .filter_map(snapshot_include_target)
        .collect::<Vec<_>>();
    includes.sort();
    sha256_hex(includes.join("\n").as_bytes())
}

fn snapshot_include_target(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if let Some(path) = trimmed.strip_prefix("!include ") {
        return Some(path.trim().to_string());
    }
    if let Some(path) = trimmed
        .strip_prefix("{{include ")
        .and_then(|value| value.strip_suffix("}}"))
    {
        return Some(path.trim().to_string());
    }
    if let Some(path) = trimmed
        .strip_prefix("<!-- include:")
        .and_then(|value| value.strip_suffix("-->"))
    {
        return Some(path.trim().to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_metadata_captures_release_fields_and_include_hash() {
        let text = "---\ntitle: Report\nversion: 2.0.0\nstatus: approved\nauthor: Strategy Team\n---\n# Report\n!include chapters/a.md\n{{include chapters/b.md}}\n<!-- include: chapters/c.md -->\n";
        let metadata = snapshot_document_metadata(text);

        assert_eq!(metadata.document_version.as_deref(), Some("2.0.0"));
        assert_eq!(metadata.status.as_deref(), Some("approved"));
        assert_eq!(metadata.author.as_deref(), Some("Strategy Team"));
        assert_ne!(metadata.include_graph_hash, sha256_hex("".as_bytes()));
    }
}
