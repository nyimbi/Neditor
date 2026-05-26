use crate::{
    diagnostics::{diag, with_range},
    path_to_string, DocumentDiagnostic, IncludeEdge,
};
use serde_json::{json, Value};
use serde_yaml::Value as YamlValue;
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

pub(crate) fn parse_front_matter(
    text: &str,
    diagnostics: &mut Vec<DocumentDiagnostic>,
    source_file: Option<String>,
) -> (Value, String, usize) {
    if !text.starts_with("---\n") {
        return (json!({}), text.to_string(), 1);
    }
    let mut lines = text.lines();
    lines.next();
    let mut consumed_lines = 1usize;
    let mut yaml = String::new();
    for line in &mut lines {
        consumed_lines += 1;
        if line.trim() == "---" {
            let body = lines.collect::<Vec<_>>().join("\n");
            let metadata = match parse_yaml_front_matter_value(&yaml)
                .and_then(|metadata| yaml_value_to_json(strip_yaml_value_tags(metadata)))
            {
                Ok(metadata) if metadata.is_object() => metadata,
                Ok(_) => {
                    diagnostics.push(with_range(
                        diag(
                            "error",
                            "YAML front matter must be a mapping.",
                            source_file.clone(),
                            Some(2),
                            Some("Use key-value YAML metadata such as title: Report."),
                        ),
                        1,
                        Some(2),
                        first_front_matter_line_width(&yaml).max(1),
                    ));
                    json!({})
                }
                Err(err) => {
                    diagnostics.push(front_matter_yaml_error(err, source_file.clone()));
                    json!({})
                }
            };
            return (metadata, body, consumed_lines + 1);
        }
        yaml.push_str(line);
        yaml.push('\n');
    }
    diagnostics.push(diag(
        "error",
        "Front matter was opened but not closed.",
        source_file,
        Some(1),
        Some("Add a closing --- marker."),
    ));
    (json!({}), text.to_string(), 1)
}

fn front_matter_yaml_error(
    err: serde_yaml::Error,
    source_file: Option<String>,
) -> DocumentDiagnostic {
    let (line, column) = err
        .location()
        .map(|location| (location.line() + 1, location.column().max(1)))
        .unwrap_or((2, 1));
    with_range(
        diag(
            "error",
            format!("Invalid YAML front matter: {err}"),
            source_file,
            Some(line),
            Some("Fix the YAML syntax between the opening and closing --- markers."),
        ),
        column,
        Some(line),
        column + 1,
    )
}

fn first_front_matter_line_width(yaml: &str) -> usize {
    yaml.lines().next().unwrap_or("").chars().count()
}

fn parse_yaml_front_matter_value(yaml: &str) -> Result<YamlValue, serde_yaml::Error> {
    serde_yaml::from_str::<YamlValue>(yaml).or_else(|err| {
        if yaml_error_is_tag_handle(&err) {
            serde_yaml::from_str::<YamlValue>(&strip_yaml_tag_decorators(yaml))
        } else {
            Err(err)
        }
    })
}

fn yaml_error_is_tag_handle(err: &serde_yaml::Error) -> bool {
    err.to_string().contains("tag")
}

fn strip_yaml_tag_decorators(yaml: &str) -> String {
    yaml.lines()
        .map(strip_yaml_line_tag_decorators)
        .collect::<Vec<_>>()
        .join("\n")
}

fn strip_yaml_line_tag_decorators(line: &str) -> String {
    let chars = line.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0usize;
    let mut quote = None;
    while index < chars.len() {
        let ch = chars[index];
        if let Some(active_quote) = quote {
            output.push(ch);
            if ch == active_quote {
                quote = None;
            }
            index += 1;
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            output.push(ch);
            index += 1;
            continue;
        }
        if ch == '!' && yaml_tag_prefix_allowed(&chars, index) {
            if let Some(end) = yaml_tag_decorator_end(&chars, index) {
                index = end;
                while index < chars.len() && chars[index].is_whitespace() {
                    index += 1;
                }
                continue;
            }
        }
        output.push(ch);
        index += 1;
    }
    output
}

fn yaml_tag_prefix_allowed(chars: &[char], index: usize) -> bool {
    index == 0
        || chars
            .get(index.saturating_sub(1))
            .is_some_and(|ch| ch.is_whitespace() || matches!(ch, '[' | '{' | ',' | ':' | '-'))
}

fn yaml_tag_decorator_end(chars: &[char], start: usize) -> Option<usize> {
    if chars.get(start + 1) == Some(&'<') {
        let mut index = start + 2;
        while index < chars.len() {
            if chars[index] == '>' {
                return Some(index + 1);
            }
            index += 1;
        }
        return None;
    }
    let mut index = start
        + if chars.get(start + 1) == Some(&'!') {
            2
        } else {
            1
        };
    index = consume_yaml_tag_name(chars, index)?;
    if chars.get(index) == Some(&'!') {
        index = consume_yaml_tag_name(chars, index + 1)?;
    }
    if index >= chars.len() || chars[index].is_whitespace() {
        Some(index)
    } else {
        None
    }
}

fn consume_yaml_tag_name(chars: &[char], mut index: usize) -> Option<usize> {
    let start = index;
    while index < chars.len() && is_yaml_tag_name_char(chars[index]) {
        index += 1;
    }
    (index > start).then_some(index)
}

fn is_yaml_tag_name_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | ':' | '/' | '@' | '-')
}

fn strip_yaml_value_tags(value: YamlValue) -> YamlValue {
    match value {
        YamlValue::Tagged(tagged) => strip_yaml_value_tags(tagged.value),
        YamlValue::Sequence(items) => YamlValue::Sequence(
            items
                .into_iter()
                .map(strip_yaml_value_tags)
                .collect::<Vec<_>>(),
        ),
        YamlValue::Mapping(mapping) => {
            let mut normalized = serde_yaml::Mapping::new();
            for (key, value) in mapping {
                normalized.insert(strip_yaml_value_tags(key), strip_yaml_value_tags(value));
            }
            YamlValue::Mapping(normalized)
        }
        other => other,
    }
}

fn yaml_value_to_json(value: YamlValue) -> Result<Value, serde_yaml::Error> {
    serde_yaml::from_value(value)
}

pub(crate) fn merge_project_variables(
    metadata: &mut Value,
    root_path: Option<&Path>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let Some(path) = project_variables_path(root_path) else {
        return;
    };
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => {
            diagnostics.push(diag(
                "warning",
                format!("Unable to read project variables {}: {err}", path.display()),
                Some(path_to_string(&path)),
                None,
                Some("Check permissions or remove the project variables file."),
            ));
            return;
        }
    };
    let mut variables = match parse_yaml_front_matter_value(&text)
        .and_then(|variables| yaml_value_to_json(strip_yaml_value_tags(variables)))
    {
        Ok(value) => value,
        Err(err) => {
            diagnostics.push(diag(
                "error",
                format!("Invalid project variables YAML {}: {err}", path.display()),
                Some(path_to_string(&path)),
                None,
                Some("Fix .neditor/variables.yaml or variables.yml."),
            ));
            return;
        }
    };
    if let Some(inner) = variables.get("variables").cloned() {
        variables = inner;
    }
    let (Some(target), Some(source)) = (metadata.as_object_mut(), variables.as_object()) else {
        diagnostics.push(diag(
            "warning",
            format!(
                "Project variables {} must be a YAML mapping.",
                path.display()
            ),
            Some(path_to_string(&path)),
            None,
            Some("Use key-value YAML such as client: Acme."),
        ));
        return;
    };
    merge_project_variable_defaults(target, source);
}

fn merge_project_variable_defaults(
    target: &mut serde_json::Map<String, Value>,
    source: &serde_json::Map<String, Value>,
) {
    for (key, value) in source {
        match (target.get_mut(key), value) {
            (Some(Value::Object(target_object)), Value::Object(source_object)) => {
                merge_project_variable_defaults(target_object, source_object);
            }
            (Some(_), _) => {}
            (None, value) => {
                target.insert(key.clone(), value.clone());
            }
        }
    }
}

fn project_variables_path(root_path: Option<&Path>) -> Option<PathBuf> {
    let mut dir = root_path
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .or_else(|| std::env::current_dir().ok())?;
    loop {
        for name in ["variables.yaml", "variables.yml"] {
            let candidate = dir.join(".neditor").join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        if !dir.pop() {
            return None;
        }
    }
}

pub(crate) fn render_front_matter_data_sources(
    metadata: &Value,
    root_path: Option<&Path>,
    root_file: &str,
    include_graph: &mut Vec<IncludeEdge>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let mut specs = Vec::new();
    collect_data_source_specs(metadata.get("dataSources"), None, &mut specs);
    collect_data_source_specs(metadata.get("data_sources"), None, &mut specs);
    collect_data_source_specs(metadata.get("csvFiles"), Some("csv"), &mut specs);
    collect_data_source_specs(metadata.get("csv_files"), Some("csv"), &mut specs);
    collect_data_source_specs(metadata.get("tsvFiles"), Some("tsv"), &mut specs);
    collect_data_source_specs(metadata.get("tsv_files"), Some("tsv"), &mut specs);
    collect_data_source_specs(metadata.get("jsonFiles"), Some("json"), &mut specs);
    collect_data_source_specs(metadata.get("json_files"), Some("json"), &mut specs);
    collect_data_source_specs(metadata.get("yamlFiles"), Some("yaml"), &mut specs);
    collect_data_source_specs(metadata.get("yaml_files"), Some("yaml"), &mut specs);
    if specs.is_empty() {
        return String::new();
    }

    let base = root_path
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut rendered = Vec::new();
    for spec in specs {
        if spec.path.trim().is_empty() {
            diagnostics.push(data_source_context_diagnostic(
                &spec,
                None,
                Some(root_file.to_string()),
                "warning",
                "Data source entry is missing a path.",
                None,
                Some("Add a path/file value or remove the empty data source entry."),
            ));
            continue;
        }
        if !data_source_path_is_document_relative(&spec.path) {
            diagnostics.push(data_source_path_diagnostic(root_file, &spec, None));
            continue;
        }
        let kind = spec
            .kind
            .as_deref()
            .or_else(|| data_source_kind_from_path(&spec.path))
            .unwrap_or("csv")
            .to_ascii_lowercase();
        if !matches!(kind.as_str(), "csv" | "tsv" | "json" | "yaml") {
            diagnostics.push(data_source_context_diagnostic(
                &spec,
                None,
                Some(root_file.to_string()),
                "warning",
                format!("Unsupported data source type '{kind}' for {}", spec.path),
                None,
                Some("Use csv, tsv, json, or yaml for local data sources."),
            ));
            continue;
        }
        let path = base.join(&spec.path);
        if data_source_resolves_outside_base(&base, &path) {
            diagnostics.push(data_source_path_diagnostic(root_file, &spec, Some(&path)));
            continue;
        }
        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(err) => {
                diagnostics.push(data_source_context_diagnostic(
                    &spec,
                    Some(&path),
                    Some(path_to_string(&path)),
                    "error",
                    format!("Unable to read data source {}: {err}", path.display()),
                    None,
                    Some("Create the data file or update front matter dataSources/csvFiles."),
                ));
                continue;
            }
        };
        include_graph.push(IncludeEdge {
            parent: root_file.to_string(),
            child: path_to_string(&path),
            depth: 0,
        });
        let title = spec.name.unwrap_or_else(|| {
            path.file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("Data source")
                .to_string()
        });
        rendered.push(format!(
            "## Data Source: {title}\n\n```{kind}\n{}\n```",
            contents.trim_end()
        ));
    }
    rendered.join("\n\n")
}

struct DataSourceSpec {
    name: Option<String>,
    path: String,
    kind: Option<String>,
}

fn collect_data_source_specs(
    value: Option<&Value>,
    default_kind: Option<&str>,
    specs: &mut Vec<DataSourceSpec>,
) {
    match value {
        Some(Value::String(path)) => specs.push(DataSourceSpec {
            name: None,
            path: path.clone(),
            kind: default_kind.map(ToString::to_string),
        }),
        Some(Value::Array(items)) => {
            for item in items {
                collect_data_source_specs(Some(item), default_kind, specs);
            }
        }
        Some(Value::Object(object)) => {
            let name = object
                .get("name")
                .or_else(|| object.get("title"))
                .and_then(Value::as_str)
                .map(ToString::to_string);
            let kind = object
                .get("type")
                .or_else(|| object.get("kind"))
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .or_else(|| default_kind.map(ToString::to_string));
            if let Some(path) = object
                .get("path")
                .or_else(|| object.get("file"))
                .and_then(Value::as_str)
            {
                specs.push(DataSourceSpec {
                    name,
                    path: path.to_string(),
                    kind,
                });
            } else {
                specs.push(DataSourceSpec {
                    name,
                    path: String::new(),
                    kind,
                });
            }
        }
        _ => {}
    }
}

fn data_source_kind_from_path(path: &str) -> Option<&'static str> {
    let path = path.to_ascii_lowercase();
    if path.ends_with(".tsv") {
        Some("tsv")
    } else if path.ends_with(".csv") {
        Some("csv")
    } else if path.ends_with(".json") {
        Some("json")
    } else if path.ends_with(".yaml") || path.ends_with(".yml") {
        Some("yaml")
    } else {
        None
    }
}

fn data_source_path_is_document_relative(path: &str) -> bool {
    let path = Path::new(path);
    !path.is_absolute()
        && path.components().all(|component| {
            !matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
}

fn data_source_resolves_outside_base(base: &Path, path: &Path) -> bool {
    let Ok(base) = fs::canonicalize(base) else {
        return false;
    };
    let Ok(path) = fs::canonicalize(path) else {
        return false;
    };
    !path.starts_with(base)
}

fn data_source_path_diagnostic(
    root_file: &str,
    spec: &DataSourceSpec,
    resolved_path: Option<&Path>,
) -> DocumentDiagnostic {
    data_source_context_diagnostic(
        spec,
        resolved_path,
        Some(root_file.to_string()),
        "error",
        format!(
            "Data source path must stay relative to the document folder: {}",
            spec.path
        ),
        None,
        Some("Use a relative child path such as data/accounts.csv."),
    )
}

fn data_source_context_diagnostic(
    spec: &DataSourceSpec,
    resolved_path: Option<&Path>,
    source_file: Option<String>,
    severity: impl Into<String>,
    message: impl Into<String>,
    line: Option<usize>,
    suggestion: Option<&str>,
) -> DocumentDiagnostic {
    let mut diagnostic = diag(severity, message, source_file, line, suggestion);
    if let Some(name) = spec.name.as_deref().filter(|name| !name.trim().is_empty()) {
        diagnostic.related.push(format!("data_source_name: {name}"));
    }
    if !spec.path.trim().is_empty() {
        diagnostic
            .related
            .push(format!("data_source_path: {}", spec.path));
    }
    if let Some(kind) = spec.kind.as_deref().filter(|kind| !kind.trim().is_empty()) {
        diagnostic.related.push(format!("data_source_type: {kind}"));
    }
    if let Some(path) = resolved_path {
        diagnostic
            .related
            .push(format!("resolved_path: {}", path.display()));
    }
    diagnostic
}

pub(crate) fn strip_front_matter(text: &str) -> String {
    if !text.starts_with("---\n") {
        return text.to_string();
    }
    let mut lines = text.lines();
    lines.next();
    for line in &mut lines {
        if line.trim() == "---" {
            return lines.collect::<Vec<_>>().join("\n");
        }
    }
    text.to_string()
}
