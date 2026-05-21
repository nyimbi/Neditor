use super::*;

#[test]
fn ai_cleanup_normalizes_chat_artifacts() {
    let response = cleanup_ai_paste(AiCleanupRequest {
        text: "ChatGPT said:\n• First\tSecond\nA\tB\nRevenue grew 24%.".to_string(),
        add_provenance: true,
        mark_as_draft: true,
        insert_citation_todos: true,
        preserve_headings: false,
        convert_numbered_lists: true,
        convert_tables: true,
    });

    assert!(response.cleaned_markdown.contains("- First"));
    assert!(response.cleaned_markdown.contains("| A | B |"));
    assert!(response
        .cleaned_markdown
        .contains("Revenue grew 24%. <!-- TODO: citation needed -->"));
    assert!(response.cleaned_markdown.contains("```ai-source"));
    assert!(response
        .cleaned_markdown
        .contains("ai-assisted: status=needs-review"));
    assert!(response
        .cleaned_markdown
        .contains("promptSummary: AI paste cleanup"));
    assert!(response.issues.len() >= 4);
}

#[test]
fn ai_cleanup_respects_preview_options() {
    let response = cleanup_ai_paste(AiCleanupRequest {
        text: "Assistant:\nClean paragraph.\n```text\nRevenue grew 24%.\n```".to_string(),
        add_provenance: false,
        mark_as_draft: false,
        insert_citation_todos: false,
        preserve_headings: false,
        convert_numbered_lists: true,
        convert_tables: true,
    });

    assert!(!response.cleaned_markdown.contains("draft: AI paste"));
    assert!(!response.cleaned_markdown.contains("```ai-source"));
    assert!(!response.cleaned_markdown.contains("TODO: citation needed"));
    assert!(response.provenance_block.is_none());
}

#[test]
fn ai_cleanup_normalizes_chat_list_numbering() {
    let response = cleanup_ai_paste(AiCleanupRequest {
            text: "1) First action\n  ◦ Nested action\n2) Second action\n```text\n1) literal\n◦ literal\n```"
                .to_string(),
            add_provenance: false,
            mark_as_draft: false,
            insert_citation_todos: false,
            preserve_headings: false,
            convert_numbered_lists: true,
            convert_tables: true,
        });

    assert!(response.cleaned_markdown.contains("1. First action"));
    assert!(response.cleaned_markdown.contains("  - Nested action"));
    assert!(response.cleaned_markdown.contains("2. Second action"));
    assert!(response
        .cleaned_markdown
        .contains("```text\n1) literal\n◦ literal\n```"));
}

#[test]
fn ai_cleanup_removes_chat_labels_without_touching_code_fences() {
    let response = cleanup_ai_paste(AiCleanupRequest {
            text: "DeepSeek said:\nAssistant: Revenue grew 24%.\n```text\nAssistant: literal\nChatGPT said: literal\n```\nYou: ignore this prompt"
                .to_string(),
            add_provenance: false,
            mark_as_draft: false,
            insert_citation_todos: false,
            preserve_headings: false,
            convert_numbered_lists: true,
            convert_tables: true,
        });

    assert!(!response.cleaned_markdown.contains("DeepSeek said:"));
    assert!(response.cleaned_markdown.contains("Revenue grew 24%."));
    assert!(response
        .cleaned_markdown
        .contains("```text\nAssistant: literal\nChatGPT said: literal\n```"));
    assert!(response.cleaned_markdown.contains("ignore this prompt"));
    assert!(response
        .issues
        .iter()
        .any(|issue| issue.contains("Removed chat labels")));
}

#[test]
fn ai_cleanup_removes_duplicate_markdown_headings() {
    let response = cleanup_ai_paste(AiCleanupRequest {
            text: "## Market Update\n\n## Market Update\nRevenue grew 24%.\n\n```markdown\n## Market Update\n## Market Update\n```"
                .to_string(),
            add_provenance: false,
            mark_as_draft: false,
            insert_citation_todos: false,
            preserve_headings: false,
            convert_numbered_lists: true,
            convert_tables: true,
        });

    assert_eq!(
        response
            .cleaned_markdown
            .matches("## Market Update")
            .count(),
        3
    );
    assert!(response
        .cleaned_markdown
        .contains("```markdown\n## Market Update\n## Market Update\n```"));
    assert!(response
        .issues
        .iter()
        .any(|issue| issue.contains("duplicated heading")));
}

#[test]
fn ai_cleanup_converts_csv_table_blocks_conservatively() {
    let response = cleanup_ai_paste(AiCleanupRequest {
            text: "Region,Revenue,Growth\nEMEA,1200,24%\nAMER,950,12%\n\nThis sentence, with a comma, should stay prose.\n```csv\nRegion,Revenue\nEMEA,1200\n```"
                .to_string(),
            add_provenance: false,
            mark_as_draft: false,
            insert_citation_todos: false,
            preserve_headings: false,
            convert_numbered_lists: true,
            convert_tables: true,
        });

    assert!(response
        .cleaned_markdown
        .contains("| Region | Revenue | Growth |\n| --- | --- | --- |\n| EMEA | 1200 | 24% |"));
    assert!(response
        .cleaned_markdown
        .contains("This sentence, with a comma, should stay prose."));
    assert!(response
        .cleaned_markdown
        .contains("```csv\nRegion,Revenue\nEMEA,1200\n```"));
    assert!(response
        .issues
        .iter()
        .any(|issue| issue.contains("comma-separated table")));
}

#[test]
fn ai_cleanup_respects_structure_conversion_options() {
    let response = cleanup_ai_paste(AiCleanupRequest {
        text: "## Market Update\n\n## Market Update\n1) Review revenue\nRegion,Revenue\nEMEA,1200"
            .to_string(),
        add_provenance: false,
        mark_as_draft: false,
        insert_citation_todos: false,
        preserve_headings: true,
        convert_numbered_lists: false,
        convert_tables: false,
    });

    assert_eq!(
        response
            .cleaned_markdown
            .matches("## Market Update")
            .count(),
        2
    );
    assert!(response.cleaned_markdown.contains("1) Review revenue"));
    assert!(response.cleaned_markdown.contains("Region,Revenue"));
    assert!(!response.cleaned_markdown.contains("| Region | Revenue |"));
}

#[test]
fn ai_cleanup_normalizes_rich_html_clipboard_content() {
    let response = cleanup_ai_paste(AiCleanupRequest {
            text: "<h2>Board Update</h2><p>Revenue grew 24%. <a href=\"https://example.com/report?x=1&amp;y=2\">Source report</a></p><ul><li>Approve budget</li></ul><table><tr><th>Region</th><th>Revenue</th></tr><tr><td>EMEA</td><td>24</td></tr></table>"
                .to_string(),
            add_provenance: false,
            mark_as_draft: false,
            insert_citation_todos: true,
            preserve_headings: false,
            convert_numbered_lists: true,
            convert_tables: true,
        });

    assert!(response.cleaned_markdown.contains("## Board Update"));
    assert!(response.cleaned_markdown.contains("Revenue grew 24%."));
    assert!(response
        .cleaned_markdown
        .contains("[Source report](https://example.com/report?x=1&y=2)"));
    assert!(response.cleaned_markdown.contains("- Approve budget"));
    assert!(response.cleaned_markdown.contains("| Region | Revenue |"));
    assert!(response.cleaned_markdown.contains("| --- | --- |"));
    assert!(response.cleaned_markdown.contains("| EMEA | 24 |"));
    assert!(response
        .issues
        .iter()
        .any(|issue| issue.contains("Converted rich HTML clipboard")));
}

#[test]
fn ai_cleanup_normalizes_ai_code_fence_variants() {
    let response = cleanup_ai_paste(AiCleanupRequest {
            text: "Copy code\n``` python\nprint(\"24% growth\")\n```\n\n~~~ TypeScript\nconst revenue = 24;\n~~~\n\nRevenue grew 24%.".to_string(),
            add_provenance: false,
            mark_as_draft: false,
            insert_citation_todos: true,
            preserve_headings: false,
            convert_numbered_lists: true,
            convert_tables: true,
        });

    assert!(response
        .cleaned_markdown
        .contains("```python\nprint(\"24% growth\")\n```"));
    assert!(response
        .cleaned_markdown
        .contains("```typescript\nconst revenue = 24;\n```"));
    assert!(!response.cleaned_markdown.contains("Copy code"));
    assert!(!response
        .cleaned_markdown
        .contains("print(\"24% growth\") <!-- TODO: citation needed -->"));
    assert!(!response
        .cleaned_markdown
        .contains("const revenue = 24; <!-- TODO: citation needed -->"));
    assert!(response
        .cleaned_markdown
        .contains("Revenue grew 24%. <!-- TODO: citation needed -->"));
    assert!(response
        .issues
        .iter()
        .any(|issue| issue.contains("AI code fence")));
    assert!(response
        .issues
        .iter()
        .any(|issue| issue.contains("AI code copy label")));
}

#[test]
fn ai_cleanup_converts_rich_html_pre_code_blocks_to_fences() {
    let response = cleanup_ai_paste(AiCleanupRequest {
            text: "<p>Use this:</p><pre><code class=\"language-python\">for item in items:\n    print(&lt;tag&gt;)</code></pre><p>Revenue grew 24%.</p>"
                .to_string(),
            add_provenance: false,
            mark_as_draft: false,
            insert_citation_todos: true,
            preserve_headings: false,
            convert_numbered_lists: true,
            convert_tables: true,
        });

    assert!(response
        .cleaned_markdown
        .contains("```python\nfor item in items:\n    print(<tag>)\n```"));
    assert!(!response
        .cleaned_markdown
        .contains("print(<tag>) <!-- TODO: citation needed -->"));
    assert!(response
        .cleaned_markdown
        .contains("Revenue grew 24%. <!-- TODO: citation needed -->"));
    assert!(response
        .issues
        .iter()
        .any(|issue| issue.contains("rich HTML code block")));
    assert!(response
        .issues
        .iter()
        .any(|issue| issue.contains("rich HTML clipboard")));
}

#[test]
fn ai_cleanup_preserves_code_fence_content() {
    let response = cleanup_ai_paste(AiCleanupRequest {
            text: "Assistant:\n```text\n• literal bullet\nA\tB\nRevenue grew 24%.\n```\n\n• Real bullet\nA\tB\nRevenue grew 24%.".to_string(),
            add_provenance: false,
            mark_as_draft: false,
            insert_citation_todos: true,
            preserve_headings: false,
            convert_numbered_lists: true,
            convert_tables: true,
        });

    assert!(response
        .cleaned_markdown
        .contains("```text\n• literal bullet\nA\tB\nRevenue grew 24%.\n```"));
    assert!(response.cleaned_markdown.contains("- Real bullet"));
    assert!(response.cleaned_markdown.contains("| A | B |"));
    assert!(response
        .cleaned_markdown
        .contains("Revenue grew 24%. <!-- TODO: citation needed -->"));
}
