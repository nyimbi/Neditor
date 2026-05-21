use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub(crate) struct TransformExecutionOptions {
    engine_paths: HashMap<String, String>,
    trusted_engines: HashMap<String, bool>,
    disabled_engines: HashMap<String, bool>,
    input_modes: HashMap<String, String>,
    pub(crate) timeout_ms: Option<u64>,
}

impl TransformExecutionOptions {
    pub(crate) fn from_compile_options(options: Option<&Value>) -> Self {
        let Some(options) = options else {
            return Self::default();
        };
        Self {
            engine_paths: string_map_option(options, "transformEnginePaths"),
            trusted_engines: bool_map_option(options, "trustedTransformEngines"),
            disabled_engines: bool_map_option(options, "disabledTransformEngines"),
            input_modes: string_map_option(options, "transformInputModes"),
            timeout_ms: options.get("transformTimeoutMs").and_then(Value::as_u64),
        }
    }

    pub(crate) fn engine_path(&self, name: &str) -> Option<String> {
        self.engine_paths
            .get(name)
            .or_else(|| {
                if name == "graphviz" {
                    self.engine_paths.get("dot")
                } else {
                    None
                }
            })
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    pub(crate) fn trusted(&self, name: &str) -> bool {
        self.trusted_engines.get(name).copied().unwrap_or(false)
    }

    pub(crate) fn disabled(&self, name: &str) -> bool {
        self.disabled_engines
            .get(name)
            .or_else(|| {
                if name == "graphviz" {
                    self.disabled_engines.get("dot")
                } else {
                    None
                }
            })
            .copied()
            .unwrap_or(false)
    }

    pub(crate) fn input_mode(&self, name: &str) -> Option<String> {
        self.input_modes.get(name).cloned()
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
