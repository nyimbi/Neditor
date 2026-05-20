use super::*;
use std::time::Instant;

#[test]
fn compiler_stress_handles_large_documents_with_many_artifacts() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-large-doc-stress-{unique}"));
    fs::create_dir_all(root.join("chapters")).expect("create stress dirs");

    let include_depth = 12;
    for index in 0..include_depth {
        let next = if index + 1 < include_depth {
            format!("!include chapter-{:02}.md\n", index + 1)
        } else {
            String::new()
        };
        fs::write(
            root.join("chapters").join(format!("chapter-{index:02}.md")),
            format!("## Included Chapter {index}\n\nIncluded body {index}.\n\n{next}"),
        )
        .expect("write include chapter");
    }

    let mut text = String::from(
        "---\ntitle: Large Stress Report\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-20\n---\n# Large Stress Report\n\n!include chapters/chapter-00.md\n\n",
    );
    text.push_str("```calc\n");
    for index in 0..120 {
        text.push_str(&format!("metric_{index} = {index} + 1\n"));
    }
    text.push_str("portfolio_total = SUM(metric_1, metric_2, metric_3)\n```\n\n");

    for index in 0..80 {
        text.push_str(&format!(
            "## Section {index}\n\nParagraph {index} with a broken [local link](missing-{index}.md) and missing media ![Figure {index}](media/missing-{index}.png){{#fig:missing-{index} caption=\"Missing figure {index}\"}}.\n\nTable: Section {index} metrics {{#tbl:section-{index}}}\n| Metric | Value |\n| --- | ---: |\n| Revenue | {} |\n| Cost | {} |\n| Margin | =SUM(B1:B2) |\n\n```csv caption=\"Regional data {index}\" audited\nRegion,Revenue\nEast,{}\nWest,=SUM(B1,{})\n```\n\n",
            index + 100,
            index + 40,
            index + 10,
            index + 20
        ));
    }

    let started_at = Instant::now();
    let response = compile_with_options(
        CompileRequest {
            text,
            file_path: Some(path_to_string(&root.join("root.md"))),
        },
        &json!({ "includeSyntaxHighlighting": true }),
    );
    let elapsed = started_at.elapsed();

    assert!(
        elapsed.as_secs() < 20,
        "large document stress compile took {elapsed:?}"
    );
    assert_eq!(response.include_graph.len(), include_depth);
    assert!(
        response.source_map.len() > 500,
        "expected source map coverage for large document, got {}",
        response.source_map.len()
    );
    assert!(response.semantic.headings.len() >= 90);
    assert!(response.semantic.tables >= 80);
    assert!(response.formula_graph.len() >= 120);
    assert!(response.transform_artifacts.len() >= 80);
    assert!(
        response.diagnostics.len() >= 150,
        "expected many broken link/media diagnostics, got {}",
        response.diagnostics.len()
    );
    assert!(response
        .export_manifest
        .included_files
        .iter()
        .any(|file| file.path.ends_with("chapter-00.md")));
    assert!(response.compiled_markdown.contains("Included Chapter 11"));

    fs::remove_dir_all(root).expect("clean stress test dir");
}
