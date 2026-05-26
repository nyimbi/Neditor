use super::*;

#[test]
fn compiler_summarizes_markdown_tables() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Tables\nstatus: approved\napprovedBy: QA\n---\n# Tables\nTable: Revenue by region {#tbl:revenue}\n| Region | Revenue |\n| --- | ---: |\n| East | 100 |\n| West | =SUM(B1,80) |\n| Total | =SUM(B1:B2) |\n\nSee {@tbl:revenue}.\n".to_string(),
            file_path: None,
        });

    assert!(response.compiled_markdown.contains("| West | 180 |"));
    assert!(response.compiled_markdown.contains("| Total | 280 |"));
    assert!(response.html.contains(">280</td>"));
    assert_eq!(response.semantic.tables, 1);
    assert_eq!(response.semantic.table_summaries[0].rows, 3);
    assert_eq!(
        response.semantic.table_summaries[0]
            .numeric_columns
            .get("Revenue"),
        Some(&560.0)
    );
    assert!(response
        .semantic
        .cross_references
        .iter()
        .any(|reference| reference.key == "tbl:revenue" && reference.resolved));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Table { id, caption, .. }
                if id.as_deref() == Some("tbl:revenue")
                    && caption.as_deref() == Some("Revenue by region")
        )
    }));
}

#[test]
fn csv_and_tsv_transforms_evaluate_table_formula_cells() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Formula Tables\nstatus: approved\napprovedBy: QA\n---\n# Formula Tables\n```csv\nMetric,Value\nTotal,=10+15\nRounded,=ROUND(2.6)\nRange,=SUM(B1:B2)\n```\n\n```tsv\nMetric\tValue\nAbs\t=ABS(-5)\nSum\t=SUM(2,3)\nProfitable\t=IF(10>5,1,0)\nEqual\t=IF(ROUND(2.6)=3,1,0)\nRange\t=SUM(B1:B4)\n```\n".to_string(),
            file_path: None,
        });

    assert!(response.html.contains("<td>25</td>"));
    assert!(response.html.contains("<td>3</td>"));
    assert!(response.html.contains("<td>1</td>"));
    assert!(response.html.contains("<td>5</td>"));
    assert!(response.html.contains("<td>28</td>"));
    assert!(response.html.contains("<td>12</td>"));
    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Table formula error")));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Table { headers, rows, .. }
                if headers == &vec!["Metric".to_string(), "Value".to_string()]
                    && rows.iter().any(|row| row == &vec![
                        "Total".to_string(),
                        "25".to_string()
                    ])
        )
    }));

    let options = json!({});
    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("<w:tbl>"));
    assert!(docx_document.contains(">25<"));
    assert!(!docx_document.contains("```csv"));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
    let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
    assert!(pptx_slide.contains("<a:tbl>"));
    assert!(pptx_slide.contains("25"));
    assert!(!pptx_slide.contains("```csv"));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains(" re S"));
    assert!(pdf_text.contains("(25) Tj"));
}

#[test]
fn csv_formula_diagnostics_report_absolute_fence_source_lines() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Bad CSV Formula\nstatus: draft\n---\n# Bad CSV Formula\n```csv\nMetric,Value\nBad,=UNKNOWN(1)\n```\n"
            .to_string(),
        file_path: None,
    });

    let diagnostic = response
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("Table formula error"))
        .expect("csv formula diagnostic");
    assert_eq!(diagnostic.source_file.as_deref(), Some("untitled.md"));
    assert_eq!(diagnostic.line, Some(8));
    assert_eq!(diagnostic.end_line, Some(8));
    assert_eq!(diagnostic.column, Some(1));
    assert_eq!(diagnostic.end_column, Some(4));
    assert!(diagnostic
        .suggestion
        .as_deref()
        .is_some_and(|suggestion| suggestion.contains("CSV/TSV cells")));
    assert!(diagnostic
        .related
        .iter()
        .any(|related| related == "transform: csv"));
    assert!(diagnostic
        .related
        .iter()
        .any(|related| related == "source range: 6-9"));

    let artifact_diagnostic = response
        .transform_artifacts
        .iter()
        .find(|artifact| artifact.name == "csv")
        .and_then(|artifact| artifact.diagnostics.first())
        .expect("csv artifact diagnostic");
    assert_eq!(artifact_diagnostic.line, Some(8));
    assert_eq!(artifact_diagnostic.end_line, Some(8));
    assert_eq!(artifact_diagnostic.column, Some(1));
    assert_eq!(artifact_diagnostic.end_column, Some(4));
}

#[test]
fn spreadsheet_table_import_export_round_trips_csv_and_xlsx() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-spreadsheet-table-{unique}"));
    fs::create_dir_all(&root).expect("test root");
    let csv_path = root.join("input.csv");
    fs::write(&csv_path, "Region,Revenue\nEast,120\nWest,95\n").expect("csv input");

    let imported = crate::data_exchange::import_spreadsheet_table(
        crate::data_exchange::ImportSpreadsheetTableRequest {
            path: path_to_string(&csv_path),
        },
    )
    .expect("import csv");
    assert_eq!(imported.columns, 2);
    assert_eq!(imported.rows, 2);
    assert!(imported.markdown.contains("| Region | Revenue |"));

    let xlsx_path = root.join("table.xlsx");
    let exported = crate::data_exchange::export_markdown_tables(
        crate::data_exchange::ExportMarkdownTablesRequest {
            markdown: imported.markdown.clone(),
            output_path: path_to_string(&xlsx_path),
            format: "xlsx".to_string(),
            table_index: None,
        },
    )
    .expect("export xlsx");
    assert_eq!(exported.exported_tables, 1);
    assert!(xlsx_path.is_file());

    let xlsx_import = crate::data_exchange::import_spreadsheet_table(
        crate::data_exchange::ImportSpreadsheetTableRequest {
            path: path_to_string(&xlsx_path),
        },
    )
    .expect("import xlsx");
    assert!(xlsx_import.markdown.contains("| East | 120 |"));

    let csv_output = root.join("table.csv");
    let csv_export = crate::data_exchange::export_markdown_tables(
        crate::data_exchange::ExportMarkdownTablesRequest {
            markdown: xlsx_import.markdown,
            output_path: path_to_string(&csv_output),
            format: "csv".to_string(),
            table_index: Some(0),
        },
    )
    .expect("export csv");
    assert_eq!(csv_export.rows, 2);
    assert!(fs::read_to_string(csv_output)
        .expect("csv output")
        .contains("Region,Revenue"));
}

#[test]
fn sql_transform_requires_read_only_trusted_queries() {
    let mutation = run_transform("sql".to_string(), "DELETE FROM accounts".to_string())
        .expect("sql transform artifact");
    assert!(mutation.html.contains("read-only SELECT"));
    assert!(mutation
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error"));

    let missing_trust = compile_with_options(
        CompileRequest {
            text: "# SQL\n```sql database=\"/tmp/neditor-test.sqlite\"\nSELECT 1;\n```\n"
                .to_string(),
            file_path: None,
        },
        &json!({
            "transformEnginePaths": {"sql": "/usr/bin/sqlite3"},
            "trustedTransformEngines": {"sql": false}
        }),
    );
    assert!(missing_trust
        .html
        .contains("requires explicit trust before NEditor runs sqlite3"));
}

#[test]
fn table_formulas_resolve_forward_refs_and_report_cycles() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Formula Cycles\nstatus: approved\napprovedBy: QA\n---\n# Formula Cycles\n| Metric | Value |\n| --- | ---: |\n| Forward | =B2 |\n| Source | 42 |\n| Cycle A | =B4 |\n| Cycle B | =B3 |\n".to_string(),
            file_path: None,
        });

    assert!(response.compiled_markdown.contains("| Forward | 42 |"));
    assert!(response.compiled_markdown.contains("| Cycle A | #ERROR |"));
    assert!(response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("#CYCLE? B3 -> B4 -> B3")));
}

#[test]
fn table_formulas_reference_named_tables() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Named Tables\nstatus: approved\napprovedBy: QA\n---\n# Named Tables\nTable: Revenue {#tbl:revenue}\n| Region | Revenue |\n| --- | ---: |\n| East | 100 |\n| West | 180 |\n| Total | =SUM(B1:B2) |\n\nTable: Summary {#tbl:summary}\n| Metric | Value |\n| --- | ---: |\n| Revenue rollup | =SUM(tbl:revenue!B1:B3) |\n| Reported total | =revenue!B3 |\n".to_string(),
            file_path: None,
        });

    assert!(response
        .compiled_markdown
        .contains("| Revenue rollup | 560 |"));
    assert!(response
        .compiled_markdown
        .contains("| Reported total | 280 |"));
    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("#NAME?")));
}

#[test]
fn markdown_tables_preserve_escaped_pipes_across_ast_and_formulas() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Escaped Tables\nstatus: approved\napprovedBy: QA\n---\n# Escaped Tables\nTable: Pricing notes {#tbl:pricing}\n| Product | Notes | Value |\n| --- | --- | ---: |\n| A \\| B | keep literal pipe | 10 |\n| Total | formula keeps source readable | =SUM(C1,5) |\n".to_string(),
            file_path: None,
        });

    assert!(response.compiled_markdown.contains("| A \\| B |"));
    assert!(response
        .compiled_markdown
        .contains("| Total | formula keeps source readable | 15 |"));
    assert_eq!(response.semantic.table_summaries[0].columns.len(), 3);
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Table { headers, rows, .. }
                if headers == &vec![
                    "Product".to_string(),
                    "Notes".to_string(),
                    "Value".to_string()
                ]
                && rows.iter().any(|row| row == &vec![
                    "A | B".to_string(),
                    "keep literal pipe".to_string(),
                    "10".to_string()
                ])
                && rows.iter().any(|row| row == &vec![
                    "Total".to_string(),
                    "formula keeps source readable".to_string(),
                    "15".to_string()
                ])
        )
    }));
}

#[test]
fn edited_table_fixture_exports_to_all_packages() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Edited Table Export\nstatus: approved\napprovedBy: QA\n---\n# Edited Table Export\nTable: Edited revenue {#tbl:edited}\n| Region | Revenue | Margin |\n| --- | ---: | ---: |\n| East | $125,000 | 42% |\n| West | $98,000 | 38% |\n| Total | =SUM(B1:B2) | =AVG(C1:C2) |\n".to_string(),
            file_path: None,
        });

    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error"));
    assert!(response
        .compiled_markdown
        .contains("| Total | 223000 | 40 |"));
    assert_eq!(response.semantic.tables, 1);
    assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Table {
                    id,
                    caption,
                    headers,
                    rows,
                    ..
                } if id.as_deref() == Some("tbl:edited")
                    && caption.as_deref() == Some("Edited revenue")
                    && headers == &vec!["Region".to_string(), "Revenue".to_string(), "Margin".to_string()]
                    && rows.iter().any(|row| row == &vec![
                        "Total".to_string(),
                        "223000".to_string(),
                        "40".to_string()
                    ])
            )
        }));

    let options = json!({});
    let docx = render_docx_bytes(&response, &options).expect("docx edited table");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("Edited revenue"));
    assert!(docx_document.contains("223000"));
    assert!(docx_document.contains(r#"<w:jc w:val="right"/>"#));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx edited table");
    let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
    assert!(pptx_slide.contains("Edited revenue"));
    assert!(pptx_slide.contains("<a:tbl>"));
    assert!(pptx_slide.contains("<a:t>223000</a:t>"));
    assert!(pptx_slide.contains(r#"<a:pPr algn="r"/>"#));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("Edited revenue"));
    assert!(pdf_text.contains("(223000) Tj"));
    assert!(pdf_text.contains("(40) Tj"));

    let bundle = render_markdown_bundle_bytes(&response, &response.export_manifest)
        .expect("edited table bundle");
    let bundled_ast = zip_entry_text(&bundle, "document-ast.json");
    assert!(bundled_ast.contains("\"kind\": \"table\""));
    assert!(bundled_ast.contains("Edited revenue"));
    assert!(bundled_ast.contains("223000"));
}

#[test]
fn edited_table_permutation_exports_alignment_escapes_and_formula_rows() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Edited Table Permutations\nstatus: approved\napprovedBy: QA\n---\n# Edited Table Permutations\nTable: Scenario grid {#tbl:scenario}\n| Scenario | Owner | Score | Status |\n| :--- | :---: | ---: | --- |\n| Base \\| Case | Finance | $1,200.50 | Ready |\n| Stretch | Ops | 75% | Watch |\n| Floor | Risk | 20 | Hold |\n| Min | Summary | =MIN(C1:C3) | Formula |\n| Max | Summary | =MAX(C1:C3) | Formula |\n| Count | Summary | =COUNT(C1:C3) | Formula |\n".to_string(),
            file_path: None,
        });

    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error"));
    assert!(response.compiled_markdown.contains("Base \\| Case"));
    assert!(response
        .compiled_markdown
        .contains("| Min | Summary | 20 | Formula |"));
    assert!(
        response
            .compiled_markdown
            .contains("| Max | Summary | 1200 | Formula |"),
        "{}",
        response.compiled_markdown
    );
    assert!(response
        .compiled_markdown
        .contains("| Count | Summary | 3 | Formula |"));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Table {
                id,
                caption,
                headers,
                alignments,
                rows,
                ..
            } if id.as_deref() == Some("tbl:scenario")
                && caption.as_deref() == Some("Scenario grid")
                && headers == &vec![
                    "Scenario".to_string(),
                    "Owner".to_string(),
                    "Score".to_string(),
                    "Status".to_string()
                ]
                && alignments == &vec![
                    "left".to_string(),
                    "center".to_string(),
                    "right".to_string(),
                    "left".to_string()
                ]
                && rows.iter().any(|row| row == &vec![
                    "Base | Case".to_string(),
                    "Finance".to_string(),
                    "$1,200.50".to_string(),
                    "Ready".to_string()
                ])
                && rows.iter().any(|row| row == &vec![
                    "Max".to_string(),
                    "Summary".to_string(),
                    "1200".to_string(),
                    "Formula".to_string()
                ])
        )
    }));

    let options = json!({});
    let docx = render_docx_bytes(&response, &options).expect("docx edited table permutation");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("Scenario grid"));
    assert!(docx_document.contains("Base | Case"));
    assert!(docx_document.contains("1200"));
    assert!(docx_document.contains(r#"<w:jc w:val="center"/>"#));
    assert!(docx_document.contains(r#"<w:jc w:val="right"/>"#));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx edited table permutation");
    let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
    assert!(pptx_slide.contains("Scenario grid"));
    assert!(pptx_slide.contains("<a:t>Base | Case</a:t>"));
    assert!(pptx_slide.contains("<a:t>1200</a:t>"));
    assert!(pptx_slide.contains(r#"<a:pPr algn="ctr"/>"#));
    assert!(pptx_slide.contains(r#"<a:pPr algn="r"/>"#));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("Scenario grid"));
    assert!(pdf_text.contains("(Base | Case) Tj"));
    assert!(pdf_text.contains("(1200) Tj"));

    let bundle = render_markdown_bundle_bytes(&response, &response.export_manifest)
        .expect("edited table permutation bundle");
    let bundled_text = zip_entry_text(&bundle, "document.txt");
    assert!(bundled_text.contains("Table: tbl:scenario: Scenario grid"));
    assert!(bundled_text.contains("| Scenario | Owner | Score | Status |"));
    assert!(bundled_text.contains("| --- | :---: | ---: | --- |"));
    assert!(bundled_text.contains("| Base \\| Case | Finance | $1,200.50 | Ready |"));
    assert!(bundled_text.contains("| Max | Summary | 1200 | Formula |"));
    let bundled_ast = zip_entry_text(&bundle, "document-ast.json");
    assert!(bundled_ast.contains("\"id\": \"tbl:scenario\""));
    assert!(bundled_ast.contains("Base | Case"));
    assert!(bundled_ast.contains("1200"));
}

#[test]
fn merged_table_cells_flow_through_semantic_exports() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Merged Tables\nstatus: approved\napprovedBy: QA\n---\n# Merged Tables\nTable: Merged plan {#tbl:merged}\n| Phase {colspan=2} | Owner |\n| --- | --- | --- |\n| Discovery {rowspan=2} | Scope | PM |\n| Detail | Analyst |\n"
            .to_string(),
        file_path: None,
    });

    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error"));
    let table = response
        .document_ast
        .blocks
        .iter()
        .find_map(|block| {
            if let DocumentBlock::Table {
                id,
                header_cells,
                row_cells,
                ..
            } = block
            {
                (id.as_deref() == Some("tbl:merged")).then_some((header_cells, row_cells))
            } else {
                None
            }
        })
        .expect("semantic merged table");
    assert_eq!(table.0[0].text, "Phase");
    assert_eq!(table.0[0].colspan, 2);
    assert!(table.0[1].covered);
    assert_eq!(table.1[0][0].text, "Discovery");
    assert_eq!(table.1[0][0].rowspan, 2);
    assert!(table.1[1][0].covered);
    assert!(table.1[1][0].continues_rowspan);

    let options = json!({});
    let docx = render_docx_bytes(&response, &options).expect("docx merged table");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains(r#"<w:gridSpan w:val="2"/>"#));
    assert!(docx_document.contains(r#"<w:vMerge w:val="restart"/>"#));
    assert!(docx_document.contains("<w:vMerge/>"));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx merged table");
    let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
    assert!(pptx_slide.contains(r#"<a:tc gridSpan="2""#));
    assert!(pptx_slide.contains(r#"rowSpan="2""#));
    assert!(pptx_slide.contains(r#"vMerge="1""#));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("(Phase) Tj"));
    assert!(pdf_text.contains("(Discovery) Tj"));

    let bundle = render_markdown_bundle_bytes(&response, &response.export_manifest)
        .expect("merged table bundle");
    let bundled_text = zip_entry_text(&bundle, "document.txt");
    assert!(bundled_text.contains("| Phase {colspan=2} |  | Owner |"));
    assert!(bundled_text.contains("| Discovery {rowspan=2} | Scope | PM |"));
    let bundled_ast = zip_entry_text(&bundle, "document-ast.json");
    assert!(bundled_ast.contains(r#""colspan": 2"#));
    assert!(bundled_ast.contains(r#""rowspan": 2"#));
    assert!(bundled_ast.contains(r#""continues_rowspan": true"#));
}

#[test]
fn imported_html_table_spans_flow_through_semantic_exports() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Imported Merged Tables\nstatus: approved\napprovedBy: QA\n---\n# Imported Merged Tables\n<table class=\"transform-table\"><thead><tr><th colspan=\"2\">Group</th><th>Owner</th></tr></thead><tbody><tr><td rowspan=\"2\">Discovery</td><td>Scope</td><td>PM</td></tr><tr><td>Detail</td><td>Analyst</td></tr></tbody></table>\n"
            .to_string(),
        file_path: None,
    });

    let table = response
        .document_ast
        .blocks
        .iter()
        .find_map(|block| {
            if let DocumentBlock::Table {
                header_cells,
                row_cells,
                ..
            } = block
            {
                (header_cells
                    .first()
                    .is_some_and(|cell| cell.text == "Group"))
                .then_some((header_cells, row_cells))
            } else {
                None
            }
        })
        .expect("imported html table");
    assert_eq!(table.0[0].colspan, 2);
    assert_eq!(table.1[0][0].rowspan, 2);
    assert!(table.1[1][0].continues_rowspan);

    let options = json!({});
    let docx = render_docx_bytes(&response, &options).expect("docx imported merged table");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains(r#"<w:gridSpan w:val="2"/>"#));
    assert!(docx_document.contains(r#"<w:vMerge w:val="restart"/>"#));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx imported merged table");
    let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
    assert!(pptx_slide.contains(r#"gridSpan="2""#));
    assert!(pptx_slide.contains(r#"rowSpan="2""#));

    let bundle = render_markdown_bundle_bytes(&response, &response.export_manifest)
        .expect("imported merged table bundle");
    let bundled_text = zip_entry_text(&bundle, "document.txt");
    assert!(bundled_text.contains("| Group {colspan=2} |  | Owner |"));
    assert!(bundled_text.contains("| Discovery {rowspan=2} | Scope | PM |"));
    let bundled_ast = zip_entry_text(&bundle, "document-ast.json");
    assert!(bundled_ast.contains(r#""text": "Group""#));
    assert!(bundled_ast.contains(r#""colspan": 2"#));
    assert!(bundled_ast.contains(r#""rowspan": 2"#));
}
