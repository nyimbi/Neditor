use super::*;

#[test]
fn compiler_tracks_ai_assisted_section_review_status() {
    let source = "---\ntitle: AI Review\nstatus: approved\napprovedBy: QA\n---\n<!-- ai-assisted: status=needs-review | source=ChatGPT | promptSummary=Drafted risk language -->\n# Risk Review\nBody.\n\n<!-- ai-assisted: status=human-reviewed | reviewedBy=Jane Doe | reviewedAt=2026-05-18 | source=Claude | promptSummary=Edited executive summary -->\n## Executive Summary\nReviewed body.\n";
    let response = compile(CompileRequest {
        text: source.to_string(),
        file_path: None,
    });

    assert_eq!(response.semantic.ai_assisted_sections.len(), 2);
    assert_eq!(
        response.semantic.ai_assisted_sections[0].heading,
        "Risk Review"
    );
    assert_eq!(
        response.semantic.ai_assisted_sections[0].prompt_summary,
        "Drafted risk language"
    );
    assert_eq!(
        response.semantic.ai_assisted_sections[1].reviewed_by,
        "Jane Doe"
    );
    assert_eq!(
        response.semantic.ai_assisted_sections[1].heading,
        "Executive Summary"
    );
    assert!(response.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("AI-assisted sections that are not human-reviewed")));
    let ai_review_diagnostic = response
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic
                .message
                .contains("AI-assisted sections that are not human-reviewed")
        })
        .expect("AI review diagnostic");
    assert_eq!(ai_review_diagnostic.severity, "error");
    assert_eq!(ai_review_diagnostic.line, Some(6));
    assert_eq!(
        ai_review_diagnostic.source_file.as_deref(),
        Some("untitled.md")
    );

    let report = prepare_for_export(PrepareExportRequest {
        text: source.to_string(),
        file_path: None,
        target: "pdf".to_string(),
        options: json!({ "includeProvenance": true }),
    });
    assert!(!report.ready);
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("AI-assisted sections that are not human-reviewed")));
}

#[test]
fn compiler_accepts_ai_assisted_section_metadata_aliases() {
    let response = compile(CompileRequest {
            text: "---\ntitle: AI Section Aliases\nstatus: approved\napprovedBy: QA\n---\n<!-- ai-assisted: status=human-reviewed | reviewed_by=Jane Doe | reviewed_at=2026-05-19 | source=OpenAI | prompt_summary=Alias section prompt -->\n# AI Section Aliases\nReviewed body.\n"
                .to_string(),
            file_path: None,
        });

    let section = response
        .semantic
        .ai_assisted_sections
        .first()
        .expect("ai-assisted section");
    assert_eq!(section.reviewed_by, "Jane Doe");
    assert_eq!(section.reviewed_at, "2026-05-19");
    assert_eq!(section.prompt_summary, "Alias section prompt");
    assert!(!response.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("AI-assisted sections that are not human-reviewed")));
}

#[test]
fn compiler_accepts_ai_source_metadata_aliases() {
    let response = compile(CompileRequest {
            text: "---\ntitle: AI Source Aliases\nstatus: approved\napprovedBy: QA\n---\n# AI Source Aliases\n```ai-source\nprovider: OpenAI\nmodel: ChatGPT\ndate: 2026-05-18\nprompt_summary: Alias prompt\nreviewer: Jane Doe\nreviewed_at: 2026-05-19T09:00:00Z\nstatus: human-reviewed\n```\n"
                .to_string(),
            file_path: None,
        });

    let source = response
        .semantic
        .ai_sources
        .first()
        .expect("ai source metadata");
    assert_eq!(source.prompt_summary, "Alias prompt");
    assert_eq!(source.reviewed_by, "Jane Doe");
    assert_eq!(source.reviewed_at, "2026-05-19T09:00:00Z");
    assert!(!response.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("AI-assisted sections that are not human-reviewed")));
}

#[test]
fn compiler_accepts_ai_provenance_block_name_aliases() {
    let response = compile(CompileRequest {
            text: "---\ntitle: AI Block Aliases\nstatus: approved\napprovedBy: QA\n---\n# AI Block Aliases\n```ai-provenance\naiProvider: OpenAI\nmodelName: gpt-5.4\ngeneratedAt: 2026-05-19T09:00:00Z\npromptSummary: Alias provenance block\nreviewedBy: Jane Doe\nreviewedAt: 2026-05-19T10:00:00Z\nstatus: human-reviewed\n```\n\n```llm-source\ntool: Claude\ndeployment: claude-approved\ncreatedAt: 2026-05-19T09:30:00Z\nprompt: Secondary alias block\nreviewer: Sam Reviewer\nreviewDate: 2026-05-19T10:30:00Z\nstatus: human-reviewed\n```\n"
                .to_string(),
            file_path: None,
        });

    assert_eq!(response.semantic.ai_sources.len(), 2);
    assert_eq!(response.semantic.ai_sources[0].provider, "OpenAI");
    assert_eq!(response.semantic.ai_sources[0].model, "gpt-5.4");
    assert_eq!(response.semantic.ai_sources[0].date, "2026-05-19T09:00:00Z");
    assert_eq!(response.semantic.ai_sources[1].provider, "Claude");
    assert_eq!(response.semantic.ai_sources[1].model, "claude-approved");
    assert_eq!(
        response.semantic.ai_sources[1].prompt_summary,
        "Secondary alias block"
    );
}

#[test]
fn compiler_accepts_ai_assisted_comment_marker_aliases() {
    let response = compile(CompileRequest {
            text: "---\ntitle: AI Comment Aliases\nstatus: approved\napprovedBy: QA\n---\n<!-- ai-generated: status=human-reviewed | reviewer=Jane Doe | reviewedAt=2026-05-19 | provider=OpenAI | prompt=Generated intro -->\n# AI Comment Aliases\nReviewed body.\n\n<!-- llm-assisted: human-reviewed | reviewedBy=Sam Reviewer | reviewed_at=2026-05-20 | source=Claude | prompt_summary=Edited section -->\n## Reviewed Section\nReviewed section.\n"
                .to_string(),
            file_path: None,
        });

    assert_eq!(response.semantic.ai_assisted_sections.len(), 2);
    assert_eq!(response.semantic.ai_assisted_sections[0].source, "OpenAI");
    assert_eq!(
        response.semantic.ai_assisted_sections[0].prompt_summary,
        "Generated intro"
    );
    assert_eq!(
        response.semantic.ai_assisted_sections[1].reviewed_by,
        "Sam Reviewer"
    );
    assert!(!response.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("AI-assisted sections that are not human-reviewed")));
}

#[test]
fn document_ast_accepts_ai_source_metadata_aliases() {
    let response = compile(CompileRequest {
            text: "---\ntitle: AI AST Aliases\nstatus: approved\napprovedBy: QA\n---\n# AI AST Aliases\n```ai-source\nprovider: OpenAI\nmodel: ChatGPT\ndate: 2026-05-18\nprompt: Alias prompt\nreviewer: Jane Doe\nreviewDate: 2026-05-19T09:00:00Z\nstatus: human-reviewed\n```\n"
                .to_string(),
            file_path: None,
        });

    let ast_source = response
        .document_ast
        .blocks
        .iter()
        .find_map(|block| match block {
            DocumentBlock::AiSource { provenance, .. } => Some(provenance),
            _ => None,
        })
        .expect("ai source AST block");
    assert_eq!(ast_source.prompt_summary, "Alias prompt");
    assert_eq!(ast_source.reviewed_by, "Jane Doe");
    assert_eq!(ast_source.reviewed_at, "2026-05-19T09:00:00Z");
}

#[test]
fn document_ast_accepts_ai_provenance_block_name_aliases() {
    let response = compile(CompileRequest {
            text: "---\ntitle: AI AST Block Aliases\nstatus: approved\napprovedBy: QA\n---\n# AI AST Block Aliases\n~~~llm-provenance\ntool: Gemini\nmodelName: approved-gemini\ngeneratedAt: 2026-05-19T09:00:00Z\nprompt: Alias prompt\nreviewer: Jane Doe\nreviewDate: 2026-05-19T10:00:00Z\nstatus: human-reviewed\n~~~\n"
                .to_string(),
            file_path: None,
        });

    let ast_source = response
        .document_ast
        .blocks
        .iter()
        .find_map(|block| match block {
            DocumentBlock::AiSource { provenance, .. } => Some(provenance),
            _ => None,
        })
        .expect("ai source AST alias block");
    assert_eq!(ast_source.provider, "Gemini");
    assert_eq!(ast_source.model, "approved-gemini");
    assert_eq!(ast_source.date, "2026-05-19T09:00:00Z");
    assert_eq!(ast_source.prompt_summary, "Alias prompt");
}
