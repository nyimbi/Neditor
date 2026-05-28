use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub(crate) struct TransformExecutionOptions {
    engine_paths: HashMap<String, String>,
    trusted_engines: HashMap<String, bool>,
    disabled_engines: HashMap<String, bool>,
    input_modes: HashMap<String, String>,
    document_dir: Option<PathBuf>,
    pub(crate) timeout_ms: Option<u64>,
}

impl TransformExecutionOptions {
    pub(crate) fn from_compile_options(
        options: Option<&Value>,
        document_path: Option<&Path>,
    ) -> Self {
        let document_dir = document_path
            .and_then(|path| path.parent())
            .map(Path::to_path_buf);
        let Some(options) = options else {
            return Self {
                document_dir,
                ..Self::default()
            };
        };
        Self {
            engine_paths: string_map_option(options, "transformEnginePaths"),
            trusted_engines: bool_map_option(options, "trustedTransformEngines"),
            disabled_engines: bool_map_option(options, "disabledTransformEngines"),
            input_modes: string_map_option(options, "transformInputModes"),
            document_dir,
            timeout_ms: options.get("transformTimeoutMs").and_then(Value::as_u64),
        }
    }

    pub(crate) fn engine_path(&self, name: &str) -> Option<String> {
        option_lookup_keys(name)
            .iter()
            .find_map(|key| self.engine_paths.get(*key))
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    pub(crate) fn trusted(&self, name: &str) -> bool {
        option_lookup_keys(name)
            .iter()
            .find_map(|key| self.trusted_engines.get(*key))
            .copied()
            .unwrap_or(false)
    }

    pub(crate) fn disabled(&self, name: &str) -> bool {
        option_lookup_keys(name)
            .iter()
            .find_map(|key| self.disabled_engines.get(*key))
            .copied()
            .unwrap_or(false)
    }

    pub(crate) fn input_mode(&self, name: &str) -> Option<String> {
        option_lookup_keys(name)
            .iter()
            .find_map(|key| self.input_modes.get(*key))
            .cloned()
    }

    pub(crate) fn resolve_document_path(&self, value: &str) -> PathBuf {
        let path = PathBuf::from(value);
        if path.is_absolute() {
            path
        } else if let Some(document_dir) = &self.document_dir {
            document_dir.join(path)
        } else {
            path
        }
    }

    pub(crate) fn document_relative_path_escapes(&self, value: &str) -> bool {
        let path = PathBuf::from(value);
        let Some(document_dir) = &self.document_dir else {
            return false;
        };
        if path.is_absolute() {
            return false;
        }
        let resolved = document_dir.join(&path);
        match (document_dir.canonicalize(), resolved.canonicalize()) {
            (Ok(base), Ok(target)) => !target.starts_with(base),
            _ => path
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir)),
        }
    }
}

fn option_lookup_keys(name: &str) -> Vec<&str> {
    let aliases = transform_option_aliases(name);
    if aliases.is_empty() {
        vec![name]
    } else {
        aliases.to_vec()
    }
}

fn transform_option_aliases(name: &str) -> &'static [&'static str] {
    match name {
        "dot" | "graphviz" | "graph" => &["dot", "graphviz", "graph"],
        "vega-lite" | "vegalite" => &["vega-lite", "vegalite"],
        "json-schema" | "jsonschema" | "schema" => &["json-schema", "jsonschema", "schema"],
        "yaml" | "yml" => &["yaml", "yml"],
        "plantuml" => &["plantuml"],
        "d2" => &["d2"],
        "pikchr" => &["pikchr"],
        "circo" => &["circo"],
        "neato" => &["neato"],
        "fdp" => &["fdp"],
        "osage" => &["osage"],
        "twopi" => &["twopi"],
        "sql" => &["sql"],
        _ => &[],
    }
}

fn string_map_option(options: &Value, key: &str) -> HashMap<String, String> {
    options
        .get(key)
        .and_then(Value::as_object)
        .map(|fields| {
            fields
                .iter()
                .filter_map(|(name, value)| {
                    value
                        .as_str()
                        .map(|field| (name.clone(), field.to_string()))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn bool_map_option(options: &Value, key: &str) -> HashMap<String, bool> {
    options
        .get(key)
        .and_then(Value::as_object)
        .map(|fields| {
            fields
                .iter()
                .filter_map(|(name, value)| value.as_bool().map(|field| (name.clone(), field)))
                .collect()
        })
        .unwrap_or_default()
}
