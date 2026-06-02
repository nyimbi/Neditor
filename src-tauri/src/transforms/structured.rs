use crate::{diag, escape_html, DocumentDiagnostic};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet, HashSet};

pub(crate) fn render_decision_table_html(
    body: &str,
    _artifact_diags: &mut Vec<DocumentDiagnostic>,
    _diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let mut headers: Vec<String> = Vec::new();
    let mut rows: Vec<Vec<String>> = Vec::new();
    for line in body.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let cols: Vec<&str> = line.split('|').map(str::trim).filter(|c| !c.is_empty()).collect();
        if headers.is_empty() {
            headers = cols.iter().map(|c| escape_html(c)).collect();
        } else {
            rows.push(cols.iter().map(|c| escape_html(c)).collect());
        }
    }
    if headers.is_empty() {
        return "<section class=\"transform transform-decision-table\"><p>No decision table data.</p></section>".to_string();
    }
    let thead = format!("<tr>{}</tr>", headers.iter().map(|h| format!("<th>{h}</th>")).collect::<String>());
    let tbody = rows.iter().map(|row| {
        let cells = row.iter().enumerate().map(|(i, cell)| {
            let class = if i == 0 { " class=\"decision-condition\"" } else { " class=\"decision-action\"" };
            format!("<td{class}>{cell}</td>")
        }).collect::<String>();
        format!("<tr>{cells}</tr>")
    }).collect::<String>();
    format!("<section class=\"transform transform-decision-table\"><h3>Decision Table</h3><table><thead>{thead}</thead><tbody>{tbody}</tbody></table></section>")
}

pub(crate) fn render_toml_html(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    match toml::from_str::<toml::Value>(body) {
        Ok(value) => {
            let json_value: Value = serde_json::to_value(&value).unwrap_or(Value::Null);
            if let Some(table) = render_structured_table("toml", &json_value) {
                table
            } else {
                format!(
                    "<section class=\"transform transform-toml structured-tree\">{}</section>",
                    render_structured_tree("root", &json_value)
                )
            }
        }
        Err(error) => {
            let diagnostic = diag(
                "error",
                format!("Invalid TOML transform input: {error}"),
                None,
                None,
                Some("Check the TOML syntax."),
            );
            diagnostics.push(diagnostic.clone());
            artifact_diags.push(diagnostic);
            format!(
                "<pre class=\"transform transform-toml transform-error\">{}</pre>",
                escape_html(body)
            )
        }
    }
}

pub(crate) fn render_structured_data_html(
    format: &str,
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let parsed = if format == "json" {
        serde_json::from_str::<Value>(body).map_err(|err| err.to_string())
    } else {
        serde_yaml::from_str::<Value>(body).map_err(|err| err.to_string())
    };
    match parsed {
        Ok(value) => {
            if let Some(table) = render_structured_table(format, &value) {
                table
            } else {
                format!(
                    "<section class=\"transform transform-{format} structured-tree\">{}</section>",
                    render_structured_tree("root", &value)
                )
            }
        }
        Err(error) => {
            let diagnostic = diag(
                "error",
                format!(
                    "Invalid {} transform input: {error}",
                    format.to_ascii_uppercase()
                ),
                None,
                None,
                Some("Check the structured data syntax."),
            );
            diagnostics.push(diagnostic.clone());
            artifact_diags.push(diagnostic);
            format!(
                "<pre class=\"transform transform-{format} transform-error\">{}</pre>",
                escape_html(body)
            )
        }
    }
}

pub(crate) fn render_openapi_html(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let value = match parse_json_or_yaml(body) {
        Ok(value) => value,
        Err(err) => {
            let diagnostic = diag(
                "error",
                format!("Invalid OpenAPI document: {err}"),
                None,
                None,
                Some("Provide valid JSON or YAML OpenAPI content."),
            );
            artifact_diags.push(diagnostic.clone());
            diagnostics.push(diagnostic);
            return "<section class=\"transform transform-error\">Invalid OpenAPI document</section>"
                .to_string();
        }
    };
    let info = value.get("info").and_then(Value::as_object);
    let title = info
        .and_then(|info| info.get("title"))
        .and_then(Value::as_str)
        .unwrap_or("API reference");
    let version = info
        .and_then(|info| info.get("version"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let mut html = format!(
        "<section class=\"transform transform-openapi api-reference\"><h3>{}</h3>",
        escape_html(title)
    );
    if !version.is_empty() {
        html.push_str(&format!(
            "<p class=\"api-version\">Version {}</p>",
            escape_html(version)
        ));
    }
    html.push_str(&render_openapi_metadata(info, &value));
    html.push_str(&render_openapi_servers(&value));
    html.push_str(
        "<table class=\"transform-table openapi\"><thead><tr><th>Method</th><th>Path</th><th>Operation</th><th>Security</th><th>Parameters</th><th>Request body</th><th>Responses</th></tr></thead><tbody>",
    );
    if let Some(paths) = value.get("paths").and_then(Value::as_object) {
        for (path, path_item) in paths {
            let path_parameters = path_item
                .get("parameters")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            if let Some(methods) = path_item.as_object() {
                for (method, operation) in methods
                    .iter()
                    .filter(|(method, _)| is_openapi_method(method.as_str()))
                {
                    let summary = operation
                        .get("summary")
                        .or_else(|| operation.get("description"))
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let operation_id = operation
                        .get("operationId")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let operation_label =
                        operation_label(summary, operation_id, operation, path_item);
                    let mut parameters = path_parameters.iter().collect::<Vec<_>>();
                    if let Some(operation_parameters) =
                        operation.get("parameters").and_then(Value::as_array)
                    {
                        parameters.extend(operation_parameters.iter());
                    }
                    html.push_str(&format!(
                        "<tr><td><code>{}</code></td><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                        escape_html(&method.to_ascii_uppercase()),
                        escape_html(path),
                        operation_label,
                        render_openapi_security(
                            operation.get("security").or_else(|| value.get("security"))
                        ),
                        render_openapi_parameters(&parameters),
                        render_openapi_request_body(operation.get("requestBody")),
                        render_openapi_responses(operation.get("responses"))
                    ));
                }
            }
        }
    }
    html.push_str(&render_openapi_webhook_rows(&value));
    html.push_str("</tbody></table>");
    html.push_str(&render_openapi_security_schemes(&value));
    html.push_str(&render_openapi_reusable_components(&value));
    html.push_str(&render_openapi_components(&value));
    html.push_str("</section>");
    html
}

pub(crate) fn render_json_schema_html(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let value = match parse_json_or_yaml(body) {
        Ok(value) => value,
        Err(err) => {
            let diagnostic = diag(
                "error",
                format!("Invalid JSON Schema document: {err}"),
                None,
                None,
                Some("Provide valid JSON or YAML JSON Schema content."),
            );
            artifact_diags.push(diagnostic.clone());
            diagnostics.push(diagnostic);
            return "<section class=\"transform transform-error\">Invalid JSON Schema document</section>"
                .to_string();
        }
    };
    let mut rows = Vec::new();
    collect_schema_rows("", &value, false, &mut rows);
    let title = value
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("JSON Schema");
    let description = value
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("");
    let mut html = format!(
        "<section class=\"transform transform-json-schema schema-reference\"><h3>{}</h3>",
        escape_html(title)
    );
    if !description.is_empty() {
        html.push_str(&format!(
            "<p class=\"schema-description\">{}</p>",
            escape_html(description)
        ));
    }
    html.push_str(&render_json_schema_metadata(&value));
    html.push_str(
        "<table class=\"transform-table json-schema\"><thead><tr><th>Field</th><th>Type</th><th>Required</th><th>Description</th><th>Constraints</th></tr></thead><tbody>",
    );
    for row in rows {
        html.push_str(&format!(
            "<tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            escape_html(&row.field),
            escape_html(&row.kind),
            if row.required { "yes" } else { "no" },
            escape_html(&row.description),
            escape_html(&row.constraints)
        ));
    }
    html.push_str("</tbody></table>");
    html.push_str("</section>");
    html
}

fn render_json_schema_metadata(value: &Value) -> String {
    let mut items = Vec::new();
    for key in [
        "$schema",
        "$id",
        "$anchor",
        "$dynamicAnchor",
        "$recursiveAnchor",
    ] {
        if let Some(summary) = value.get(key).map(structured_value_summary) {
            items.push(format!(
                "<li><code>{}</code>: {}</li>",
                escape_html(key),
                escape_html(&summary)
            ));
        }
    }
    if let Some(vocabulary) = value.get("$vocabulary").and_then(Value::as_object) {
        let summary = vocabulary
            .iter()
            .map(|(name, required)| {
                format!(
                    "{}={}",
                    name,
                    required
                        .as_bool()
                        .map(|value| if value { "required" } else { "optional" })
                        .unwrap_or("declared")
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        if !summary.is_empty() {
            items.push(format!(
                "<li><code>$vocabulary</code>: {}</li>",
                escape_html(&summary)
            ));
        }
    }
    if items.is_empty() {
        String::new()
    } else {
        format!(
            "<section class=\"schema-metadata\"><h4>Schema metadata</h4><ul>{}</ul></section>",
            items.join("")
        )
    }
}

struct SchemaRow {
    field: String,
    kind: String,
    required: bool,
    description: String,
    constraints: String,
}

fn render_openapi_servers(value: &Value) -> String {
    let Some(servers) = value.get("servers").and_then(Value::as_array) else {
        return String::new();
    };
    render_openapi_server_list(servers, "api-servers")
}

fn render_openapi_metadata(info: Option<&Map<String, Value>>, value: &Value) -> String {
    let mut items = Vec::new();
    if let Some(terms) = info
        .and_then(|info| info.get("termsOfService"))
        .and_then(Value::as_str)
        .filter(|terms| !terms.trim().is_empty())
    {
        items.push(format!(
            "<li><strong>Terms:</strong> <code>{}</code></li>",
            escape_html(terms)
        ));
    }
    if let Some(contact) = info
        .and_then(|info| info.get("contact"))
        .and_then(Value::as_object)
        .map(openapi_contact_summary)
        .filter(|summary| !summary.is_empty())
    {
        items.push(format!("<li><strong>Contact:</strong> {contact}</li>"));
    }
    if let Some(license) = info
        .and_then(|info| info.get("license"))
        .and_then(Value::as_object)
        .map(openapi_license_summary)
        .filter(|summary| !summary.is_empty())
    {
        items.push(format!("<li><strong>License:</strong> {license}</li>"));
    }
    if let Some(external_docs) = openapi_external_docs_summary(value.get("externalDocs")) {
        items.push(format!(
            "<li><strong>External docs:</strong> {external_docs}</li>"
        ));
    }
    if let Some(tags) = openapi_tags_summary(value.get("tags")) {
        items.push(format!("<li><strong>Tags:</strong> {tags}</li>"));
    }
    if items.is_empty() {
        String::new()
    } else {
        format!(
            "<section class=\"api-metadata\"><h4>API metadata</h4><ul>{}</ul></section>",
            items.join("")
        )
    }
}

fn openapi_contact_summary(contact: &Map<String, Value>) -> String {
    [
        contact
            .get("name")
            .and_then(Value::as_str)
            .map(escape_html)
            .unwrap_or_default(),
        contact
            .get("email")
            .and_then(Value::as_str)
            .map(|email| format!("email: {}", escape_html(email)))
            .unwrap_or_default(),
        contact
            .get("url")
            .and_then(Value::as_str)
            .map(|url| format!("url: <code>{}</code>", escape_html(url)))
            .unwrap_or_default(),
    ]
    .into_iter()
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join(" ")
}

fn openapi_license_summary(license: &Map<String, Value>) -> String {
    [
        license
            .get("name")
            .and_then(Value::as_str)
            .map(escape_html)
            .unwrap_or_default(),
        license
            .get("identifier")
            .and_then(Value::as_str)
            .map(|identifier| format!("identifier: {}", escape_html(identifier)))
            .unwrap_or_default(),
        license
            .get("url")
            .and_then(Value::as_str)
            .map(|url| format!("url: <code>{}</code>", escape_html(url)))
            .unwrap_or_default(),
    ]
    .into_iter()
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join(" ")
}

fn openapi_external_docs_summary(value: Option<&Value>) -> Option<String> {
    let docs = value?.as_object()?;
    let url = docs.get("url").and_then(Value::as_str).unwrap_or("");
    let description = docs
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("External docs");
    if url.trim().is_empty() {
        return None;
    }
    Some(format!(
        "{} <code>{}</code>",
        escape_html(description),
        escape_html(url)
    ))
}

fn openapi_tags_summary(value: Option<&Value>) -> Option<String> {
    let tags = value?
        .as_array()?
        .iter()
        .filter_map(|tag| {
            let tag = tag.as_object()?;
            let name = tag.get("name").and_then(Value::as_str)?;
            let description = tag.get("description").and_then(Value::as_str).unwrap_or("");
            let external_docs = openapi_external_docs_summary(tag.get("externalDocs"));
            let mut parts = vec![escape_html(name)];
            if !description.trim().is_empty() {
                parts.push(format!("- {}", escape_html(description)));
            }
            if let Some(external_docs) = external_docs {
                parts.push(format!("({external_docs})"));
            }
            Some(parts.join(" "))
        })
        .collect::<Vec<_>>();
    (!tags.is_empty()).then(|| tags.join("; "))
}

fn render_openapi_server_list(servers: &[Value], class_name: &str) -> String {
    if servers.is_empty() {
        return String::new();
    }
    let items = servers
        .iter()
        .filter_map(openapi_server_list_item)
        .collect::<Vec<_>>()
        .join("");
    if items.is_empty() {
        String::new()
    } else {
        format!("<ul class=\"{class_name}\">{items}</ul>")
    }
}

fn openapi_server_list_item(server: &Value) -> Option<String> {
    let summary = openapi_server_summary(server)?;
    Some(format!("<li>{summary}</li>"))
}

fn openapi_server_summary(server: &Value) -> Option<String> {
    let url = server.get("url").and_then(Value::as_str)?;
    let description = server
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("");
    let variables = openapi_server_variables_summary(server);
    let mut parts = vec![format!("<code>{}</code>", escape_html(url))];
    if !description.is_empty() {
        parts.push(escape_html(description));
    }
    if !variables.is_empty() {
        parts.push(escape_html(&variables));
    }
    Some(parts.join(" - "))
}

fn openapi_server_summary_text(server: &Value) -> Option<String> {
    let url = server.get("url").and_then(Value::as_str)?;
    let description = server
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("");
    let variables = openapi_server_variables_summary(server);
    let mut parts = vec![url.to_string()];
    if !description.is_empty() {
        parts.push(description.to_string());
    }
    if !variables.is_empty() {
        parts.push(variables);
    }
    Some(parts.join(" - "))
}

fn openapi_server_variables_summary(server: &Value) -> String {
    let Some(variables) = server.get("variables").and_then(Value::as_object) else {
        return String::new();
    };
    let items = variables
        .iter()
        .map(|(name, variable)| {
            let default = variable
                .get("default")
                .map(structured_value_summary)
                .unwrap_or_default();
            let enum_values = variable
                .get("enum")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .map(structured_value_summary)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            let description = variable
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("");
            let mut parts = vec![name.to_string()];
            if !default.is_empty() {
                parts.push(format!("default {default}"));
            }
            if !enum_values.is_empty() {
                parts.push(format!("enum {enum_values}"));
            }
            let summary = parts.join(" ");
            if description.is_empty() {
                summary
            } else {
                format!("{summary} - {description}")
            }
        })
        .collect::<Vec<_>>()
        .join("; ");
    if items.is_empty() {
        String::new()
    } else {
        format!("variables: {items}")
    }
}

fn render_openapi_operation_servers(operation: &Value, path_item: &Value) -> String {
    let servers = operation
        .get("servers")
        .or_else(|| path_item.get("servers"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let summaries = servers
        .iter()
        .filter_map(openapi_server_summary_text)
        .collect::<Vec<_>>()
        .join("; ");
    if summaries.is_empty() {
        String::new()
    } else {
        format!("<small>servers: {}</small>", escape_html(&summaries))
    }
}

fn is_openapi_method(method: &str) -> bool {
    matches!(
        method,
        "get" | "put" | "post" | "delete" | "options" | "head" | "patch" | "trace"
    )
}

fn operation_label(
    summary: &str,
    operation_id: &str,
    operation: &Value,
    path_item: &Value,
) -> String {
    let tags = operation
        .get("tags")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(escape_html)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let mut parts = Vec::new();
    if !summary.is_empty() {
        parts.push(escape_html(summary));
    }
    if !operation_id.is_empty() {
        parts.push(format!("<code>{}</code>", escape_html(operation_id)));
    }
    if operation
        .get("deprecated")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        parts.push("<small>deprecated</small>".to_string());
    }
    if !tags.is_empty() {
        parts.push(format!("<small>{tags}</small>"));
    }
    if let Some(external_docs) = operation.get("externalDocs").and_then(Value::as_object) {
        let url = external_docs
            .get("url")
            .and_then(Value::as_str)
            .unwrap_or("");
        let description = external_docs
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("External docs");
        if !url.is_empty() {
            parts.push(format!(
                "<small>{}: <code>{}</code></small>",
                escape_html(description),
                escape_html(url)
            ));
        }
    }
    let callbacks = render_openapi_callbacks(operation.get("callbacks"));
    if !callbacks.is_empty() {
        parts.push(callbacks);
    }
    let servers = render_openapi_operation_servers(operation, path_item);
    if !servers.is_empty() {
        parts.push(servers);
    }
    if parts.is_empty() {
        "&nbsp;".to_string()
    } else {
        parts.join("<br>")
    }
}

fn render_openapi_callbacks(callbacks: Option<&Value>) -> String {
    render_openapi_callbacks_with_depth(callbacks, 0)
}

fn render_openapi_callbacks_with_depth(callbacks: Option<&Value>, depth: usize) -> String {
    const MAX_CALLBACK_DEPTH: usize = 3;
    let Some(callbacks) = callbacks.and_then(Value::as_object) else {
        return String::new();
    };
    if depth >= MAX_CALLBACK_DEPTH {
        return String::new();
    }
    let items = callbacks
        .iter()
        .flat_map(|(name, callback)| {
            callback
                .as_object()
                .into_iter()
                .flat_map(move |expressions| {
                    expressions.iter().flat_map(move |(expression, path_item)| {
                        path_item.as_object().into_iter().flat_map(move |methods| {
                            methods
                                .iter()
                                .filter(|(method, _)| is_openapi_method(method.as_str()))
                                .map(move |(method, operation)| {
                                    let label = operation
                                        .get("operationId")
                                        .or_else(|| operation.get("summary"))
                                        .and_then(Value::as_str)
                                        .unwrap_or("");
                                    let nested_callbacks = render_openapi_callbacks_with_depth(
                                        operation.get("callbacks"),
                                        depth + 1,
                                    );
                                    let mut parts = [
                                        escape_html(name),
                                        escape_html(&method.to_ascii_uppercase()),
                                        escape_html(expression),
                                        escape_html(label),
                                    ]
                                    .into_iter()
                                    .filter(|part| !part.is_empty())
                                    .collect::<Vec<_>>();
                                    if !nested_callbacks.is_empty() {
                                        parts.push(nested_callbacks);
                                    }
                                    parts.join(" ")
                                })
                        })
                    })
                })
        })
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>()
        .join("; ");
    if items.is_empty() {
        String::new()
    } else {
        format!("<small>callbacks: {items}</small>")
    }
}

fn render_openapi_webhook_rows(value: &Value) -> String {
    let Some(webhooks) = value.get("webhooks").and_then(Value::as_object) else {
        return String::new();
    };
    webhooks
        .iter()
        .flat_map(|(name, path_item)| {
            let path_parameters = path_item
                .get("parameters")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            path_item.as_object().into_iter().flat_map(move |methods| {
                methods
                    .iter()
                    .filter(|(method, _)| is_openapi_method(method.as_str()))
                    .map(move |(method, operation)| {
                        let summary = operation
                            .get("summary")
                            .or_else(|| operation.get("description"))
                            .and_then(Value::as_str)
                            .unwrap_or("");
                        let operation_id = operation
                            .get("operationId")
                            .and_then(Value::as_str)
                            .unwrap_or("");
                        let operation_label =
                            operation_label(summary, operation_id, operation, path_item);
                        let mut parameters = path_parameters.iter().collect::<Vec<_>>();
                        if let Some(operation_parameters) = operation
                            .get("parameters")
                            .and_then(Value::as_array)
                        {
                            parameters.extend(operation_parameters.iter());
                        }
                        format!(
                            "<tr><td><code>WEBHOOK {}</code></td><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                            escape_html(&method.to_ascii_uppercase()),
                            escape_html(name),
                            operation_label,
                            render_openapi_security(
                                operation.get("security").or_else(|| value.get("security"))
                            ),
                            render_openapi_parameters(&parameters),
                            render_openapi_request_body(operation.get("requestBody")),
                            render_openapi_responses(operation.get("responses"))
                        )
                    })
            })
        })
        .collect::<Vec<_>>()
        .join("")
}

fn render_openapi_security(security: Option<&Value>) -> String {
    let Some(security) = security.and_then(Value::as_array) else {
        return "&nbsp;".to_string();
    };
    if security.is_empty() {
        return "none".to_string();
    }
    let items = security
        .iter()
        .filter_map(Value::as_object)
        .flat_map(|requirement| {
            requirement.iter().map(|(scheme, scopes)| {
                let scopes = scopes
                    .as_array()
                    .map(|scopes| {
                        scopes
                            .iter()
                            .filter_map(Value::as_str)
                            .map(escape_html)
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                if scopes.is_empty() {
                    format!("<li><code>{}</code></li>", escape_html(scheme))
                } else {
                    format!(
                        "<li><code>{}</code> scopes: {scopes}</li>",
                        escape_html(scheme)
                    )
                }
            })
        })
        .collect::<Vec<_>>()
        .join("");
    if items.is_empty() {
        "&nbsp;".to_string()
    } else {
        format!("<ul>{items}</ul>")
    }
}

fn render_openapi_parameters(parameters: &[&Value]) -> String {
    let items = parameters
        .iter()
        .map(|parameter| {
            if let Some(reference) = parameter.get("$ref").and_then(Value::as_str) {
                return format!("<li><code>{}</code></li>", escape_html(reference));
            }
            let name = parameter
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("parameter");
            let location = parameter.get("in").and_then(Value::as_str).unwrap_or("");
            let required = parameter
                .get("required")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let schema = parameter
                .get("schema")
                .map(schema_type_summary)
                .unwrap_or_default();
            let description = parameter
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("");
            let mut label = format!(
                "<code>{}</code> {} {}",
                escape_html(name),
                escape_html(location),
                if required { "required" } else { "optional" }
            );
            if !schema.is_empty() {
                label.push_str(&format!(" {}", escape_html(&schema)));
            }
            let details = openapi_parameter_details(parameter);
            if !details.is_empty() {
                label.push_str(&format!(" {}", escape_html(&details)));
            }
            if !description.is_empty() {
                label.push_str(&format!(" - {}", escape_html(description)));
            }
            format!("<li>{label}</li>")
        })
        .collect::<Vec<_>>()
        .join("");
    if items.is_empty() {
        "&nbsp;".to_string()
    } else {
        format!("<ul>{items}</ul>")
    }
}

fn openapi_parameter_details(parameter: &Value) -> String {
    let mut details = Vec::new();
    for key in ["style", "explode", "allowReserved", "deprecated"] {
        if let Some(value) = parameter.get(key) {
            details.push(format!("{key}: {}", structured_value_summary(value)));
        }
    }
    let examples = render_openapi_media_examples(parameter);
    if !examples.is_empty() {
        details.push(examples);
    }
    let content = render_openapi_content(parameter.get("content"));
    if !content.is_empty() {
        details.push(format!("content: {content}"));
    }
    details.join("; ")
}

fn render_openapi_request_body(request_body: Option<&Value>) -> String {
    let Some(request_body) = request_body else {
        return "&nbsp;".to_string();
    };
    if let Some(reference) = request_body.get("$ref").and_then(Value::as_str) {
        return format!("<code>{}</code>", escape_html(reference));
    }
    let description = request_body
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("");
    let required = request_body
        .get("required")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let content = render_openapi_content(request_body.get("content"));
    let mut parts = Vec::new();
    if !description.is_empty() {
        parts.push(escape_html(description));
    }
    if required {
        parts.push("required".to_string());
    }
    if !content.is_empty() {
        parts.push(content);
    }
    if parts.is_empty() {
        "&nbsp;".to_string()
    } else {
        parts.join("<br>")
    }
}

fn render_openapi_responses(responses: Option<&Value>) -> String {
    let Some(responses) = responses.and_then(Value::as_object) else {
        return "&nbsp;".to_string();
    };
    let items = responses
        .iter()
        .map(|(status, response)| {
            format!(
                "<li><code>{}</code>: {}</li>",
                escape_html(status),
                openapi_response_summary(response)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    if items.is_empty() {
        "&nbsp;".to_string()
    } else {
        format!("<ul>{items}</ul>")
    }
}

fn openapi_response_summary(response: &Value) -> String {
    if let Some(reference) = response.get("$ref").and_then(Value::as_str) {
        return format!("<code>{}</code>", escape_html(reference));
    }
    let description = response
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("");
    let content = render_openapi_content(response.get("content"));
    let detail = [
        escape_html(description),
        content,
        render_openapi_headers(response.get("headers")),
        render_openapi_links(response.get("links")),
    ]
    .into_iter()
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join(" ");
    if detail.is_empty() {
        "&nbsp;".to_string()
    } else {
        detail
    }
}

fn render_openapi_content(content: Option<&Value>) -> String {
    let Some(content) = content.and_then(Value::as_object) else {
        return String::new();
    };
    content
        .iter()
        .map(|(content_type, media)| {
            let schema = media
                .get("schema")
                .map(schema_type_summary)
                .unwrap_or_default();
            let examples = render_openapi_media_examples(media);
            if schema.is_empty() && examples.is_empty() {
                format!("<code>{}</code>", escape_html(content_type))
            } else {
                [
                    format!(
                        "<code>{}</code> {}",
                        escape_html(content_type),
                        escape_html(&schema)
                    ),
                    examples,
                ]
                .into_iter()
                .filter(|part| !part.trim().is_empty())
                .collect::<Vec<_>>()
                .join(" ")
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_openapi_media_examples(media: &Value) -> String {
    let mut names = Vec::new();
    if media.get("example").is_some() {
        names.push("example".to_string());
    }
    if let Some(examples) = media.get("examples").and_then(Value::as_object) {
        names.extend(examples.keys().cloned());
    }
    if names.is_empty() {
        String::new()
    } else {
        format!("examples: {}", escape_html(&names.join(", ")))
    }
}

fn render_openapi_headers(headers: Option<&Value>) -> String {
    let Some(headers) = headers.and_then(Value::as_object) else {
        return String::new();
    };
    let items = headers
        .iter()
        .map(|(name, header)| openapi_header_summary(Some(name), header))
        .filter(|item| !item.is_empty())
        .map(|item| format!("<li>{item}</li>"))
        .collect::<Vec<_>>()
        .join("");
    if items.is_empty() {
        String::new()
    } else {
        format!("headers:<ul>{items}</ul>")
    }
}

fn openapi_header_summary(name: Option<&str>, header: &Value) -> String {
    let prefix = name
        .map(|name| format!("<code>{}</code>", escape_html(name)))
        .unwrap_or_default();
    if let Some(reference) = header.get("$ref").and_then(Value::as_str) {
        return [
            prefix,
            format!("ref {}", escape_html(&reference_tail(reference))),
        ]
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(": ");
    }
    let description = header
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("");
    let schema = header
        .get("schema")
        .map(schema_type_summary)
        .unwrap_or_default();
    let examples = render_openapi_media_examples(header);
    [
        prefix,
        escape_html(description),
        escape_html(&schema),
        examples,
    ]
    .into_iter()
    .filter(|part| !part.trim().is_empty())
    .collect::<Vec<_>>()
    .join(" ")
}

fn render_openapi_links(links: Option<&Value>) -> String {
    let Some(links) = links.and_then(Value::as_object) else {
        return String::new();
    };
    let items = links
        .iter()
        .map(|(name, link)| openapi_link_summary(Some(name), link))
        .filter(|item| !item.is_empty())
        .map(|item| format!("<li>{item}</li>"))
        .collect::<Vec<_>>()
        .join("");
    if items.is_empty() {
        String::new()
    } else {
        format!("links:<ul>{items}</ul>")
    }
}

fn openapi_link_summary(name: Option<&str>, link: &Value) -> String {
    let prefix = name
        .map(|name| format!("<code>{}</code>", escape_html(name)))
        .unwrap_or_default();
    if let Some(reference) = link.get("$ref").and_then(Value::as_str) {
        return [
            prefix,
            format!("ref {}", escape_html(&reference_tail(reference))),
        ]
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(": ");
    }
    let target = link
        .get("operationId")
        .or_else(|| link.get("operationRef"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let description = link
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("");
    let parameters = link
        .get("parameters")
        .and_then(Value::as_object)
        .map(|parameters| parameters.keys().cloned().collect::<Vec<_>>().join(", "))
        .unwrap_or_default();
    [
        prefix,
        escape_html(target),
        if parameters.is_empty() {
            String::new()
        } else {
            format!("parameters: {}", escape_html(&parameters))
        },
        escape_html(description),
    ]
    .into_iter()
    .filter(|part| !part.trim().is_empty())
    .collect::<Vec<_>>()
    .join(" ")
}

fn openapi_example_summary(example: &Value) -> String {
    if let Some(reference) = example.get("$ref").and_then(Value::as_str) {
        return format!("ref {}", escape_html(&reference_tail(reference)));
    }
    let summary = example.get("summary").and_then(Value::as_str).unwrap_or("");
    let description = example
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("");
    let external_value = example
        .get("externalValue")
        .and_then(Value::as_str)
        .unwrap_or("");
    let value_summary = example
        .get("value")
        .map(structured_value_summary)
        .unwrap_or_default();
    [
        escape_html(summary),
        escape_html(description),
        if external_value.is_empty() {
            String::new()
        } else {
            format!(
                "externalValue: <code>{}</code>",
                escape_html(external_value)
            )
        },
        if value_summary.is_empty() {
            String::new()
        } else {
            format!("value: {}", escape_html(&value_summary))
        },
    ]
    .into_iter()
    .filter(|part| !part.trim().is_empty())
    .collect::<Vec<_>>()
    .join(" ")
}

fn render_openapi_security_schemes(value: &Value) -> String {
    let Some(schemes) = value
        .pointer("/components/securitySchemes")
        .and_then(Value::as_object)
    else {
        return String::new();
    };
    if schemes.is_empty() {
        return String::new();
    }
    let items = schemes
        .iter()
        .map(|(name, scheme)| {
            let kind = scheme.get("type").and_then(Value::as_str).unwrap_or("");
            let location = scheme.get("in").and_then(Value::as_str).unwrap_or("");
            let header_name = scheme.get("name").and_then(Value::as_str).unwrap_or("");
            let flows = scheme
                .get("flows")
                .and_then(Value::as_object)
                .map(|flows| flows.keys().cloned().collect::<Vec<_>>().join(", "))
                .unwrap_or_default();
            let parts = [
                escape_html(kind),
                escape_html(location),
                escape_html(header_name),
                if flows.is_empty() {
                    String::new()
                } else {
                    format!("flows: {}", escape_html(&flows))
                },
            ]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
            format!("<li><code>{}</code> {parts}</li>", escape_html(name))
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<section class=\"api-security\"><h4>Security schemes</h4><ul>{items}</ul></section>")
}

fn render_openapi_reusable_components(value: &Value) -> String {
    [
        render_openapi_named_component_section(
            value,
            "/components/parameters",
            "Reusable parameters",
            |component| render_openapi_parameters(&[component]),
        ),
        render_openapi_named_component_section(
            value,
            "/components/requestBodies",
            "Reusable request bodies",
            |component| render_openapi_request_body(Some(component)),
        ),
        render_openapi_named_component_section(
            value,
            "/components/responses",
            "Reusable responses",
            |component| openapi_response_summary(component),
        ),
        render_openapi_named_component_section(
            value,
            "/components/headers",
            "Reusable headers",
            |component| openapi_header_summary(None, component),
        ),
        render_openapi_named_component_section(
            value,
            "/components/examples",
            "Reusable examples",
            |component| openapi_example_summary(component),
        ),
        render_openapi_named_component_section(
            value,
            "/components/links",
            "Reusable links",
            |component| openapi_link_summary(None, component),
        ),
        render_openapi_callback_component_section(value),
    ]
    .into_iter()
    .filter(|section| !section.is_empty())
    .collect::<Vec<_>>()
    .join("")
}

fn render_openapi_named_component_section(
    value: &Value,
    pointer: &str,
    heading: &str,
    render: fn(&Value) -> String,
) -> String {
    let Some(components) = value.pointer(pointer).and_then(Value::as_object) else {
        return String::new();
    };
    if components.is_empty() {
        return String::new();
    }
    let items = components
        .iter()
        .map(|(name, component)| {
            let summary = render(component);
            format!(
                "<li><code>{}</code> {}</li>",
                escape_html(name),
                if summary.is_empty() || summary == "&nbsp;" {
                    "&nbsp;".to_string()
                } else {
                    summary
                }
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<section class=\"api-components api-component-library\"><h4>{}</h4><ul>{items}</ul></section>",
        escape_html(heading)
    )
}

fn render_openapi_callback_component_section(value: &Value) -> String {
    let Some(callbacks) = value
        .pointer("/components/callbacks")
        .and_then(Value::as_object)
    else {
        return String::new();
    };
    if callbacks.is_empty() {
        return String::new();
    }
    let items = callbacks
        .iter()
        .map(|(name, callback)| {
            let mut wrapper = Map::new();
            wrapper.insert(name.clone(), callback.clone());
            let wrapped = Value::Object(wrapper);
            let summary = render_openapi_callbacks(Some(&wrapped));
            format!(
                "<li><code>{}</code> {}</li>",
                escape_html(name),
                if summary.is_empty() {
                    "&nbsp;".to_string()
                } else {
                    summary
                }
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<section class=\"api-components api-component-library\"><h4>Reusable callbacks</h4><ul>{items}</ul></section>"
    )
}

fn render_openapi_components(value: &Value) -> String {
    let Some(schemas) = value
        .pointer("/components/schemas")
        .and_then(Value::as_object)
    else {
        return String::new();
    };
    if schemas.is_empty() {
        return String::new();
    }
    let mut html = String::from("<section class=\"api-components\"><h4>Component schemas</h4>");
    for (name, schema) in schemas {
        let mut rows = Vec::new();
        collect_schema_rows(name, schema, false, &mut rows);
        html.push_str(&format!(
            "<h5>{}</h5><table class=\"transform-table json-schema\"><thead><tr><th>Field</th><th>Type</th><th>Required</th><th>Description</th><th>Constraints</th></tr></thead><tbody>",
            escape_html(name)
        ));
        for row in rows {
            html.push_str(&format!(
                "<tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                escape_html(&row.field),
                escape_html(&row.kind),
                if row.required { "yes" } else { "no" },
                escape_html(&row.description),
                escape_html(&row.constraints)
            ));
        }
        html.push_str("</tbody></table>");
    }
    html.push_str("</section>");
    html
}

fn collect_schema_rows(prefix: &str, schema: &Value, required: bool, rows: &mut Vec<SchemaRow>) {
    let field = if prefix.is_empty() { "root" } else { prefix };
    let description = schema
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    rows.push(SchemaRow {
        field: field.to_string(),
        kind: schema_type_summary(schema),
        required,
        description,
        constraints: schema_constraints(schema),
    });

    let required_fields = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
    if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
        for (field, child_schema) in properties {
            let child_prefix = if prefix.is_empty() || prefix == "root" {
                field.to_string()
            } else {
                format!("{prefix}.{field}")
            };
            collect_schema_rows(
                &child_prefix,
                child_schema,
                required_fields.contains(field.as_str()),
                rows,
            );
        }
    }
    if let Some(properties) = schema.get("patternProperties").and_then(Value::as_object) {
        for (pattern, child_schema) in properties {
            collect_schema_rows(
                &schema_child_path(field, &format!("patternProperties[{pattern}]")),
                child_schema,
                false,
                rows,
            );
        }
    }
    if let Some(items) = schema.get("items") {
        if let Some(tuple_items) = items.as_array() {
            for (index, item) in tuple_items.iter().enumerate() {
                collect_schema_rows(
                    &schema_child_path(field, &format!("items[{}]", index + 1)),
                    item,
                    false,
                    rows,
                );
            }
        } else {
            let child_prefix = if prefix.is_empty() || prefix == "root" {
                "items[]".to_string()
            } else {
                format!("{prefix}[]")
            };
            collect_schema_rows(&child_prefix, items, false, rows);
        }
    }
    if let Some(items) = schema.get("prefixItems").and_then(Value::as_array) {
        for (index, item) in items.iter().enumerate() {
            collect_schema_rows(
                &schema_child_path(field, &format!("prefixItems[{}]", index + 1)),
                item,
                false,
                rows,
            );
        }
    }
    for keyword in [
        "additionalProperties",
        "additionalItems",
        "contains",
        "propertyNames",
        "contentSchema",
        "unevaluatedItems",
        "unevaluatedProperties",
        "not",
        "if",
        "then",
        "else",
    ] {
        if let Some(child_schema) = schema.get(keyword).filter(|value| is_schema_value(value)) {
            collect_schema_rows(
                &schema_child_path(field, keyword),
                child_schema,
                false,
                rows,
            );
        }
    }
    for keyword in ["$defs", "definitions"] {
        if let Some(definitions) = schema.get(keyword).and_then(Value::as_object) {
            for (name, child_schema) in definitions {
                collect_schema_rows(
                    &schema_child_path(field, &format!("{keyword}[{name}]")),
                    child_schema,
                    false,
                    rows,
                );
            }
        }
    }
    if let Some(dependent_schemas) = schema.get("dependentSchemas").and_then(Value::as_object) {
        for (property, child_schema) in dependent_schemas {
            collect_schema_rows(
                &schema_child_path(field, &format!("dependentSchemas[{property}]")),
                child_schema,
                false,
                rows,
            );
        }
    }
    for keyword in ["allOf", "anyOf", "oneOf"] {
        if let Some(variants) = schema.get(keyword).and_then(Value::as_array) {
            for (index, variant) in variants.iter().enumerate() {
                let child_prefix = format!("{field}.{keyword}[{}]", index + 1);
                collect_schema_rows(&child_prefix, variant, false, rows);
            }
        }
    }
}

fn is_schema_value(value: &Value) -> bool {
    value.is_object() || value.is_boolean()
}

fn schema_child_path(parent: &str, child: &str) -> String {
    if parent.is_empty() || parent == "root" {
        child.to_string()
    } else {
        format!("{parent}.{child}")
    }
}

fn schema_type_summary(schema: &Value) -> String {
    if let Some(boolean_schema) = schema.as_bool() {
        return if boolean_schema {
            "any".to_string()
        } else {
            "never".to_string()
        };
    }
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        return schema_nullable_type(format!("ref {}", reference_tail(reference)), schema);
    }
    if let Some(items) = schema.get("enum").and_then(Value::as_array) {
        return schema_nullable_type(format!("enum {}", value_list_summary(items)), schema);
    }
    if let Some(constant) = schema.get("const") {
        return schema_nullable_type(
            format!("const {}", structured_value_summary(constant)),
            schema,
        );
    }
    if let Some(types) = schema.get("type").and_then(Value::as_array) {
        return schema_nullable_type(
            types
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" | "),
            schema,
        );
    }
    let kind = match schema.get("type").and_then(Value::as_str) {
        Some("array") => schema_array_type_summary(schema),
        Some(kind) => kind.to_string(),
        None if schema.get("properties").is_some() => "object".to_string(),
        None if schema.get("prefixItems").is_some() => schema_array_type_summary(schema),
        None if schema.get("items").is_some() => "array".to_string(),
        None if schema.get("contains").is_some() => "array".to_string(),
        None if schema.get("oneOf").is_some() => "oneOf".to_string(),
        None if schema.get("anyOf").is_some() => "anyOf".to_string(),
        None if schema.get("allOf").is_some() => "allOf".to_string(),
        None => String::new(),
    };
    schema_nullable_type(append_schema_discriminator(kind, schema), schema)
}

fn schema_array_type_summary(schema: &Value) -> String {
    let mut summary = if let Some(items) = schema.get("prefixItems").and_then(Value::as_array) {
        let items = items
            .iter()
            .map(schema_type_summary)
            .collect::<Vec<_>>()
            .join(", ");
        if items.is_empty() {
            "tuple".to_string()
        } else {
            format!("tuple<{items}>")
        }
    } else if let Some(items) = schema.get("items").and_then(Value::as_array) {
        let items = items
            .iter()
            .map(schema_type_summary)
            .collect::<Vec<_>>()
            .join(", ");
        if items.is_empty() {
            "tuple".to_string()
        } else {
            format!("tuple<{items}>")
        }
    } else if let Some(items) = schema.get("items") {
        format!("array<{}>", schema_type_summary(items))
    } else {
        "array".to_string()
    };

    if schema.get("prefixItems").is_some() {
        if let Some(items) = schema.get("items").filter(|items| is_schema_value(items)) {
            summary.push_str(&format!(" + array<{}>", schema_type_summary(items)));
        }
    }
    summary
}

fn schema_nullable_type(kind: String, schema: &Value) -> String {
    if !schema
        .get("nullable")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return kind;
    }
    if kind
        .split('|')
        .any(|part| part.trim().eq_ignore_ascii_case("null"))
    {
        return kind;
    }
    if kind.is_empty() {
        "null".to_string()
    } else {
        format!("{kind} | null")
    }
}

fn schema_constraints(schema: &Value) -> String {
    if let Some(boolean_schema) = schema.as_bool() {
        return if boolean_schema {
            "boolean schema: accepts any value".to_string()
        } else {
            "boolean schema: rejects all values".to_string()
        };
    }
    let mut constraints = Vec::new();
    for key in [
        "$schema",
        "$id",
        "$anchor",
        "$dynamicAnchor",
        "$dynamicRef",
        "$recursiveAnchor",
        "$recursiveRef",
        "$comment",
        "format",
        "pattern",
        "minimum",
        "maximum",
        "exclusiveMinimum",
        "exclusiveMaximum",
        "minLength",
        "maxLength",
        "minItems",
        "maxItems",
        "minContains",
        "maxContains",
        "minProperties",
        "maxProperties",
        "multipleOf",
        "contentEncoding",
        "contentMediaType",
        "default",
        "example",
        "readOnly",
        "writeOnly",
        "deprecated",
        "uniqueItems",
        "nullable",
    ] {
        if let Some(value) = schema.get(key) {
            constraints.push(format!("{key}: {}", structured_value_summary(value)));
        }
    }
    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        let summary = required
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        if !summary.is_empty() {
            constraints.push(format!("required: {summary}"));
        }
    }
    if let Some(items) = schema.get("enum").and_then(Value::as_array) {
        constraints.push(format!("enum: {}", value_list_summary(items)));
    }
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        constraints.push(format!("ref: {}", reference_tail(reference)));
    }
    if let Some(discriminator) = schema_discriminator_summary(schema) {
        constraints.push(discriminator);
    }
    for key in ["additionalProperties", "additionalItems"] {
        if let Some(additional_properties) = schema.get(key) {
            if !additional_properties.is_boolean() {
                continue;
            }
            constraints.push(format!(
                "{key}: {}",
                structured_value_summary(additional_properties)
            ));
        }
    }
    for key in ["unevaluatedItems", "unevaluatedProperties"] {
        if let Some(value) = schema.get(key).filter(|value| value.is_boolean()) {
            constraints.push(format!("{key}: {}", structured_value_summary(value)));
        }
    }
    if let Some(content_schema) = schema.get("contentSchema") {
        constraints.push(format!(
            "contentSchema: {}",
            structured_value_summary(content_schema)
        ));
    }
    if let Some(contains) = schema.get("contains") {
        constraints.push(format!("contains: {}", schema_type_summary(contains)));
    }
    if let Some(prefix_items) = schema.get("prefixItems").and_then(Value::as_array) {
        constraints.push(format!("prefixItems: {} items", prefix_items.len()));
    }
    if let Some(tuple_items) = schema.get("items").and_then(Value::as_array) {
        constraints.push(format!("items: {} tuple items", tuple_items.len()));
    }
    for keyword in ["allOf", "anyOf", "oneOf"] {
        if let Some(variants) = schema.get(keyword).and_then(Value::as_array) {
            constraints.push(format!("{keyword}: {} variants", variants.len()));
        }
    }
    for keyword in [
        "$defs",
        "definitions",
        "dependentSchemas",
        "patternProperties",
    ] {
        if let Some(map) = schema.get(keyword).and_then(Value::as_object) {
            let keys = map.keys().cloned().collect::<Vec<_>>().join(", ");
            if !keys.is_empty() {
                constraints.push(format!("{keyword}: {keys}"));
            }
        }
    }
    if let Some(vocabulary) = schema.get("$vocabulary").and_then(Value::as_object) {
        let keys = vocabulary.keys().cloned().collect::<Vec<_>>().join(", ");
        if !keys.is_empty() {
            constraints.push(format!("$vocabulary: {keys}"));
        }
    }
    if let Some(dependent_required) = schema.get("dependentRequired").and_then(Value::as_object) {
        let summary = dependent_required
            .iter()
            .map(|(property, requirements)| {
                let required = requirements
                    .as_array()
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                format!("{property} -> {required}")
            })
            .collect::<Vec<_>>()
            .join("; ");
        if !summary.is_empty() {
            constraints.push(format!("dependentRequired: {summary}"));
        }
    }
    if let Some(examples) = schema.get("examples").and_then(Value::as_array) {
        constraints.push(format!("examples: {}", value_list_summary(examples)));
    }
    constraints.join("; ")
}

fn append_schema_discriminator(kind: String, schema: &Value) -> String {
    let Some(discriminator) = schema_discriminator_property(schema) else {
        return kind;
    };
    let mapping = schema_discriminator_mapping(schema)
        .map(|mapping| format!(" mapping {mapping}"))
        .unwrap_or_default();
    if kind.is_empty() {
        format!("discriminator {discriminator}{mapping}")
    } else {
        format!("{kind} discriminator {discriminator}{mapping}")
    }
}

fn schema_discriminator_summary(schema: &Value) -> Option<String> {
    let property = schema_discriminator_property(schema)?;
    if let Some(mapping) = schema_discriminator_mapping(schema) {
        Some(format!("discriminator: {property}; mapping: {mapping}"))
    } else {
        Some(format!("discriminator: {property}"))
    }
}

fn schema_discriminator_property(schema: &Value) -> Option<String> {
    schema
        .get("discriminator")
        .and_then(Value::as_object)
        .and_then(|discriminator| discriminator.get("propertyName"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn schema_discriminator_mapping(schema: &Value) -> Option<String> {
    let mapping = schema
        .get("discriminator")
        .and_then(Value::as_object)
        .and_then(|discriminator| discriminator.get("mapping"))
        .and_then(Value::as_object)
        .map(|mapping| mapping.keys().cloned().collect::<Vec<_>>().join(", "))
        .unwrap_or_default();
    (!mapping.is_empty()).then_some(mapping)
}

fn reference_tail(reference: &str) -> String {
    reference
        .rsplit('/')
        .next()
        .unwrap_or(reference)
        .replace("~1", "/")
        .replace("~0", "~")
}

fn value_list_summary(values: &[Value]) -> String {
    values
        .iter()
        .map(structured_value_summary)
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_structured_table(format: &str, value: &Value) -> Option<String> {
    let (caption, rows) = structured_table_rows(value)?;
    if rows.is_empty() || !rows.iter().all(|row| row.is_object()) {
        return None;
    }
    let flattened_rows = rows
        .iter()
        .map(flatten_structured_table_row)
        .collect::<Vec<_>>();
    let headers = flattened_rows
        .iter()
        .flat_map(|object| object.keys().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if headers.is_empty() {
        return None;
    }
    let mut html = format!("<table class=\"transform-table transform-{format}\">");
    if let Some(caption) = caption {
        html.push_str(&format!("<caption>{}</caption>", escape_html(&caption)));
    }
    html.push_str("<thead><tr>");
    for header in &headers {
        html.push_str(&format!("<th>{}</th>", escape_html(header)));
    }
    html.push_str("</tr></thead><tbody>");
    for object in &flattened_rows {
        html.push_str("<tr>");
        for header in &headers {
            let cell = object.get(header).cloned().unwrap_or_default();
            html.push_str(&format!("<td>{}</td>", escape_html(&cell)));
        }
        html.push_str("</tr>");
    }
    html.push_str("</tbody></table>");
    Some(html)
}

fn flatten_structured_table_row(row: &Value) -> BTreeMap<String, String> {
    let mut cells = BTreeMap::new();
    if let Some(object) = row.as_object() {
        for (key, value) in object {
            flatten_structured_table_value(key, value, &mut cells);
        }
    }
    cells
}

fn flatten_structured_table_value(path: &str, value: &Value, cells: &mut BTreeMap<String, String>) {
    match value {
        Value::Object(object) if !object.is_empty() => {
            for (key, child) in object {
                flatten_structured_table_value(&format!("{path}.{key}"), child, cells);
            }
        }
        Value::Array(values) if values.is_empty() => {
            cells.insert(path.to_string(), "[]".to_string());
        }
        Value::Array(values) if values.iter().all(is_structured_scalar) => {
            cells.insert(path.to_string(), value_list_summary(values));
        }
        Value::Object(object) if object.is_empty() => {
            cells.insert(path.to_string(), "{}".to_string());
        }
        _ => {
            cells.insert(path.to_string(), structured_value_summary(value));
        }
    }
}

fn is_structured_scalar(value: &Value) -> bool {
    matches!(
        value,
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
    )
}

fn structured_table_rows(value: &Value) -> Option<(Option<String>, Vec<Value>)> {
    if let Some(rows) = value.as_array() {
        return Some((None, rows.clone()));
    }
    let object = value.as_object()?;
    for key in ["rows", "records", "data", "items", "values"] {
        if let Some(rows) = object.get(key).and_then(Value::as_array) {
            if rows_are_object_rows(rows) {
                return Some((Some(key.to_string()), rows.clone()));
            }
        }
        if let Some(rows) = object
            .get(key)
            .and_then(Value::as_object)
            .and_then(keyed_object_rows)
        {
            return Some((Some(key.to_string()), rows));
        }
    }
    let mut array_fields = object.iter().filter_map(|(key, value)| {
        value
            .as_array()
            .filter(|rows| rows_are_object_rows(rows))
            .map(|rows| (key.as_str(), rows))
    });
    if let Some((key, rows)) = array_fields.next() {
        if array_fields.next().is_none() {
            return Some((Some(key.to_string()), rows.clone()));
        }
    }
    let mut object_fields = object
        .iter()
        .filter_map(|(key, value)| value.as_object().map(|rows| (key.as_str(), rows)));
    if let Some((key, rows)) = object_fields.next() {
        if object_fields.next().is_none() {
            if let Some(rows) = keyed_object_rows(rows) {
                return Some((Some(key.to_string()), rows));
            }
        }
    }
    keyed_object_rows(object)
        .map(|rows| (Some("entries".to_string()), rows))
        .or_else(|| scalar_object_rows(object).map(|rows| (Some("fields".to_string()), rows)))
}

fn rows_are_object_rows(rows: &[Value]) -> bool {
    !rows.is_empty() && rows.iter().all(Value::is_object)
}

fn keyed_object_rows(object: &Map<String, Value>) -> Option<Vec<Value>> {
    if object.len() < 2 || !object.values().all(Value::is_object) {
        return None;
    }
    let key_column = if object
        .values()
        .filter_map(Value::as_object)
        .any(|row| row.contains_key("key"))
    {
        "_key"
    } else {
        "key"
    };
    let rows = object
        .iter()
        .map(|(key, value)| {
            let mut row = value.as_object()?.clone();
            row.insert(key_column.to_string(), Value::String(key.clone()));
            Some(Value::Object(row))
        })
        .collect::<Option<Vec<_>>>()?;
    (!rows.is_empty()).then_some(rows)
}

fn scalar_object_rows(object: &Map<String, Value>) -> Option<Vec<Value>> {
    if object.is_empty() || !object.values().all(is_structured_scalar_or_scalar_array) {
        return None;
    }
    Some(
        object
            .iter()
            .map(|(key, value)| {
                let mut row = Map::new();
                row.insert("key".to_string(), Value::String(key.clone()));
                row.insert("value".to_string(), value.clone());
                Value::Object(row)
            })
            .collect(),
    )
}

fn is_structured_scalar_or_scalar_array(value: &Value) -> bool {
    is_structured_scalar(value)
        || value
            .as_array()
            .is_some_and(|values| values.iter().all(is_structured_scalar))
}

fn render_structured_tree(label: &str, value: &Value) -> String {
    match value {
        Value::Object(object) => {
            let mut html = format!(
                "<details open><summary>{}</summary><dl>",
                escape_html(label)
            );
            for (key, value) in object {
                html.push_str("<dt>");
                html.push_str(&escape_html(key));
                html.push_str("</dt><dd>");
                html.push_str(&render_structured_tree(key, value));
                html.push_str("</dd>");
            }
            html.push_str("</dl></details>");
            html
        }
        Value::Array(values) => {
            let mut html = format!(
                "<details open><summary>{} [{}]</summary><ol>",
                escape_html(label),
                values.len()
            );
            for value in values {
                html.push_str("<li>");
                html.push_str(&render_structured_tree("item", value));
                html.push_str("</li>");
            }
            html.push_str("</ol></details>");
            html
        }
        _ => escape_html(&structured_value_summary(value)),
    }
}

fn structured_value_summary(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Array(values) => format!("[{} items]", values.len()),
        Value::Object(object) => format!("{{{} fields}}", object.len()),
    }
}

fn parse_json_or_yaml(body: &str) -> Result<Value, String> {
    serde_json::from_str::<Value>(body)
        .or_else(|_| serde_yaml::from_str::<Value>(body))
        .map_err(|err| err.to_string())
}
