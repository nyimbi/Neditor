use serde_json::{json, Value};

pub(crate) fn apply_compile_options(metadata: &mut Value, options: Option<&Value>) {
    let Some(fields) = metadata.as_object_mut() else {
        return;
    };
    apply_default_citation_style(fields, options);
    apply_default_brand_profile(fields, options);
}

fn apply_default_citation_style(
    fields: &mut serde_json::Map<String, Value>,
    options: Option<&Value>,
) {
    let Some(style) = options
        .and_then(|value| value.get("defaultCitationStyle"))
        .and_then(Value::as_str)
        .filter(|style| matches!(*style, "title" | "author-year" | "key"))
    else {
        return;
    };
    if fields.contains_key("citationStyle")
        || fields.contains_key("cslStyle")
        || fields.contains_key("citation_style")
    {
        return;
    }
    fields.insert(
        "citationStyle".to_string(),
        Value::String(style.to_string()),
    );
}

fn apply_default_brand_profile(
    fields: &mut serde_json::Map<String, Value>,
    options: Option<&Value>,
) {
    let Some(profile) = options
        .and_then(|value| value.get("defaultBrandProfile"))
        .and_then(Value::as_object)
    else {
        return;
    };
    let defaults = ["name", "color", "logo", "font"]
        .into_iter()
        .filter_map(|key| {
            profile
                .get(key)
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| (key, Value::String(value.to_string())))
        })
        .collect::<Vec<_>>();
    if defaults.is_empty() {
        return;
    }
    let brand = fields.entry("brand").or_insert_with(|| json!({}));
    let Some(brand_fields) = brand.as_object_mut() else {
        return;
    };
    for (key, value) in defaults {
        brand_fields.entry(key.to_string()).or_insert(value);
    }
    apply_default_layout_template(fields, profile, "header");
    apply_default_layout_template(fields, profile, "footer");
    apply_default_scalar_metadata(fields, profile, "legalDisclaimer");
}

fn apply_default_layout_template(
    fields: &mut serde_json::Map<String, Value>,
    profile: &serde_json::Map<String, Value>,
    key: &str,
) {
    let Some(value) = profile
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    let layout = fields.entry("layout").or_insert_with(|| json!({}));
    let Some(layout_fields) = layout.as_object_mut() else {
        return;
    };
    layout_fields
        .entry(key.to_string())
        .or_insert_with(|| Value::String(value.to_string()));
}

fn apply_default_scalar_metadata(
    fields: &mut serde_json::Map<String, Value>,
    profile: &serde_json::Map<String, Value>,
    key: &str,
) {
    let Some(value) = profile
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    fields
        .entry(key.to_string())
        .or_insert_with(|| Value::String(value.to_string()));
}
