use super::*;

#[test]
fn compiler_resolves_metadata_variables_transforms_and_manifest() {
    let response = compile(CompileRequest {
        text: sample_document(),
        file_path: None,
    });

    assert_eq!(response.semantic.title, "Test Report");
    assert_eq!(response.semantic.status, "approved");
    assert!(response.compiled_markdown.contains("Prepared for Acme."));
    assert!(response.compiled_markdown.contains("Margin: 60.00%"));
    assert!(response.compiled_markdown.contains("After tax: $42.00"));
    assert!(response.compiled_markdown.contains("Healthy score: 60"));
    assert!(response.compiled_markdown.contains("Discount: 12.50%"));
    assert!(response.html.contains("Table of Contents"));
    assert!(response.html.contains("transform-table"));
    assert!(response.html.contains("<h1 id=\"test-report\">"));
    assert!(response.html.contains("href=\"#test-report\""));
    assert!(response.index_terms.iter().any(|term| term == "ARR"));
    assert_eq!(response.export_manifest.document_version, "1.2.0");
    let csv_artifact = response
        .transform_artifacts
        .iter()
        .find(|artifact| artifact.name == "csv")
        .expect("csv transform artifact");
    assert!(!csv_artifact.output_hash.is_empty());
    assert!(csv_artifact.source.contains("Region,Revenue"));
    assert!(csv_artifact.source_line.is_some_and(|line| line > 1));
    assert!(csv_artifact.end_source_line >= csv_artifact.source_line);
    assert_eq!(
        csv_artifact.options.get("caption").and_then(Value::as_str),
        Some("Regional revenue")
    );
    assert_eq!(
        csv_artifact.options.get("audited").and_then(Value::as_bool),
        Some(true)
    );
    let manifest_csv_artifact = response
        .export_manifest
        .transform_artifacts
        .iter()
        .find(|artifact| artifact.get("name").and_then(Value::as_str) == Some("csv"))
        .expect("csv manifest artifact");
    assert_eq!(
        manifest_csv_artifact
            .get("sourceHash")
            .and_then(Value::as_str),
        Some(csv_artifact.source_hash.as_str())
    );
    assert_eq!(
        manifest_csv_artifact.get("source").and_then(Value::as_str),
        Some(csv_artifact.source.as_str())
    );
    assert_eq!(
        manifest_csv_artifact
            .get("sourceFile")
            .and_then(Value::as_str),
        csv_artifact.source_file.as_deref()
    );
    assert_eq!(
        manifest_csv_artifact
            .get("sourceLine")
            .and_then(Value::as_u64),
        csv_artifact.source_line.map(|line| line as u64)
    );
    assert_eq!(
        manifest_csv_artifact
            .get("endSourceLine")
            .and_then(Value::as_u64),
        csv_artifact.end_source_line.map(|line| line as u64)
    );
    assert_eq!(
        manifest_csv_artifact
            .get("options")
            .and_then(|options| options.get("caption"))
            .and_then(Value::as_str),
        Some("Regional revenue")
    );
    assert_eq!(
        manifest_csv_artifact
            .get("outputHash")
            .and_then(Value::as_str),
        Some(csv_artifact.output_hash.as_str())
    );
    assert!(manifest_csv_artifact
        .get("cacheKey")
        .and_then(Value::as_str)
        .is_some_and(|cache_key| !cache_key.is_empty()));
    assert_eq!(
        manifest_csv_artifact
            .get("executionKind")
            .and_then(Value::as_str),
        Some("embedded")
    );
    assert_eq!(
        manifest_csv_artifact
            .get("inputMode")
            .and_then(Value::as_str),
        Some("embedded")
    );
    assert_eq!(
        manifest_csv_artifact
            .get("engineVersion")
            .and_then(Value::as_str),
        Some(env!("CARGO_PKG_VERSION"))
    );
    assert!(response
        .formula_graph
        .iter()
        .any(|formula| formula.name == "profit" && formula.value == Some(60.0)));
    let profit_formula = response
        .formula_graph
        .iter()
        .find(|formula| formula.name == "profit")
        .expect("profit formula");
    assert!(matches!(
        profit_formula.ast.as_ref(),
        Some(calculations::FormulaAstNode::Binary { op, .. }) if op == "-"
    ));
    assert!(response
        .formula_graph
        .iter()
        .any(|formula| formula.name == "healthy" && formula.value == Some(1.0)));
    assert!(response
        .formula_graph
        .iter()
        .any(|formula| formula.name == "target_met" && formula.value == Some(1.0)));
    assert!(response
        .formula_graph
        .iter()
        .any(|formula| formula.name == "cost_match" && formula.value == Some(1.0)));
    assert!(response
        .formula_graph
        .iter()
        .any(|formula| formula.name == "spread" && formula.value == Some(1.0)));
    assert!(response.formula_graph.iter().any(|formula| {
        formula.name == "discount"
            && (formula.value.unwrap_or_default() - 0.125).abs() < f64::EPSILON
    }));
}

#[test]
fn compiler_supports_default_document_variables() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Defaults\nstatus: approved\napprovedBy: QA\nclient: Acme\n---\n# Defaults\nPrepared for {{client | default:Fallback}} in {{region | default:\"East Africa\"}}.\nStill missing {{owner}}.\n".to_string(),
            file_path: None,
        });

    assert!(response
        .compiled_markdown
        .contains("Prepared for Acme in East Africa."));
    let missing_owner = response
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic
                .message
                .contains("Missing document variable: owner")
        })
        .expect("missing owner diagnostic");
    assert_eq!(missing_owner.line, Some(9));
    assert_eq!(missing_owner.column, Some(15));
    assert_eq!(missing_owner.end_line, Some(9));
    assert_eq!(missing_owner.end_column, Some(24));
    assert_eq!(missing_owner.source_file.as_deref(), Some("untitled.md"));
    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("region")));
}

#[test]
fn compiler_reports_malformed_front_matter_with_source_ranges() {
    let invalid = compile(CompileRequest {
        text: "---\ntitle: [unterminated\nstatus: approved\n---\n# Invalid\n".to_string(),
        file_path: None,
    });
    let yaml_error = invalid
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.starts_with("Invalid YAML front matter:"))
        .expect("invalid YAML front matter diagnostic");
    assert_eq!(yaml_error.severity, "error");
    assert_eq!(yaml_error.source_file.as_deref(), Some("untitled.md"));
    assert!(yaml_error.line.is_some_and(|line| line >= 2));
    assert!(yaml_error.column.is_some());
    assert_eq!(yaml_error.end_line, yaml_error.line);
    assert!(yaml_error.end_column.is_some());
    assert!(yaml_error
        .suggestion
        .as_deref()
        .is_some_and(|suggestion| suggestion.contains("Fix the YAML syntax")));

    let non_mapping = compile(CompileRequest {
        text: "---\n- title\n- status\n---\n# Non Mapping\n".to_string(),
        file_path: None,
    });
    let shape_error = non_mapping
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message == "YAML front matter must be a mapping.")
        .expect("non-mapping front matter diagnostic");
    assert_eq!(shape_error.severity, "error");
    assert_eq!(shape_error.source_file.as_deref(), Some("untitled.md"));
    assert_eq!(shape_error.line, Some(2));
    assert_eq!(shape_error.column, Some(1));
    assert_eq!(shape_error.end_line, Some(2));
    assert!(shape_error.end_column.is_some_and(|column| column > 1));
    assert!(shape_error
        .suggestion
        .as_deref()
        .is_some_and(|suggestion| suggestion.contains("key-value YAML")));
}

#[test]
fn compiler_formats_document_variables_and_reports_bad_filters() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Variable Filters\nstatus: approved\napprovedBy: QA\nclient: acme holdings\nregion: ' east africa '\nbudget: 1234.6\nmargin: 0.275\n---\n# Variable Filters\nClient: {{client | title}}\nRegion: {{region | trim | upper}}\nBudget: {{budget | currency}}\nMargin: {{margin | percent}}\nRounded: {{budget | round}}\nMissing: {{owner | default:'strategy office' | title}}\nBad filter: {{client | snake}}\nBad numeric: {{client | currency}}\n".to_string(),
        file_path: None,
    });

    assert!(response.compiled_markdown.contains("Client: Acme Holdings"));
    assert!(response.compiled_markdown.contains("Region: EAST AFRICA"));
    assert!(response.compiled_markdown.contains("Budget: $1234.60"));
    assert!(response.compiled_markdown.contains("Margin: 27.50%"));
    assert!(response.compiled_markdown.contains("Rounded: 1235"));
    assert!(response
        .compiled_markdown
        .contains("Missing: Strategy Office"));
    assert!(response
        .compiled_markdown
        .contains("Bad filter: acme holdings"));
    assert!(response
        .compiled_markdown
        .contains("Bad numeric: acme holdings"));

    let unsupported = response
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic
                .message
                .contains("Unsupported document variable filter 'snake' for client")
        })
        .expect("unsupported variable filter diagnostic");
    assert_eq!(unsupported.severity, "warning");
    assert_eq!(unsupported.source_file.as_deref(), Some("untitled.md"));
    assert!(unsupported.column.is_some());
    assert!(unsupported
        .suggestion
        .as_deref()
        .is_some_and(|suggestion| suggestion.contains("supported filters")));
    assert!(response.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("Cannot apply numeric document variable filter 'currency' to client")
            && diagnostic
                .suggestion
                .as_deref()
                .is_some_and(|suggestion| suggestion.contains("numeric values"))
    }));
    assert!(!response.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("Missing document variable: owner")));
}

#[test]
fn compiler_handles_document_variable_filter_edge_cases() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Variable Filter Edges\nstatus: approved\napprovedBy: QA\nclient: Acme HOLDINGS\nbudgetText: '$1,234.60'\nmarginText: '27.5%'\n---\n# Variable Filter Edges\nUpper alias: {{client | uppercase}}\nLower alias: {{client | lowercase}}\nTitle alias: {{client | titlecase}}\nCurrency string: {{budgetText | currency}}\nRounded string: {{budgetText | round}}\nPercent string: {{marginText | percent}}\nDefault equals: {{owner | default=Strategy Office | upper}}\nDefault space: {{reviewer | default Strategy Lead | title}}\n".to_string(),
        file_path: None,
    });

    assert!(response
        .compiled_markdown
        .contains("Upper alias: ACME HOLDINGS"));
    assert!(response
        .compiled_markdown
        .contains("Lower alias: acme holdings"));
    assert!(response
        .compiled_markdown
        .contains("Title alias: Acme Holdings"));
    assert!(response
        .compiled_markdown
        .contains("Currency string: $1234.60"));
    assert!(response.compiled_markdown.contains("Rounded string: 1235"));
    assert!(response
        .compiled_markdown
        .contains("Percent string: 27.50%"));
    assert!(response
        .compiled_markdown
        .contains("Default equals: STRATEGY OFFICE"));
    assert!(response
        .compiled_markdown
        .contains("Default space: Strategy Lead"));
    assert!(response
        .diagnostics
        .iter()
        .all(|diagnostic| !diagnostic.message.contains("document variable filter")));
}

#[test]
fn compiler_resolves_literal_dotted_front_matter_keys() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Dotted Keys\nstatus: approved\napprovedBy: QA\nclient.name: Acme Holdings\nclient.owner: Strategy Office\nprofile:\n  name: Nested Profile\nprofile.name: Literal Profile\nlayout.header: Board Pack\n---\n# Dotted Keys\nClient: {{client.name}}\nOwner: {{client.owner}}\nProfile: {{profile.name}}\nHeader: {{layout.header}}\nMissing: {{layout.footer | default=No footer}}\n".to_string(),
        file_path: None,
    });

    assert!(response.compiled_markdown.contains("Client: Acme Holdings"));
    assert!(response
        .compiled_markdown
        .contains("Owner: Strategy Office"));
    assert!(response
        .compiled_markdown
        .contains("Profile: Nested Profile"));
    assert!(response.compiled_markdown.contains("Header: Board Pack"));
    assert!(response.compiled_markdown.contains("Missing: No footer"));
    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Missing document variable")));
}

#[test]
fn compiler_strips_yaml_tags_from_front_matter_metadata() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-tagged-front-matter-test-{unique}"));
    fs::create_dir_all(root.join("data")).expect("create data dir");
    fs::write(
        root.join("data").join("tagged.csv"),
        "Region,Revenue\nEast,100\nWest,80\n",
    )
    .expect("write tagged data source");
    let doc = root.join("tagged.md");

    let response = compile(CompileRequest {
        text: "---\ntitle: !docs!title Tagged Metadata\nstatus: approved\napprovedBy: QA\nclient: !docs!client Acme Holdings\nbudget: !<tag:yaml.org,2002:int> 1250\naccount: !docs!account {owner: Strategy Office, tier: Enterprise}\ndataSources: !docs!sources\n  - !docs!source {name: Tagged Revenue, path: data/tagged.csv, type: csv}\n---\n# Tagged\nClient: {{client}}\nOwner: {{account.owner}}\nTier: {{account.tier}}\nBudget: {{budget | currency}}\n".to_string(),
        file_path: Some(path_to_string(&doc)),
    });

    assert!(
        response.compiled_markdown.contains("Client: Acme Holdings"),
        "compiled markdown:\n{}\ndiagnostics: {:?}",
        response.compiled_markdown,
        response.diagnostics
    );
    assert!(response
        .compiled_markdown
        .contains("Owner: Strategy Office"));
    assert!(response.compiled_markdown.contains("Tier: Enterprise"));
    assert!(response.compiled_markdown.contains("Budget: $1250.00"));
    assert!(response
        .compiled_markdown
        .contains("## Data Source: Tagged Revenue"));
    assert!(response.html.contains("<td>East</td>"));
    assert!(response.html.contains("<td>100</td>"));
    assert!(response
        .include_graph
        .iter()
        .any(|edge| edge.child.ends_with("data/tagged.csv")));
    assert_eq!(response.metadata["title"], "Tagged Metadata");
    assert_eq!(response.metadata["account"]["owner"], "Strategy Office");
    assert!(response
        .diagnostics
        .iter()
        .all(|diagnostic| !diagnostic.message.contains("front matter")));

    fs::remove_dir_all(root).expect("clean tagged front matter test dir");
}

#[test]
fn calc_blocks_resolve_forward_refs_and_report_cycles() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Calc Graph\nstatus: approved\napprovedBy: QA\n---\n# Calc Graph\n```calc\nprofit = revenue - cost\ncost = 40\nrevenue = 100\ncycle_a = cycle_b + 1\ncycle_b = cycle_a + 1\n```\n\nProfit: {{=profit | round}}\n".to_string(),
            file_path: None,
        });

    assert!(response.compiled_markdown.contains("Profit: 60"));
    assert!(response
        .formula_graph
        .iter()
        .any(|formula| formula.name == "profit" && formula.value == Some(60.0)));
    assert!(response
        .formula_dependency_edges
        .iter()
        .any(|edge| edge.from == "profit" && edge.to == "revenue"));
    assert!(response
        .formula_dependency_edges
        .iter()
        .any(|edge| edge.from == "profit" && edge.to == "cost"));
    assert!(response.formula_graph.iter().any(|formula| {
        formula.name == "cycle_a"
            && formula
                .error
                .as_deref()
                .is_some_and(|error| error.contains("#CYCLE? cycle_a -> cycle_b -> cycle_a"))
    }));
    let cycle_diagnostic = response
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic
                .message
                .contains("#CYCLE? cycle_a -> cycle_b -> cycle_a")
        })
        .expect("cycle diagnostic");
    assert_eq!(cycle_diagnostic.source_file.as_deref(), Some("untitled.md"));
    assert_eq!(cycle_diagnostic.line, Some(11));
    assert_eq!(cycle_diagnostic.column, Some(1));
    assert_eq!(cycle_diagnostic.end_line, Some(11));
    assert_eq!(cycle_diagnostic.end_column, Some(22));
    assert!(cycle_diagnostic
        .related
        .iter()
        .any(|related| related == "formula_name:cycle_a"));
    assert!(cycle_diagnostic
        .related
        .iter()
        .any(|related| related == "dependency:cycle_b"));
}

#[test]
fn inline_formula_diagnostics_include_source_ranges() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Formula Diagnostics\nstatus: approved\napprovedBy: QA\n---\n# Formula Diagnostics\nBad: {{=missing + 1}}\n"
                .to_string(),
            file_path: None,
        });

    let diagnostic = response
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("Inline formula error"))
        .expect("inline formula diagnostic");
    assert_eq!(diagnostic.line, Some(7));
    assert_eq!(diagnostic.column, Some(6));
    assert_eq!(diagnostic.end_line, Some(7));
    assert_eq!(diagnostic.end_column, Some(22));
    assert_eq!(diagnostic.source_file.as_deref(), Some("untitled.md"));
}

#[test]
fn compiler_loads_project_level_variables_without_overriding_front_matter() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-project-vars-test-{unique}"));
    fs::create_dir_all(root.join(".neditor")).expect("create project vars dir");
    fs::write(
        root.join(".neditor").join("variables.yaml"),
        "variables: !docs!vars\n  client: !docs!client Project Client\n  region: !docs!region West\n  owner: !docs!owner Strategy Office\n  review:\n    lead: !docs!person QA Lead\n  profile:\n    name: Project Profile\n    owner: Project Owner\n    address:\n      city: Nairobi\n      country: Kenya\n",
    )
    .expect("write project variables");
    let doc = root.join("docs").join("report.md");
    fs::create_dir_all(doc.parent().expect("doc parent")).expect("create docs dir");
    fs::write(&doc, "# Report").expect("write doc");

    let response = compile(CompileRequest {
            text: "---\ntitle: Project Vars\nstatus: approved\napprovedBy: QA\nclient: Front Matter Client\nprofile:\n  name: Front Profile\n  address:\n    city: Lagos\n---\n# Project Vars\nPrepared for {{client}} in {{region}} by {{owner}} with {{review.lead}}.\nProfile {{profile.name}} in {{profile.address.city}}, {{profile.address.country}} by {{profile.owner}}.\n".to_string(),
            file_path: Some(path_to_string(&doc)),
        });

    assert!(response
        .compiled_markdown
        .contains("Prepared for Front Matter Client in West by Strategy Office with QA Lead."));
    assert!(response
        .compiled_markdown
        .contains("Profile Front Profile in Lagos, Kenya by Project Owner."));
    assert_eq!(response.metadata["client"], "Front Matter Client");
    assert_eq!(response.metadata["region"], "West");
    assert_eq!(response.metadata["review"]["lead"], "QA Lead");
    assert_eq!(response.metadata["profile"]["name"], "Front Profile");
    assert_eq!(response.metadata["profile"]["address"]["city"], "Lagos");
    assert_eq!(response.metadata["profile"]["address"]["country"], "Kenya");
    assert_eq!(response.metadata["profile"]["owner"], "Project Owner");
    fs::remove_dir_all(root).expect("clean project vars test dir");
}

#[test]
fn compiler_loads_front_matter_csv_data_sources() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-data-source-test-{unique}"));
    fs::create_dir_all(root.join("data")).expect("create data dir");
    fs::write(
        root.join("data").join("revenue.csv"),
        "Region,Revenue\n\"East\nCoast\",100\nWest,\"=SUM(B1,80)\"\n",
    )
    .expect("write csv data source");

    let response = compile(CompileRequest {
            text: "---\ntitle: Data Source\nstatus: approved\napprovedBy: QA\ndataSources:\n  - name: Revenue\n    path: data/revenue.csv\n    type: csv\n---\n# Data Source\n".to_string(),
            file_path: Some(path_to_string(&root.join("report.md"))),
        });

    assert!(response
        .compiled_markdown
        .contains("## Data Source: Revenue"));
    assert!(response.html.contains("<td>180</td>"));
    assert!(response.html.contains("East\nCoast"));
    assert!(response
        .include_graph
        .iter()
        .any(|edge| edge.child.ends_with("data/revenue.csv")));
    assert!(response
        .export_manifest
        .include_graph
        .iter()
        .any(|edge| edge.child.ends_with("data/revenue.csv") && edge.depth == 0));
    assert!(response
        .export_manifest
        .included_files
        .iter()
        .any(|file| file.path.ends_with("data/revenue.csv")));
    assert!(response.export_manifest.source_hash.starts_with("sha256:"));
    assert!(response
        .export_manifest
        .included_files
        .iter()
        .all(|file| file.hash.starts_with("sha256:")));
    fs::remove_dir_all(root).expect("clean data source test dir");
}

#[test]
fn compiler_loads_front_matter_json_and_yaml_data_sources() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-structured-data-source-{unique}"));
    fs::create_dir_all(root.join("data")).expect("create structured data dir");
    fs::write(
        root.join("data").join("accounts.json"),
        r#"[{"name":"Acme","arr":1200},{"name":"Globex","arr":900}]"#,
    )
    .expect("write json data source");
    fs::write(
        root.join("data").join("settings.yml"),
        "region: East Africa\nowner: Strategy Office\nthresholds:\n  warn: 0.7\n",
    )
    .expect("write yaml data source");
    fs::write(
        root.join("data").join("targets.tsv"),
        "Metric\tValue\nARR\t300\n",
    )
    .expect("write tsv data source");

    let response = compile(CompileRequest {
        text: "---\ntitle: Structured Data Sources\nstatus: approved\napprovedBy: QA\ndataSources:\n  - name: Accounts\n    path: data/accounts.json\n  - title: Settings\n    file: data/settings.yml\ntsvFiles:\n  - data/targets.tsv\n---\n# Structured Data Sources\n".to_string(),
        file_path: Some(path_to_string(&root.join("report.md"))),
    });

    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error"));
    assert!(response.html.contains("Data Source: Accounts"));
    assert!(response
        .html
        .contains("class=\"transform-table transform-json\""));
    assert!(response.html.contains("<td>Acme</td>"));
    assert!(response.html.contains("<td>1200</td>"));
    assert!(response.html.contains("Data Source: Settings"));
    assert!(response.html.contains("transform-yaml structured-tree"));
    assert!(response.html.contains("<dt>region</dt>"));
    assert!(response.html.contains("Data Source: targets"));
    assert!(response.html.contains("<td>ARR</td>"));
    assert!(response.html.contains("<td>300</td>"));
    assert!(response
        .transform_artifacts
        .iter()
        .any(|artifact| artifact.name == "json" && artifact.source.contains("Globex")));
    assert!(response
        .transform_artifacts
        .iter()
        .any(|artifact| artifact.name == "yaml" && artifact.source.contains("thresholds")));
    assert!(response
        .transform_artifacts
        .iter()
        .any(|artifact| artifact.name == "tsv" && artifact.source.contains("Metric\tValue")));
    assert!(response
        .include_graph
        .iter()
        .any(|edge| edge.child.ends_with("data/accounts.json")));
    assert!(response
        .export_manifest
        .included_files
        .iter()
        .any(|file| file.path.ends_with("data/settings.yml")));

    fs::remove_dir_all(root).expect("clean structured data source test dir");
}

#[test]
fn compiler_reports_malformed_front_matter_data_sources() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-bad-data-source-{unique}"));
    fs::create_dir_all(root.join("data")).expect("create bad data source dir");

    let response = compile(CompileRequest {
        text: "---\ntitle: Bad Data Sources\nstatus: approved\napprovedBy: QA\ndataSources:\n  - name: Missing path\n    type: json\n  - path: data/report.xlsx\n    type: xlsx\n  - path: data/missing.json\n    type: json\n---\n# Bad Data Sources\n".to_string(),
        file_path: Some(path_to_string(&root.join("report.md"))),
    });

    let missing_path = response
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.severity == "warning"
                && diagnostic.message == "Data source entry is missing a path."
        })
        .expect("missing-path data source diagnostic");
    assert!(missing_path
        .related
        .iter()
        .any(|related| related == "data_source_name: Missing path"));
    assert!(missing_path
        .related
        .iter()
        .any(|related| related == "data_source_type: json"));
    let unsupported = response
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.severity == "warning"
                && diagnostic
                    .message
                    .contains("Unsupported data source type 'xlsx'")
        })
        .expect("unsupported data source diagnostic");
    assert!(unsupported
        .related
        .iter()
        .any(|related| related == "data_source_path: data/report.xlsx"));
    assert!(unsupported
        .related
        .iter()
        .any(|related| related == "data_source_type: xlsx"));
    assert!(unsupported
        .suggestion
        .as_deref()
        .is_some_and(|suggestion| suggestion.contains("csv, tsv, json, or yaml")));
    let unreadable = response
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.severity == "error"
                && diagnostic.message.contains("Unable to read data source")
                && diagnostic.message.contains("data/missing.json")
        })
        .expect("unreadable data source diagnostic");
    assert!(unreadable
        .related
        .iter()
        .any(|related| related == "data_source_path: data/missing.json"));
    assert!(unreadable
        .related
        .iter()
        .any(|related| related == "data_source_type: json"));
    assert!(unreadable
        .related
        .iter()
        .any(|related| related.starts_with("resolved_path: ")));
    assert!(response.transform_artifacts.is_empty());

    fs::remove_dir_all(root).expect("clean bad data source test dir");
}

#[test]
fn compiler_blocks_data_sources_outside_document_folder() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-unsafe-data-source-{unique}"));
    fs::create_dir_all(root.join("docs")).expect("create unsafe data source dir");
    fs::write(root.join("secrets.csv"), "Key,Value\nToken,secret\n")
        .expect("write outside data source");
    fs::write(root.join("docs").join("safe.csv"), "Key,Value\nPublic,ok\n")
        .expect("write safe data source");

    let response = compile(CompileRequest {
        text: "---\ntitle: Unsafe Data Sources\nstatus: approved\napprovedBy: QA\ndataSources:\n  - name: Parent escape\n    path: ../secrets.csv\n    type: csv\n  - name: Absolute escape\n    path: /tmp/neditor-secret.csv\n    type: csv\n  - name: Safe data\n    path: safe.csv\n    type: csv\n---\n# Unsafe Data Sources\n".to_string(),
        file_path: Some(path_to_string(&root.join("docs").join("report.md"))),
    });

    let path_errors = response
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.severity == "error"
                && diagnostic
                    .message
                    .contains("Data source path must stay relative")
        })
        .collect::<Vec<_>>();
    assert_eq!(path_errors.len(), 2, "{:#?}", response.diagnostics);
    assert!(path_errors
        .iter()
        .any(|diagnostic| diagnostic.message.contains("../secrets.csv")));
    assert!(path_errors
        .iter()
        .any(|diagnostic| diagnostic.message.contains("/tmp/neditor-secret.csv")));
    assert!(path_errors.iter().any(|diagnostic| {
        diagnostic
            .related
            .iter()
            .any(|related| related == "data_source_name: Parent escape")
            && diagnostic
                .related
                .iter()
                .any(|related| related == "data_source_path: ../secrets.csv")
    }));
    assert!(path_errors.iter().any(|diagnostic| {
        diagnostic
            .related
            .iter()
            .any(|related| related == "data_source_name: Absolute escape")
            && diagnostic
                .related
                .iter()
                .any(|related| related == "data_source_path: /tmp/neditor-secret.csv")
    }));
    assert!(response.html.contains("Data Source: Safe data"));
    assert!(response.html.contains("<td>Public</td>"));
    assert!(!response.html.contains("Token"));
    assert!(response
        .include_graph
        .iter()
        .any(|edge| edge.child.ends_with("docs/safe.csv")));
    assert!(!response
        .include_graph
        .iter()
        .any(|edge| edge.child.ends_with("secrets.csv")));

    fs::remove_dir_all(root).expect("clean unsafe data source test dir");
}

#[cfg(unix)]
#[test]
fn compiler_blocks_symlinked_data_sources_outside_document_folder() {
    use std::os::unix::fs::symlink;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-symlink-data-source-{unique}"));
    fs::create_dir_all(root.join("docs").join("data")).expect("create symlink data source dir");
    fs::write(root.join("secrets.csv"), "Key,Value\nToken,secret\n")
        .expect("write symlink target data source");
    symlink(
        root.join("secrets.csv"),
        root.join("docs").join("data").join("outside.csv"),
    )
    .expect("create data source symlink");

    let response = compile(CompileRequest {
        text: "---\ntitle: Symlink Data Source\nstatus: approved\napprovedBy: QA\ndataSources:\n  - name: Linked outside\n    path: data/outside.csv\n    type: csv\n---\n# Symlink Data Source\n".to_string(),
        file_path: Some(path_to_string(&root.join("docs").join("report.md"))),
    });

    assert!(response.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == "error"
            && diagnostic
                .message
                .contains("Data source path must stay relative")
            && diagnostic.message.contains("data/outside.csv")
    }));
    assert!(!response.html.contains("Token"));
    assert!(response.include_graph.is_empty());

    fs::remove_dir_all(root).expect("clean symlink data source test dir");
}

#[test]
fn compiler_honors_toc_depth_and_numbering() {
    let response = compile(CompileRequest {
            text: "---\ntitle: TOC\nstatus: approved\napprovedBy: QA\ntoc: true\ntocDepth: 2\ntocNumbered: true\n---\n# Alpha\n## Beta\n### Gamma\n## Delta\n".to_string(),
            file_path: None,
        });

    assert!(response.compiled_markdown.contains("- [1 Alpha](#alpha)"));
    assert!(response.compiled_markdown.contains("  - [1.1 Beta](#beta)"));
    assert!(response
        .compiled_markdown
        .contains("  - [1.2 Delta](#delta)"));
    assert!(!response.compiled_markdown.contains("[1.1.1 Gamma](#gamma)"));
    let docx = render_docx_bytes(&response, &json!({})).expect("docx bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains(r#"w:instr="TOC \o &quot;1-2&quot; \h \z \u""#));
    assert!(!docx_document.contains("#alpha"));
}

#[test]
fn compiler_generates_lists_of_figures_and_tables() {
    let response = compile(CompileRequest {
        text: [
            "---",
            "title: Caption Lists",
            "status: approved",
            "approvedBy: QA",
            "---",
            "# Caption Lists",
            "",
            "[LIST_OF_FIGURES]",
            "",
            "[LIST_OF_TABLES]",
            "",
            "![Architecture](architecture.svg){#fig:architecture caption=\"System architecture\"}",
            "",
            "![Fallback alt](fallback.svg){#fig:fallback}",
            "",
            "Table: Revenue by region {#tbl:revenue}",
            "| Region | Revenue |",
            "| --- | ---: |",
            "| East | 120 |",
            "",
            "```md",
            "![Example](example.svg){#fig:example caption=\"Example only\"}",
            "Table: Example table {#tbl:example}",
            "| A | B |",
            "| --- | --- |",
            "```",
        ]
        .join("\n"),
        file_path: None,
    });

    assert!(response
        .compiled_markdown
        .contains("## List of Figures\n\n- [Figure 1: System architecture](#fig:architecture)"));
    assert!(response
        .compiled_markdown
        .contains("- [Figure 2: Fallback alt](#fig:fallback)"));
    assert!(response
        .compiled_markdown
        .contains("## List of Tables\n\n- [Table 1: Revenue by region](#tbl:revenue)"));
    assert!(!response
        .compiled_markdown
        .contains("Figure 3: Example only"));
    assert!(!response
        .compiled_markdown
        .contains("Table 2: Example table"));
    assert!(response.html.contains("List of Figures"));
    assert!(response.html.contains("Figure 1: System architecture"));
    assert!(response.html.contains("Table 1: Revenue by region"));

    let docx = render_docx_bytes(&response, &json!({})).expect("docx bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("List of Figures"));
    assert!(docx_document.contains("Figure 1: System architecture"));
    assert!(docx_document.contains("List of Tables"));
    assert!(docx_document.contains("Table 1: Revenue by region"));
}

#[test]
fn compiler_adds_glossary_hover_terms_to_preview_html() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Glossary Hover\nstatus: approved\napprovedBy: QA\n---\n# Glossary Hover\nARR informs planning.\n\n```glossary\nARR: Annual recurring revenue.\n```\n".to_string(),
            file_path: None,
        });

    assert!(response.html.contains("class=\"glossary-term\""));
    assert!(response
        .html
        .contains("title=\"Annual recurring revenue.\""));
    assert!(response.html.contains(">ARR</span> informs planning"));
}

#[test]
fn compiler_generates_glossary_sections_from_marker_and_metadata() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Glossary Section\nstatus: approved\napprovedBy: QA\n---\n# Glossary Section\nARR informs planning.\n\n[GLOSSARY]\n\n```glossary\nARR: Annual recurring revenue.\nCAC: Customer acquisition cost.\n```\n".to_string(),
            file_path: None,
        });

    assert!(response
        .compiled_markdown
        .contains("## Glossary\n\n- **ARR**: Annual recurring revenue."));
    assert!(response
        .compiled_markdown
        .contains("- **CAC**: Customer acquisition cost."));
    assert!(!response.compiled_markdown.contains("[GLOSSARY]"));
    assert!(response.html.contains("<h2 id=\"glossary\">Glossary</h2>"));
    assert!(response.html.contains("Annual recurring revenue."));
    assert!(response.html.contains("class=\"glossary-term\""));

    let docx = render_docx_bytes(&response, &json!({})).expect("docx glossary bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("Glossary"));
    assert!(docx_document.contains("Customer acquisition cost."));

    let metadata_response = compile(CompileRequest {
            text: "---\ntitle: Front Matter Glossary\nstatus: approved\napprovedBy: QA\nglossarySection: true\n---\n# Front Matter Glossary\n\n```glossary\nARR: Annual recurring revenue.\n```\n".to_string(),
            file_path: None,
        });

    assert!(metadata_response
        .compiled_markdown
        .starts_with("## Glossary\n\n- **ARR**: Annual recurring revenue."));
}

#[test]
fn compiler_preserves_figure_float_semantics() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Floating Figure\nstatus: approved\napprovedBy: QA\n---\n# Floating Figure\n![Diagram](data:image/svg+xml;base64,PHN2Zy8+){#fig:float caption=\"Floating diagram\" float=\"right\"}\n".to_string(),
            file_path: None,
        });

    assert!(response.html.contains("figure-float-right"));
    assert!(response.html.contains("data-float=\"right\""));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Figure {
                id,
                caption,
                float,
                ..
            } if id.as_deref() == Some("fig:float")
                && caption.as_deref() == Some("Floating diagram")
                && float.as_deref() == Some("right")
        )
    }));

    let exported = export::export_text(&response, &json!({}));
    assert!(exported.contains("float=right"));

    let full_html = render_full_html(&response, &json!({}));
    assert!(full_html.contains("figure-float-right"));
    assert!(full_html.contains("float:right"));

    let docx = render_docx_bytes(&response, &json!({})).expect("docx bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("float=right"));
    assert!(docx_document.contains(r#"<w:jc w:val="right"/>"#));

    let pptx = render_pptx_bytes(&response, &json!({})).expect("pptx bytes");
    let floating_slide = zip_entry_texts_with_prefix(&pptx, "ppt/slides/")
        .into_iter()
        .find(|slide| slide.contains(r#"r:embed="rIdImage1""#))
        .expect("floating figure slide");
    assert!(floating_slide.contains(r#"<a:off x="5029200""#));

    let pdf = render_pdf_bytes(&response, &json!({}));
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("287 627 240 135 re S"));
}

#[test]
fn compiler_generates_linked_index_with_exclusions_and_proper_terms() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Index\nstatus: approved\napprovedBy: QA\nindexExclude:\n  - internal draft\n---\n# Market Analysis\nAcme Strategy appears here. **Working Capital** matters.\n\n## Follow Up\nAcme Strategy returns. Internal Draft should stay out. Working capital{#index:Liquidity} marker.\n\n[INDEX]\n".to_string(),
            file_path: None,
        });

    assert!(response
        .index_terms
        .iter()
        .any(|term| term == "Acme Strategy"));
    assert!(response.index_terms.iter().any(|term| term == "Liquidity"));
    assert!(response
        .index_terms
        .iter()
        .any(|term| term == "Working Capital"));
    assert!(!response
        .index_terms
        .iter()
        .any(|term| term == "Internal Draft"));
    assert!(response.html.contains("href=\"#market-analysis\""));
    assert!(response.html.contains("Acme Strategy"));
    assert!(response.html.contains("Liquidity"));
    assert!(!response.html.contains("{#index:Liquidity}"));
}

#[test]
fn compiler_generates_index_from_front_matter_without_marker() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Front Matter Index\nstatus: approved\napprovedBy: QA\nindex:\n  enabled: true\n  exclude:\n    - secret plan\n---\n# Market Analysis\nAcme Strategy appears here. **Working Capital** matters.\n\n## Follow Up\nAcme Strategy returns. Secret Plan should stay out. Working capital{#index:Liquidity} marker.\n".to_string(),
            file_path: None,
        });

    assert!(response.compiled_markdown.starts_with("## Index\n\n"));
    assert!(response
        .compiled_markdown
        .contains("- [Acme Strategy](#market-analysis)"));
    assert!(response
        .compiled_markdown
        .contains("- [Liquidity](#follow-up)"));
    assert!(response
        .compiled_markdown
        .contains("- [Working Capital](#market-analysis)"));
    assert!(!response
        .index_terms
        .iter()
        .any(|term| term == "Secret Plan"));
    assert!(!response.compiled_markdown.contains("{#index:Liquidity}"));
    assert!(response.html.contains("<h2 id=\"index\">Index</h2>"));
}

#[test]
fn compiler_parses_review_comment_metadata() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Review\nstatus: approved\napprovedBy: QA\n---\n# Review\n<!-- comment: unresolved | author: Dana | at: 2026-05-18T10:00:00Z | Clarify the risk note. -->\n<!-- change: author: Dana | at: 2026-05-18T11:00:00Z | Updated the risk note. -->\n".to_string(),
            file_path: None,
        });
    let comment = response.semantic.comments.first().expect("review comment");
    let change_note = response.semantic.change_notes.first().expect("change note");

    assert_eq!(comment.state, "unresolved");
    assert_eq!(comment.author, "Dana");
    assert_eq!(comment.created_at, "2026-05-18T10:00:00Z");
    assert_eq!(comment.text, "Clarify the risk note.");
    assert_eq!(change_note.author, "Dana");
    assert_eq!(change_note.created_at, "2026-05-18T11:00:00Z");
    assert_eq!(change_note.text, "Updated the risk note.");
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::ReviewComment { comment, .. }
                if comment.author == "Dana"
                    && comment.state == "unresolved"
                    && comment.text == "Clarify the risk note."
        )
    }));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::ChangeNote { note, .. }
                if note.author == "Dana" && note.text == "Updated the risk note."
        )
    }));
    let unresolved_comment_diagnostic = response
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("unresolved review comments"))
        .expect("unresolved comment diagnostic");
    assert_eq!(unresolved_comment_diagnostic.severity, "error");
    assert_eq!(unresolved_comment_diagnostic.line, Some(7));
    assert_eq!(
        unresolved_comment_diagnostic.source_file.as_deref(),
        Some("untitled.md")
    );
    assert!(unresolved_comment_diagnostic
        .related
        .iter()
        .any(|related| related.contains("Clarify the risk note")));
}

#[test]
fn compiler_reports_missing_include_without_panicking() {
    let response = compile(CompileRequest {
        text: "!include missing/chapter.md\n".to_string(),
        file_path: None,
    });

    let diagnostic = response
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.severity == "error" && diagnostic.message.contains("Missing include file")
        })
        .expect("missing include diagnostic");
    assert!(diagnostic
        .related
        .iter()
        .any(|related| related.contains("missing/chapter.md")));
}

#[test]
fn compiler_reports_circular_and_too_deep_includes() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-include-guards-{unique}"));
    let cycle_dir = root.join("cycle");
    let depth_dir = root.join("depth");
    fs::create_dir_all(&cycle_dir).expect("create cycle dir");
    fs::create_dir_all(&depth_dir).expect("create depth dir");

    fs::write(cycle_dir.join("a.md"), "# A\n!include b.md\n").expect("write cycle a");
    fs::write(cycle_dir.join("b.md"), "# B\n!include a.md\n").expect("write cycle b");
    let cycle_root = root.join("cycle-root.md");
    fs::write(&cycle_root, "# Root\n!include cycle/a.md\n").expect("write cycle root");
    let cycle_response = compile(CompileRequest {
        text: fs::read_to_string(&cycle_root).expect("read cycle root"),
        file_path: Some(path_to_string(&cycle_root)),
    });

    let cycle_diagnostic = cycle_response
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message == "Circular include detected.")
        .expect("circular include diagnostic");
    assert_eq!(cycle_diagnostic.severity, "error");
    assert_eq!(
        cycle_diagnostic.source_file.as_deref(),
        Some(path_to_string(&cycle_dir.join("b.md")).as_str())
    );
    assert!(cycle_diagnostic
        .related
        .iter()
        .any(|related| related.contains("Include target:")));
    assert!(cycle_response
        .include_graph
        .iter()
        .any(|edge| edge.depth == 1 && edge.child.ends_with("cycle/a.md")));
    assert!(cycle_response
        .include_graph
        .iter()
        .any(|edge| edge.depth == 2 && edge.child.ends_with("cycle/b.md")));

    for index in 0..18 {
        let next = index + 1;
        let body = if index < 17 {
            format!("# Depth {index}\n!include include-{next:02}.md\n")
        } else {
            format!("# Depth {index}\n")
        };
        fs::write(depth_dir.join(format!("include-{index:02}.md")), body)
            .expect("write depth include");
    }
    let depth_root = root.join("depth-root.md");
    fs::write(&depth_root, "# Root\n!include depth/include-00.md\n").expect("write depth root");
    let depth_response = compile(CompileRequest {
        text: fs::read_to_string(&depth_root).expect("read depth root"),
        file_path: Some(path_to_string(&depth_root)),
    });

    let depth_diagnostic = depth_response
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message == "Maximum include depth exceeded.")
        .expect("maximum include depth diagnostic");
    assert_eq!(depth_diagnostic.severity, "error");
    assert!(depth_diagnostic
        .suggestion
        .as_deref()
        .is_some_and(|suggestion| suggestion.contains("Reduce nested include directives")));
    assert!(depth_response
        .include_graph
        .iter()
        .any(|edge| edge.depth == 17));
    assert!(!depth_response.compiled_markdown.contains("Depth 17"));

    fs::remove_dir_all(root).expect("clean include guard test dir");
}

#[test]
fn compiler_reports_unreadable_include_targets_with_context() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-unreadable-include-{unique}"));
    let include_dir = root.join("chapters").join("directory.md");
    fs::create_dir_all(&include_dir).expect("create directory include target");
    let root_doc = root.join("root.md");
    fs::write(&root_doc, "# Root\n!include chapters/directory.md\n")
        .expect("write unreadable include root");

    let response = compile(CompileRequest {
        text: fs::read_to_string(&root_doc).expect("read unreadable include root"),
        file_path: Some(path_to_string(&root_doc)),
    });

    let diagnostic = response
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic
                .message
                .starts_with("Unable to read include file:")
        })
        .expect("unreadable include diagnostic");
    assert_eq!(diagnostic.severity, "error");
    assert_eq!(diagnostic.line, Some(2));
    assert_eq!(
        diagnostic.source_file.as_deref(),
        Some(path_to_string(&root_doc).as_str())
    );
    assert!(diagnostic
        .suggestion
        .as_deref()
        .is_some_and(|suggestion| suggestion.contains("Check file permissions")));
    assert!(diagnostic
        .related
        .iter()
        .any(|related| related == "Include target: chapters/directory.md"));
    assert!(diagnostic
        .related
        .iter()
        .any(|related| related.contains("Resolved path:")));

    fs::remove_dir_all(root).expect("clean unreadable include test dir");
}

#[test]
fn compiler_reports_broken_local_markdown_links() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-link-test-{unique}"));
    fs::create_dir_all(root.join("docs")).expect("create link test dir");
    fs::write(root.join("docs").join("existing.md"), "# Existing").expect("write linked doc");

    let response = compile(CompileRequest {
            text: "---\ntitle: Links\nstatus: approved\napprovedBy: QA\nbrand:\n  logo: docs/missing-logo.svg\n---\n# Links\nRead [existing](docs/existing.md), [missing](docs/missing.md), [section](#links), and [web](https://example.com).\n![Missing image](docs/missing.png)\n\n```md\n[example link](docs/code-missing.md)\n![Example image](docs/code-missing.png)\n```\n".to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
        });
    let root_doc = path_to_string(&root.join("root.md"));

    let broken_link = response
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("Broken link path"))
        .expect("broken link diagnostic");
    assert_eq!(broken_link.line, Some(9));
    assert!(broken_link.column.is_some());
    assert!(broken_link.end_column > broken_link.column);
    assert_eq!(broken_link.source_file.as_deref(), Some(root_doc.as_str()));
    assert!(broken_link
        .related
        .iter()
        .any(|related| related.contains("docs/missing.md")));
    assert_eq!(
        response
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.message.contains("Broken link path"))
            .count(),
        1
    );
    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("code-missing")));
    let broken_image = response
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("Broken image path"))
        .expect("broken image diagnostic");
    assert_eq!(broken_image.line, Some(10));
    assert!(broken_image.column.is_some());
    assert!(broken_image.end_column > broken_image.column);
    assert_eq!(broken_image.source_file.as_deref(), Some(root_doc.as_str()));
    assert!(broken_image
        .related
        .iter()
        .any(|related| related.contains("docs/missing.png")));
    assert!(response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Broken logo path")));
    fs::remove_dir_all(root).expect("clean link test dir");
}
