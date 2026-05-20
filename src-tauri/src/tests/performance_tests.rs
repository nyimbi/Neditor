use super::*;
use std::time::Instant;

#[cfg(target_os = "linux")]
fn current_resident_set_kb() -> Option<u64> {
    let statm = fs::read_to_string("/proc/self/statm").ok()?;
    let resident_pages = statm.split_whitespace().nth(1)?.parse::<u64>().ok()?;
    Some(resident_pages * 4)
}

#[cfg(target_os = "macos")]
fn current_resident_set_kb() -> Option<u64> {
    use std::mem::{size_of, zeroed};
    use std::os::raw::{c_int, c_void};

    #[repr(C)]
    struct ProcTaskInfo {
        pti_virtual_size: u64,
        pti_resident_size: u64,
        pti_total_user: u64,
        pti_total_system: u64,
        pti_threads_user: u64,
        pti_threads_system: u64,
        pti_policy: i32,
        pti_faults: i32,
        pti_pageins: i32,
        pti_cow_faults: i32,
        pti_messages_sent: i32,
        pti_messages_received: i32,
        pti_syscalls_mach: i32,
        pti_syscalls_unix: i32,
        pti_csw: i32,
        pti_threadnum: i32,
        pti_numrunning: i32,
        pti_priority: i32,
    }

    #[link(name = "proc")]
    extern "C" {
        fn proc_pidinfo(
            pid: c_int,
            flavor: c_int,
            arg: u64,
            buffer: *mut c_void,
            buffersize: c_int,
        ) -> c_int;
    }

    const PROC_PIDTASKINFO: c_int = 4;
    let mut info: ProcTaskInfo = unsafe { zeroed() };
    let expected_size = size_of::<ProcTaskInfo>() as c_int;
    let result = unsafe {
        proc_pidinfo(
            std::process::id() as c_int,
            PROC_PIDTASKINFO,
            0,
            &mut info as *mut ProcTaskInfo as *mut c_void,
            expected_size,
        )
    };
    if result != expected_size {
        return None;
    }
    Some(info.pti_resident_size / 1024)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn current_resident_set_kb() -> Option<u64> {
    None
}

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

#[test]
fn repeated_export_loop_keeps_large_artifacts_stable() {
    let mut text = String::from(
        "---\ntitle: Loop Export Stress\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-20\nclient: Example Holdings\ntoc: true\n---\n# Loop Export Stress\n\nPrepared for {{client}}.\n\n[TOC]\n\n",
    );
    text.push_str("```calc\n");
    for index in 0..90 {
        text.push_str(&format!("loop_metric_{index} = {index} + 10\n"));
    }
    text.push_str("loop_total = SUM(loop_metric_1, loop_metric_2, loop_metric_3)\n```\n\n");

    for index in 0..50 {
        text.push_str(&format!(
            "## Export Section {index}\n\nParagraph {index} with enough body text to exercise layout and repeated export rendering across targets.\n\nTable: Export metrics {index} {{#tbl:export-{index}}}\n| Metric | Value |\n| --- | ---: |\n| Revenue | {} |\n| Cost | {} |\n| Margin | =SUM(B1:B2) |\n\n```csv caption=\"Loop data {index}\" audited\nRegion,Revenue\nNorth,{}\nSouth,{}\n```\n\n",
            500 + index,
            200 + index,
            75 + index,
            25 + index
        ));
    }

    let response = compile_with_options(
        CompileRequest {
            text,
            file_path: None,
        },
        &json!({ "includeSyntaxHighlighting": true }),
    );
    let options = json!({
        "includeSyntaxHighlighting": true,
        "includeManifest": true,
        "includeCoverPage": true,
        "includePageNumbers": true,
        "includeToc": true,
        "includeCommentsAppendix": true,
        "includeAiProvenanceAppendix": true,
        "includeGlossaryAppendix": true,
        "includeAgenda": true,
        "watermark": "CONFIDENTIAL"
    });

    assert!(response.semantic.headings.len() >= 50);
    assert!(response.semantic.tables >= 50);
    assert!(response.formula_graph.len() >= 90);
    assert!(response.transform_artifacts.len() >= 50);

    let mut previous_lengths: Option<[usize; 5]> = None;
    let started_at = Instant::now();
    for iteration in 0..3 {
        let html = render_full_html(&response, &options);
        let pdf = render_pdf_bytes(&response, &options);
        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
        let bundle = render_markdown_bundle_bytes(&response, &response.export_manifest)
            .expect("markdown bundle bytes");

        assert!(html.contains("Loop Export Stress"));
        assert!(html.contains("Export Section 49"));
        assert!(pdf.starts_with(b"%PDF-1.4"));
        let pdf_text = String::from_utf8_lossy(&pdf).into_owned();
        assert!(pdf_text.contains("Loop Export Stress"));
        assert!(zip_has_entry(&docx, "word/document.xml"));
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        assert!(docx_document.contains("Loop Export Stress"));
        assert!(zip_has_entry(&pptx, "ppt/presentation.xml"));
        let pptx_slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/").join("\n");
        assert!(pptx_slides.contains("Export Section 49"));
        assert!(zip_has_entry(&bundle, "document.md"));
        assert!(zip_has_entry(&bundle, "manifest.json"));
        assert!(zip_has_entry(&bundle, "transform-artifacts.json"));
        let bundled_document = zip_entry_text(&bundle, "document.md");
        assert!(bundled_document.contains("Loop Export Stress"));
        assert!(zip_entry_text(&bundle, "transform-artifacts.json").contains("Loop data 49"));

        let lengths = [html.len(), pdf.len(), docx.len(), pptx.len(), bundle.len()];
        for (target, length) in ["html", "pdf", "docx", "pptx", "bundle"]
            .iter()
            .zip(lengths)
        {
            assert!(
                length > 1024,
                "{target} artifact was unexpectedly small on iteration {iteration}: {length}"
            );
        }
        if let Some(previous) = previous_lengths {
            for ((target, length), previous_length) in ["html", "pdf", "docx", "pptx", "bundle"]
                .iter()
                .zip(lengths)
                .zip(previous)
            {
                let delta = length.abs_diff(previous_length);
                assert!(
                    delta <= 512,
                    "{target} artifact size changed by {delta} bytes on iteration {iteration}"
                );
            }
        }
        previous_lengths = Some(lengths);
    }
    let elapsed = started_at.elapsed();

    assert!(
        elapsed.as_secs() < 20,
        "repeated export loop took {elapsed:?}"
    );
}

#[test]
fn repeated_compile_export_cycles_keep_memory_growth_bounded() {
    let options = json!({
        "includeSyntaxHighlighting": true,
        "includeManifest": true,
        "includeCoverPage": true,
        "includePageNumbers": true,
        "includeToc": true,
        "watermark": "INTERNAL"
    });
    let mut retained_summaries = Vec::new();
    let baseline_rss_kb = current_resident_set_kb();
    let mut peak_rss_growth_kb = 0;
    let mut rss_samples = 0;
    let started_at = Instant::now();

    for iteration in 0..10 {
        let mut text = format!(
            "---\ntitle: Memory Growth Stress {iteration}\nversion: 1.0.{iteration}\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-20\ntoc: true\n---\n# Memory Growth Stress {iteration}\n\n[TOC]\n\n"
        );
        text.push_str("```calc\n");
        for metric in 0..60 {
            text.push_str(&format!(
                "memory_metric_{metric}_{iteration} = {} + {iteration}\n",
                metric + 10
            ));
        }
        text.push_str(&format!(
            "memory_total = SUM(memory_metric_1_{iteration}, memory_metric_2_{iteration}, memory_metric_3_{iteration})\n```\n\n"
        ));
        for section in 0..36 {
            text.push_str(&format!(
                "## Memory Section {section}\n\nParagraph {section} iteration {iteration} keeps enough text in circulation to exercise repeated compile/export allocation paths without retaining full artifacts between cycles.\n\nTable: Memory metrics {section} {{#tbl:memory-{iteration}-{section}}}\n| Metric | Value |\n| --- | ---: |\n| Revenue | {} |\n| Cost | {} |\n| Margin | =SUM(B1:B2) |\n\n```csv caption=\"Memory data {iteration}-{section}\"\nRegion,Revenue\nEast,{}\nWest,{}\n```\n\n",
                800 + section + iteration,
                300 + section,
                175 + iteration,
                125 + section
            ));
        }

        let response = compile_with_options(
            CompileRequest {
                text,
                file_path: None,
            },
            &options,
        );
        let html = render_full_html(&response, &options);
        let pdf = render_pdf_bytes(&response, &options);
        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
        let bundle = render_markdown_bundle_bytes(&response, &response.export_manifest)
            .expect("markdown bundle bytes");
        let lengths = [html.len(), pdf.len(), docx.len(), pptx.len(), bundle.len()];

        assert!(response.semantic.headings.len() >= 36);
        assert!(response.semantic.tables >= 36);
        assert!(response.transform_artifacts.len() >= 36);
        for (target, length) in ["html", "pdf", "docx", "pptx", "bundle"]
            .iter()
            .zip(lengths)
        {
            assert!(
                length > 1024,
                "{target} artifact was unexpectedly small on iteration {iteration}: {length}"
            );
        }

        retained_summaries.push((
            response.semantic.headings.len(),
            response.semantic.tables,
            response.transform_artifacts.len(),
            lengths,
        ));

        if let (Some(baseline), Some(current)) = (baseline_rss_kb, current_resident_set_kb()) {
            rss_samples += 1;
            peak_rss_growth_kb = peak_rss_growth_kb.max(current.saturating_sub(baseline));
        }
    }

    assert_eq!(retained_summaries.len(), 10);
    assert!(retained_summaries.iter().all(
        |(_, table_count, transform_count, lengths)| *table_count >= 36
            && *transform_count >= 36
            && lengths.iter().all(|length| *length > 1024)
    ));
    assert!(
        peak_rss_growth_kb < 196_608,
        "repeated compile/export cycles grew resident memory by {peak_rss_growth_kb} KiB"
    );
    if !cfg!(windows) {
        assert!(rss_samples > 0, "expected at least one process RSS sample");
    }
    let elapsed = started_at.elapsed();
    assert!(
        elapsed.as_secs() < 30,
        "repeated compile/export memory stress took {elapsed:?}"
    );
}

#[cfg(unix)]
#[test]
fn repeated_editing_sessions_reuse_external_transform_cache() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-edit-cache-stress-{unique}"));
    fs::create_dir_all(&root).expect("create edit cache stress dir");
    let counter_path = root.join("graphviz-count.txt");
    let graphviz = write_executable_script(
        "graphviz-cache-stress",
        &format!(
            "#!/bin/sh\ncount_file=\"{}\"\ncount=0\nif [ -f \"$count_file\" ]; then count=$(cat \"$count_file\"); fi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$count_file\"\nprintf '<svg><text>'\ncat\nprintf '</text></svg>'\n",
            path_to_string(&counter_path)
        ),
    );
    let graphviz_path = path_to_string(&graphviz);
    let graph_body = format!("digraph Cache{unique} {{ alpha -> beta; }}");
    let options = json!({
        "transformEnginePaths": { "dot": graphviz_path },
        "trustedTransformEngines": { "dot": true },
        "transformInputModes": { "dot": "stdin" },
        "transformTimeoutMs": 1000
    });

    let mut previous_cache_key: Option<String> = None;
    let started_at = Instant::now();
    for iteration in 0..8 {
        let text = format!(
            "---\ntitle: Editing Cache Stress\nversion: 1.0.{iteration}\nstatus: draft\n---\n# Editing Cache Stress\n\nRevision {iteration} keeps the diagram stable while the document changes.\n\n```dot\n{graph_body}\n```\n\n```csv caption=\"Edit data {iteration}\"\nRegion,Revenue\nNorth,{}\nSouth,{}\n```\n",
            100 + iteration,
            80 + iteration
        );
        let response = compile_with_options(
            CompileRequest {
                text,
                file_path: Some(path_to_string(&root.join("editing-cache.md"))),
            },
            &options,
        );
        let dot_artifact = response
            .transform_artifacts
            .iter()
            .find(|artifact| artifact.name == "dot")
            .expect("dot artifact");

        assert!(response.html.contains(&format!("Revision {iteration}")));
        assert!(dot_artifact.html.contains(&graph_body));
        assert_eq!(dot_artifact.execution_kind, "external");
        assert_eq!(dot_artifact.input_mode, "stdin");
        if let Some(previous) = &previous_cache_key {
            assert_eq!(&dot_artifact.cache_key, previous);
            assert_eq!(dot_artifact.duration_ms, Some(0));
            assert!(response
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("served from cache")));
        } else {
            previous_cache_key = Some(dot_artifact.cache_key.clone());
            transforms::external::clear_external_transform_memory_cache_for_tests();
        }
        assert_eq!(
            fs::read_to_string(&counter_path).expect("counter text"),
            "1",
            "external transform should execute only once despite repeated edits"
        );
    }
    let elapsed = started_at.elapsed();

    assert!(
        elapsed.as_secs() < 20,
        "repeated editing cache stress took {elapsed:?}"
    );
    fs::remove_dir_all(root).expect("clean edit cache stress dir");
    let _ = fs::remove_file(graphviz);
}
