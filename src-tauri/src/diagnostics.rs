pub(crate) use crate::diagnostics_types::DocumentDiagnostic;

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
        column: None,
        end_line: None,
        end_column: None,
        suggestion: suggestion.map(ToString::to_string),
        related: Vec::new(),
    }
}

pub(crate) fn with_range(
    mut diagnostic: DocumentDiagnostic,
    column: usize,
    end_line: Option<usize>,
    end_column: usize,
) -> DocumentDiagnostic {
    diagnostic.column = Some(column);
    diagnostic.end_line = end_line.or(diagnostic.line);
    diagnostic.end_column = Some(end_column.max(column));
    diagnostic
}
