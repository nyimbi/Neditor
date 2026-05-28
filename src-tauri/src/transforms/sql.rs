use super::options::TransformExecutionOptions;
use crate::{
    diag, escape_html,
    tables::{parse_delimited_rows, render_delimited_table},
    DocumentDiagnostic,
};
use serde_json::Value;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub(crate) fn render_sql_table(
    query: &str,
    fence_options: &Value,
    options: &TransformExecutionOptions,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let query = query.trim();
    if !read_only_select(query) {
        let message = "SQL transform only allows read-only SELECT or WITH queries.";
        artifact_diags.push(diag(
            "error",
            message,
            None,
            None,
            Some("Use SELECT ... or WITH ... SELECT ...; mutation statements are blocked."),
        ));
        diagnostics.push(diag("error", message, None, None, None));
        return error_block(message);
    }
    let Some(database_path) = sql_database_path(fence_options) else {
        let message = "SQL transform requires a database=\"path/to/file.sqlite\" option.";
        artifact_diags.push(diag(
            "error",
            message,
            None,
            None,
            Some("Keep database paths local to the workspace and pass only read-only SQL."),
        ));
        diagnostics.push(diag("error", message, None, None, None));
        return error_block(message);
    };
    if options.disabled("sql") {
        let message = "SQL transform is disabled in Settings.";
        artifact_diags.push(diag(
            "info",
            message,
            None,
            None,
            Some("Enable the SQL transform when database query rendering is required."),
        ));
        return error_block(message);
    }
    if !options.trusted("sql") {
        let message = "SQL transform requires explicit trust before NEditor runs sqlite3.";
        let suggestion = "Configure and trust the sqlite3 executable in Settings > Transforms.";
        artifact_diags.push(diag("warning", message, None, None, Some(suggestion)));
        diagnostics.push(diag("warning", message, None, None, Some(suggestion)));
        return error_block(message);
    }
    let Some(engine_path) = options.engine_path("sql") else {
        let message = "Configure the sqlite3 executable path before running SQL transforms.";
        artifact_diags.push(diag(
            "warning",
            message,
            None,
            None,
            Some("Choose an absolute sqlite3 path in Settings > Transforms."),
        ));
        return error_block(message);
    };
    let engine_path = PathBuf::from(engine_path);
    if !engine_path.is_absolute() || !engine_path.is_file() {
        let message = "SQL transform engine path must be an absolute sqlite3 executable path.";
        artifact_diags.push(diag("error", message, None, None, None));
        diagnostics.push(diag("error", message, None, None, None));
        return error_block(message);
    }
    if options.document_relative_path_escapes(&database_path) {
        let message =
            format!("SQL database path must stay inside the document folder: {database_path}");
        artifact_diags.push(diag(
            "error",
            message.clone(),
            None,
            None,
            Some("Move the SQLite file under the document folder or select a trusted local database explicitly."),
        ));
        diagnostics.push(diag("error", message.clone(), None, None, None));
        return error_block(&message);
    }
    let database_path = options.resolve_document_path(&database_path);
    if !database_path.is_file() {
        let message = format!("SQL database was not found: {}", database_path.display());
        artifact_diags.push(diag("error", message.clone(), None, None, None));
        diagnostics.push(diag("error", message.clone(), None, None, None));
        return error_block(&message);
    }
    let timeout_ms = options.timeout_ms.unwrap_or(5_000).clamp(1, 30_000);
    let started = Instant::now();
    let mut child = match Command::new(&engine_path)
        .args(["-header", "-csv"])
        .arg(&database_path)
        .arg(query)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            let message = format!("Could not start sqlite3 for SQL transform: {error}");
            artifact_diags.push(diag("error", message.clone(), None, None, None));
            return error_block(&message);
        }
    };
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() < Duration::from_millis(timeout_ms) => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                let message = format!("SQL transform timed out after {timeout_ms}ms.");
                artifact_diags.push(diag("error", message.clone(), None, None, None));
                return error_block(&message);
            }
            Err(error) => {
                let message = format!("Could not poll SQL transform: {error}");
                artifact_diags.push(diag("error", message.clone(), None, None, None));
                return error_block(&message);
            }
        }
    }
    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(error) => {
            let message = format!("Could not read SQL transform output: {error}");
            artifact_diags.push(diag("error", message.clone(), None, None, None));
            return error_block(&message);
        }
    };
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        artifact_diags.push(diag(
            "warning",
            format!("sqlite3: {stderr}"),
            None,
            None,
            Some("Check database path, table names, and SQL syntax."),
        ));
    }
    if !output.status.success() {
        let message = format!("SQL transform failed: {stderr}");
        diagnostics.push(diag("error", message.clone(), None, None, None));
        return error_block(&message);
    }
    let csv = String::from_utf8_lossy(&output.stdout).to_string();
    if parse_delimited_rows(&csv, ',').is_empty() {
        return "<table class=\"transform-table transform-sql\"><tbody><tr><td>No rows returned.</td></tr></tbody></table>".to_string();
    }
    let mut sql_diags = Vec::new();
    let mut html = render_delimited_table(&csv, ',', &mut sql_diags, diagnostics);
    html = html.replacen("transform-table", "transform-table transform-sql", 1);
    artifact_diags.extend(sql_diags);
    artifact_diags.push(diag(
        "info",
        format!("SQL transform returned CSV in {}ms.", started.elapsed().as_millis()),
        None,
        None,
        Some("sqlite3 was invoked directly without a shell and limited to read-only SELECT/WITH queries."),
    ));
    html
}

fn sql_database_path(options: &Value) -> Option<String> {
    ["database", "db", "path", "source"]
        .iter()
        .find_map(|key| options.get(*key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn read_only_select(query: &str) -> bool {
    let normalized = query
        .trim_start_matches('\u{feff}')
        .trim()
        .trim_end_matches(';')
        .trim_start()
        .to_ascii_lowercase();
    if !(normalized.starts_with("select ") || normalized.starts_with("with ")) {
        return false;
    }
    if has_non_trailing_statement_separator(query) {
        return false;
    }
    !contains_blocked_sql_keyword(query)
}

fn has_non_trailing_statement_separator(query: &str) -> bool {
    let chars = query.chars().collect::<Vec<_>>();
    let mut quote: Option<char> = None;
    let mut index = 0usize;
    while index < chars.len() {
        let ch = chars[index];
        if let Some(quote_char) = quote {
            if ch == quote_char {
                if chars.get(index + 1) == Some(&quote_char) {
                    index += 2;
                    continue;
                }
                quote = None;
            }
            index += 1;
            continue;
        }
        if matches!(ch, '\'' | '"' | '`') {
            quote = Some(ch);
        } else if ch == ';' {
            let remainder = chars[index + 1..].iter().collect::<String>();
            if !remainder.trim().trim_matches(';').trim().is_empty() {
                return true;
            }
        }
        index += 1;
    }
    false
}

fn contains_blocked_sql_keyword(query: &str) -> bool {
    let query = sql_without_quoted_segments(query);
    let blocked = [
        "insert", "update", "delete", "drop", "alter", "create", "replace", "attach", "detach",
        "vacuum", "pragma", "reindex",
    ];
    blocked
        .iter()
        .any(|keyword| contains_sql_keyword(&query, keyword))
}

fn sql_without_quoted_segments(query: &str) -> String {
    let chars = query.chars().collect::<Vec<_>>();
    let mut quote: Option<char> = None;
    let mut output = String::with_capacity(query.len());
    let mut index = 0usize;
    while index < chars.len() {
        let ch = chars[index];
        if let Some(quote_char) = quote {
            if ch == quote_char {
                if chars.get(index + 1) == Some(&quote_char) {
                    output.push(' ');
                    output.push(' ');
                    index += 2;
                    continue;
                }
                quote = None;
            }
            output.push(' ');
            index += 1;
            continue;
        }
        if matches!(ch, '\'' | '"' | '`') {
            quote = Some(ch);
            output.push(' ');
        } else {
            output.push(ch.to_ascii_lowercase());
        }
        index += 1;
    }
    output
}

fn contains_sql_keyword(query: &str, keyword: &str) -> bool {
    let mut search_from = 0usize;
    while let Some(offset) = query[search_from..].find(keyword) {
        let start = search_from + offset;
        let end = start + keyword.len();
        let before = query[..start].chars().next_back();
        let after = query[end..].chars().next();
        if !is_sql_identifier_char(before) && !is_sql_identifier_char(after) {
            return true;
        }
        search_from = end;
    }
    false
}

fn is_sql_identifier_char(ch: Option<char>) -> bool {
    ch.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn error_block(message: &str) -> String {
    format!(
        "<section class=\"transform transform-sql transform-error\">{}</section>",
        escape_html(message)
    )
}
