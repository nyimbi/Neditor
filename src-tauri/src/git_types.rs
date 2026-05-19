use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub(crate) struct GitStatus {
    pub(crate) inside_repo: bool,
    pub(crate) branch: Option<String>,
    pub(crate) dirty: bool,
    pub(crate) summary: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GitPathRequest {
    pub(crate) path: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GitCommitRequest {
    pub(crate) path: String,
    pub(crate) message: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GitTagRequest {
    pub(crate) path: String,
    pub(crate) tag: String,
    pub(crate) message: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GitRestoreRequest {
    pub(crate) path: String,
    pub(crate) revision: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct GitHistoryEntry {
    pub(crate) revision: String,
    pub(crate) author: String,
    pub(crate) date: String,
    pub(crate) subject: String,
}
