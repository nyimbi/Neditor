use super::*;

#[test]
fn approved_documents_require_approval_metadata() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Approved\nstatus: approved\n---\n# Approved\n".to_string(),
        file_path: None,
    });

    assert!(response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("missing approval metadata")));
}

#[test]
fn validation_requires_version_metadata() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Versioned\nstatus: approved\napprovedBy: QA\n---\n# Versioned\n"
            .to_string(),
        file_path: None,
    });

    assert!(response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message == "Missing version metadata."));
}

#[test]
fn validation_rejects_unknown_release_status() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Status\nversion: 1.0.0\nstatus: final\n---\n# Status\n".to_string(),
        file_path: None,
    });

    assert!(response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message == "Invalid document status: final"));
}

#[test]
fn approved_documents_require_approval_timestamp() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Approved\nversion: 1.0.0\nstatus: published\napprovedBy: QA\n---\n# Approved\n".to_string(),
            file_path: None,
        });

    assert!(response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("missing approval metadata")));
}
