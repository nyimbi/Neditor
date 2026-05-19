use crate::diagnostics::DocumentDiagnostic;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub(crate) mod external;
pub(crate) mod qr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct TransformArtifact {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) output_kind: String,
    pub(crate) source_hash: String,
    #[serde(default)]
    pub(crate) source: String,
    #[serde(default = "empty_transform_options")]
    pub(crate) options: Value,
    pub(crate) output_hash: String,
    pub(crate) cache_key: String,
    pub(crate) execution_kind: String,
    pub(crate) engine_version: Option<String>,
    pub(crate) engine_path: Option<String>,
    pub(crate) input_mode: String,
    pub(crate) duration_ms: Option<u64>,
    pub(crate) html: String,
    pub(crate) diagnostics: Vec<DocumentDiagnostic>,
}

fn empty_transform_options() -> Value {
    json!({})
}

pub(crate) fn transform_cache_key(
    name: &str,
    input_mode: &str,
    engine_path: &str,
    source_hash: &str,
) -> String {
    crate::sha256_hex(format!("{name}:{input_mode}:{engine_path}:{source_hash}").as_bytes())
}
