use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct DocumentDiagnostic {
    pub(crate) severity: String,
    pub(crate) message: String,
    pub(crate) source_file: Option<String>,
    pub(crate) line: Option<usize>,
    pub(crate) column: Option<usize>,
    pub(crate) end_line: Option<usize>,
    pub(crate) end_column: Option<usize>,
    pub(crate) suggestion: Option<String>,
    pub(crate) related: Vec<String>,
}
