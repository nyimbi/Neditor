use crate::{diag, escape_html, DocumentDiagnostic};
use serde_json::Value;
use std::collections::{BTreeSet, HashSet};

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
    html.push_str(&render_openapi_servers(&value));
    html.push_str(
        "<table class=\"transform-table openapi\"><thead><tr><th>Method</th><th>Path</th><th>Operation</th><th>Parameters</th><th>Request body</th><th>Responses</th></tr></thead><tbody>",
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
                    let operation_label = operation_label(summary, operation_id, operation);
                    let mut parameters = path_parameters.iter().collect::<Vec<_>>();
                    if let Some(operation_parameters) =
                        operation.get("parameters").and_then(Value::as_array)
                    {
                        parameters.extend(operation_parameters.iter());
                    }
                    html.push_str(&format!(
                        "<tr><td><code>{}</code></td><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                        escape_html(&method.to_ascii_uppercase()),
                        escape_html(path),
                        operation_label,
                        render_openapi_parameters(&parameters),
                        render_openapi_request_body(operation.get("requestBody")),
                        render_openapi_responses(operation.get("responses"))
                    ));
                }
            }
        }
    }
    html.push_str("</tbody></table>");
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
    if servers.is_empty() {
        return String::new();
    }
    let items = servers
        .iter()
        .filter_map(|server| {
            let url = server.get("url").and_then(Value::as_str)?;
            let description = server
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("");
            Some(if description.is_empty() {
                format!("<li><code>{}</code></li>", escape_html(url))
            } else {
                format!(
                    "<li><code>{}</code> - {}</li>",
                    escape_html(url),
                    escape_html(description)
                )
            })
        })
        .collect::<Vec<_>>()
        .join("");
    if items.is_empty() {
        String::new()
    } else {
        format!("<ul class=\"api-servers\">{items}</ul>")
    }
}

fn is_openapi_method(method: &str) -> bool {
    matches!(
        method,
        "get" | "put" | "post" | "delete" | "options" | "head" | "patch" | "trace"
    )
}

fn operation_label(summary: &str, operation_id: &str, operation: &Value) -> String {
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
    if !tags.is_empty() {
        parts.push(format!("<small>{tags}</small>"));
    }
    if parts.is_empty() {
        "&nbsp;".to_string()
    } else {
        parts.join("<br>")
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
            if let Some(reference) = response.get("$ref").and_then(Value::as_str) {
                return format!(
                    "<li><code>{}</code>: <code>{}</code></li>",
                    escape_html(status),
                    escape_html(reference)
                );
            }
            let description = response
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("");
            let content = render_openapi_content(response.get("content"));
            let detail = [escape_html(description), content]
                .into_iter()
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            format!(
                "<li><code>{}</code>: {}</li>",
                escape_html(status),
                if detail.is_empty() {
                    "&nbsp;".to_string()
                } else {
                    detail
                }
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
            if schema.is_empty() {
                format!("<code>{}</code>", escape_html(content_type))
            } else {
                format!(
                    "<code>{}</code> {}",
                    escape_html(content_type),
                    escape_html(&schema)
                )
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
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
    if let Some(items) = schema.get("items") {
        let child_prefix = if prefix.is_empty() || prefix == "root" {
            "items[]".to_string()
        } else {
            format!("{prefix}[]")
        };
        collect_schema_rows(&child_prefix, items, false, rows);
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

fn schema_type_summary(schema: &Value) -> String {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        return format!("ref {}", reference_tail(reference));
    }
    if let Some(items) = schema.get("enum").and_then(Value::as_array) {
        return format!("enum {}", value_list_summary(items));
    }
    if let Some(constant) = schema.get("const") {
        return format!("const {}", structured_value_summary(constant));
    }
    if let Some(types) = schema.get("type").and_then(Value::as_array) {
        return types
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(" | ");
    }
    match schema.get("type").and_then(Value::as_str) {
        Some("array") => schema
            .get("items")
            .map(|items| format!("array<{}>", schema_type_summary(items)))
            .unwrap_or_else(|| "array".to_string()),
        Some(kind) => kind.to_string(),
        None if schema.get("properties").is_some() => "object".to_string(),
        None if schema.get("items").is_some() => "array".to_string(),
        None if schema.get("oneOf").is_some() => "oneOf".to_string(),
        None if schema.get("anyOf").is_some() => "anyOf".to_string(),
        None if schema.get("allOf").is_some() => "allOf".to_string(),
        None => String::new(),
    }
}

fn schema_constraints(schema: &Value) -> String {
    let mut constraints = Vec::new();
    for key in [
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
        "default",
        "example",
    ] {
        if let Some(value) = schema.get(key) {
            constraints.push(format!("{key}: {}", structured_value_summary(value)));
        }
    }
    if let Some(items) = schema.get("enum").and_then(Value::as_array) {
        constraints.push(format!("enum: {}", value_list_summary(items)));
    }
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        constraints.push(format!("ref: {}", reference_tail(reference)));
    }
    constraints.join("; ")
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
    let rows = value.as_array()?;
    if rows.is_empty() || !rows.iter().all(Value::is_object) {
        return None;
    }
    let headers = rows
        .iter()
        .filter_map(Value::as_object)
        .flat_map(|object| object.keys().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if headers.is_empty() {
        return None;
    }
    let mut html = format!("<table class=\"transform-table transform-{format}\"><thead><tr>");
    for header in &headers {
        html.push_str(&format!("<th>{}</th>", escape_html(header)));
    }
    html.push_str("</tr></thead><tbody>");
    for row in rows {
        let object = row.as_object()?;
        html.push_str("<tr>");
        for header in &headers {
            let cell = object
                .get(header)
                .map(structured_value_summary)
                .unwrap_or_default();
            html.push_str(&format!("<td>{}</td>", escape_html(&cell)));
        }
        html.push_str("</tr>");
    }
    html.push_str("</tbody></table>");
    Some(html)
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
