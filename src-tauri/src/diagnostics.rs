use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct DocumentDiagnostic {
    pub(crate) severity: String,
    pub(crate) message: String,
    pub(crate) source_file: Option<String>,
    pub(crate) line: Option<usize>,
    pub(crate) suggestion: Option<String>,
    pub(crate) related: Vec<String>,
}

pub(crate) fn diag(
    severity: impl Into<String>,
    message: impl Into<String>,
    source_file: Option<String>,
    line: Option<usize>,
    suggestion: Option<&str>,
) -> DocumentDiagnostic {
    DocumentDiagnostic {
        severity: severity.into(),
        message: message.into(),
        source_file,
        line,
        suggestion: suggestion.map(ToString::to_string),
        related: Vec::new(),
    }
}
