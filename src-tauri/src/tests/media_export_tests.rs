use super::*;

#[test]
fn markdown_bundle_keeps_duplicate_include_basenames_distinct() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-bundle-includes-{unique}"));
    let north = root.join("north");
    let south = root.join("south");
    fs::create_dir_all(&north).expect("create north include dir");
    fs::create_dir_all(&south).expect("create south include dir");
    let north_section = north.join("section.md");
    let south_section = south.join("section.md");
    fs::write(&north_section, "North section").expect("write north include");
    fs::write(&south_section, "South section").expect("write south include");
    let root_doc = root.join("root.md");
    let response = compile(CompileRequest {
            text: "---\ntitle: Bundle Includes\nstatus: approved\napprovedBy: QA\n---\n# Root\n!include north/section.md\n!include south/section.md\n"
                .to_string(),
            file_path: Some(path_to_string(&root_doc)),
        });

    let bundle =
        render_markdown_bundle_bytes(&response, &response.export_manifest).expect("bundle");
    let north_bundle_path = format!(
        "includes/{}-section.md",
        &sha256_hex(path_to_string(&north_section).as_bytes())[..12]
    );
    let south_bundle_path = format!(
        "includes/{}-section.md",
        &sha256_hex(path_to_string(&south_section).as_bytes())[..12]
    );
    assert_ne!(north_bundle_path, south_bundle_path);
    assert_eq!(zip_entry_text(&bundle, &north_bundle_path), "North section");
    assert_eq!(zip_entry_text(&bundle, &south_bundle_path), "South section");
    let manifest = zip_entry_text(&bundle, "manifest.json");
    assert!(manifest.contains("\"include_graph\""));
    assert!(manifest.contains(&path_to_string(&north_section)));
    assert!(manifest.contains(&path_to_string(&south_section)));
    let include_graph = zip_entry_text(&bundle, "include-graph.json");
    assert!(include_graph.contains(&path_to_string(&root_doc)));
    assert!(include_graph.contains(&path_to_string(&north_section)));
    assert!(include_graph.contains(&path_to_string(&south_section)));
    let include_map = zip_entry_text(&bundle, "include-map.json");
    assert!(include_map.contains(&format!("\"bundle_path\": \"{north_bundle_path}\"")));
    assert!(include_map.contains(&format!("\"bundle_path\": \"{south_bundle_path}\"")));
    assert!(include_map.contains(&path_to_string(&north_section)));
    assert!(include_map.contains(&path_to_string(&south_section)));

    fs::remove_dir_all(root).expect("clean bundle include fixture");
}

#[test]
fn export_packages_local_figure_media_relative_to_source_file() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-local-media-export-{unique}"));
    let assets = root.join("assets");
    fs::create_dir_all(&assets).expect("create media fixture dir");
    let image = assets.join("diagram.svg");
    fs::write(
            &image,
            "<svg width=\"320\" height=\"180\" viewBox=\"0 0 320 180\"><rect width=\"320\" height=\"180\"/></svg>",
        )
        .expect("write svg");
    let doc = root.join("report.md");
    fs::write(
            &doc,
            "---\ntitle: Local Media\nstatus: approved\napprovedBy: QA\n---\n# Local Media\n![Diagram](assets/diagram.svg){#fig:local caption=\"Local diagram\"}\n",
        )
        .expect("write document");

    let response = compile(CompileRequest {
        text: fs::read_to_string(&doc).expect("read document"),
        file_path: Some(path_to_string(&doc)),
    });
    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error"));
    assert!(response
        .export_manifest
        .media_files
        .iter()
        .any(|file| { file.path == path_to_string(&image) && file.hash.starts_with("sha256:") }));

    let options = json!({});
    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    let docx_relationships = zip_entry_text(&docx, "word/_rels/document.xml.rels");
    let docx_svg = zip_entry_text(&docx, "word/media/image1.svg");
    assert!(docx_document.contains(r#"r:embed="rIdImage1""#));
    assert!(docx_document.contains(r#"<wp:extent cx="3048000" cy="1714500""#));
    assert!(docx_document.contains(r#"<a:ext cx="3048000" cy="1714500""#));
    assert!(docx_relationships.contains(r#"Target="media/image1.svg""#));
    assert!(docx_svg.contains("<rect"));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
    let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
    let pptx_relationships = zip_entry_text(&pptx, "ppt/slides/_rels/slide2.xml.rels");
    let pptx_svg = zip_entry_text(&pptx, "ppt/media/image1.svg");
    assert!(pptx_slide.contains(r#"r:embed="rIdImage1""#));
    assert!(pptx_slide.contains(r#"<a:ext cx="3048000" cy="1714500""#));
    assert!(pptx_relationships.contains(r#"Target="../media/image1.svg""#));
    assert!(pptx_svg.contains("<rect"));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains(" 240 135 re S"));
    assert!(pdf_text.contains("Local diagram"));

    let mut bundle_manifest = response.export_manifest.clone();
    bundle_manifest.export_options = options;
    let bundle = render_markdown_bundle_bytes(&response, &bundle_manifest).expect("bundle");
    let media_map = zip_entry_text(&bundle, "media-map.json");
    assert!(media_map.contains(r#""width_px": 320.0"#));
    assert!(media_map.contains(r#""height_px": 180.0"#));

    fs::remove_dir_all(root).expect("clean media export fixture");
}

#[test]
fn export_packages_preserve_figure_cover_fit_crop() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-cover-fit-export-{unique}"));
    let assets = root.join("assets");
    fs::create_dir_all(&assets).expect("create cover fit fixture dir");
    let image = assets.join("square.svg");
    fs::write(
            &image,
            "<svg width=\"320\" height=\"320\" viewBox=\"0 0 320 320\"><rect width=\"320\" height=\"320\"/></svg>",
        )
        .expect("write square svg");
    let doc = root.join("report.md");
    fs::write(
            &doc,
            "---\ntitle: Cover Fit\nstatus: approved\napprovedBy: QA\n---\n# Cover Fit\n![Square](assets/square.svg){#fig:square caption=\"Square crop\" fit=\"cover\"}\n",
        )
        .expect("write document");

    let response = compile(CompileRequest {
        text: fs::read_to_string(&doc).expect("read document"),
        file_path: Some(path_to_string(&doc)),
    });
    assert!(response.html.contains("figure-fit-cover"));
    assert!(response.html.contains("data-fit=\"cover\""));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Figure { id, fit, .. }
                if id.as_deref() == Some("fig:square")
                    && fit.as_deref() == Some("cover")
        )
    }));
    assert!(export::export_text(&response, &json!({})).contains("fit=cover"));

    let options = json!({});
    let full_html = render_full_html(&response, &options);
    assert!(full_html.contains("figure[data-fit='cover'] img"));

    let docx = render_docx_bytes(&response, &options).expect("docx cover fit");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains(r#"<wp:extent cx="4320000" cy="3240000""#));
    assert!(docx_document.contains(r#"<a:srcRect t="12500" b="12500"/>"#));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx cover fit");
    let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
    assert!(pptx_slide.contains(r#"<a:ext cx="3657600" cy="2057400""#));
    assert!(pptx_slide.contains(r#"<a:srcRect t="21875" b="21875"/>"#));

    let bundle = render_markdown_bundle_bytes(&response, &response.export_manifest)
        .expect("cover fit bundle");
    let media_map = zip_entry_text(&bundle, "media-map.json");
    assert!(media_map.contains(r#""fit": "cover""#));

    fs::remove_dir_all(root).expect("clean cover fit fixture");
}

#[test]
fn export_packages_preserve_figure_cover_crop_position() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-cover-position-export-{unique}"));
    let assets = root.join("assets");
    fs::create_dir_all(&assets).expect("create cover position fixture dir");
    let image = assets.join("square.svg");
    fs::write(
            &image,
            "<svg width=\"320\" height=\"320\" viewBox=\"0 0 320 320\"><rect width=\"320\" height=\"320\"/></svg>",
        )
        .expect("write square svg");
    let doc = root.join("report.md");
    fs::write(
            &doc,
            "---\ntitle: Cover Position\nstatus: approved\napprovedBy: QA\n---\n# Cover Position\n![Square](assets/square.svg){#fig:square-top caption=\"Top crop\" fit=\"cover\" position=\"top\"}\n",
        )
        .expect("write document");

    let response = compile(CompileRequest {
        text: fs::read_to_string(&doc).expect("read document"),
        file_path: Some(path_to_string(&doc)),
    });
    assert!(response.html.contains("figure-position-top"));
    assert!(response.html.contains("data-position=\"top\""));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Figure { id, fit, position, .. }
                if id.as_deref() == Some("fig:square-top")
                    && fit.as_deref() == Some("cover")
                    && position.as_deref() == Some("top")
        )
    }));
    let export_text = export::export_text(&response, &json!({}));
    assert!(export_text.contains("fit=cover"));
    assert!(export_text.contains("position=top"));

    let options = json!({});
    let full_html = render_full_html(&response, &options);
    assert!(full_html.contains("figure[data-position='top'] img"));

    let docx = render_docx_bytes(&response, &options).expect("docx cover position");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains(r#"<a:srcRect t="0" b="25000"/>"#));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx cover position");
    let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
    assert!(pptx_slide.contains(r#"<a:srcRect t="0" b="43750"/>"#));

    let bundle = render_markdown_bundle_bytes(&response, &response.export_manifest)
        .expect("cover position bundle");
    let media_map = zip_entry_text(&bundle, "media-map.json");
    assert!(media_map.contains(r#""position": "top""#));

    fs::remove_dir_all(root).expect("clean cover position fixture");
}

#[test]
fn pptx_repeated_media_keeps_per_figure_crop_settings() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-pptx-repeated-media-{unique}"));
    let assets = root.join("assets");
    fs::create_dir_all(&assets).expect("create repeated media fixture dir");
    let image = assets.join("square.svg");
    fs::write(
            &image,
            "<svg width=\"320\" height=\"320\" viewBox=\"0 0 320 320\"><rect width=\"320\" height=\"320\"/></svg>",
        )
        .expect("write square svg");
    let doc = root.join("report.md");
    fs::write(
            &doc,
            "---\ntitle: Reused Media\nstatus: approved\napprovedBy: QA\n---\n# Reused Media\n![Contain](assets/square.svg){#fig:contain caption=\"Contain\" fit=\"contain\"}\n![Cover](assets/square.svg){#fig:cover caption=\"Cover\" fit=\"cover\" position=\"top\"}\n",
        )
        .expect("write document");

    let response = compile(CompileRequest {
        text: fs::read_to_string(&doc).expect("read document"),
        file_path: Some(path_to_string(&doc)),
    });
    let pptx = render_pptx_bytes(&response, &json!({})).expect("pptx repeated media");
    let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
    assert_eq!(pptx_slide.matches("<p:pic>").count(), 2);
    assert!(pptx_slide.contains(r#"<a:srcRect t="0" b="43750"/>"#));

    let pptx_relationships = zip_entry_text(&pptx, "ppt/slides/_rels/slide2.xml.rels");
    assert_eq!(
        pptx_relationships
            .matches(r#"Target="../media/image1.svg""#)
            .count(),
        1
    );

    fs::remove_dir_all(root).expect("clean repeated media fixture");
}

#[test]
fn export_packages_raster_media_intrinsic_dimensions() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-raster-media-export-{unique}"));
    let assets = root.join("assets");
    fs::create_dir_all(&assets).expect("create media fixture dir");
    let png = assets.join("chart.png");
    fs::write(
        &png,
        [
            0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, b'I', b'H',
            b'D', b'R', 0x00, 0x00, 0x00, 0xc8, 0x00, 0x00, 0x00, 0x64, 0x08, 0x02, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ],
    )
    .expect("write png");
    let jpg = assets.join("photo.jpg");
    fs::write(
        &jpg,
        [
            0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x01,
            0x00, 0x60, 0x00, 0x60, 0x00, 0x00, 0xff, 0xc0, 0x00, 0x11, 0x08, 0x00, 0x78, 0x00,
            0xf0, 0x03, 0x01, 0x11, 0x00, 0x02, 0x11, 0x00, 0x03, 0x11, 0x00, 0xff, 0xd9,
        ],
    )
    .expect("write jpg");
    let doc = root.join("report.md");
    fs::write(
            &doc,
            "---\ntitle: Raster Media\nstatus: approved\napprovedBy: QA\n---\n# Raster Media\n![Chart](assets/chart.png){#fig:chart caption=\"PNG chart\"}\n\n![Photo](assets/photo.jpg){#fig:photo caption=\"JPEG photo\"}\n",
        )
        .expect("write document");

    let response = compile(CompileRequest {
        text: fs::read_to_string(&doc).expect("read document"),
        file_path: Some(path_to_string(&doc)),
    });
    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error"));

    let options = json!({});
    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    let docx_relationships = zip_entry_text(&docx, "word/_rels/document.xml.rels");
    assert!(docx_relationships.contains(r#"Target="media/image1.png""#));
    assert!(docx_relationships.contains(r#"Target="media/image2.jpg""#));
    assert!(docx_document.contains(r#"<wp:extent cx="1905000" cy="952500""#));
    assert!(docx_document.contains(r#"<wp:extent cx="2286000" cy="1143000""#));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
    let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
    let pptx_relationships = zip_entry_text(&pptx, "ppt/slides/_rels/slide2.xml.rels");
    assert!(pptx_relationships.contains(r#"Target="../media/image1.png""#));
    assert!(pptx_relationships.contains(r#"Target="../media/image2.jpg""#));
    assert!(pptx_slide.contains(r#"<a:ext cx="1905000" cy="952500""#));
    assert!(pptx_slide.contains(r#"<a:ext cx="2286000" cy="1143000""#));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains(" 150 75 re S"));
    assert!(pdf_text.contains(" 180 90 re S"));

    fs::remove_dir_all(root).expect("clean media export fixture");
}

#[test]
fn export_keeps_duplicate_relative_media_from_includes_distinct() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-include-media-export-{unique}"));
    let chapter_a = root.join("a");
    let chapter_b = root.join("b");
    fs::create_dir_all(chapter_a.join("assets")).expect("create chapter a assets");
    fs::create_dir_all(chapter_b.join("assets")).expect("create chapter b assets");
    fs::write(
        chapter_a.join("assets").join("diagram.svg"),
        "<svg><text>A</text></svg>",
    )
    .expect("write a svg");
    fs::write(
        chapter_b.join("assets").join("diagram.svg"),
        "<svg><text>B</text></svg>",
    )
    .expect("write b svg");
    fs::write(
        chapter_a.join("section.md"),
        "## A\n![Diagram](assets/diagram.svg){#fig:a caption=\"A diagram\"}\n",
    )
    .expect("write a section");
    fs::write(
        chapter_b.join("section.md"),
        "## B\n![Diagram](assets/diagram.svg){#fig:b caption=\"B diagram\"}\n",
    )
    .expect("write b section");
    let doc = root.join("root.md");
    fs::write(
            &doc,
            "---\ntitle: Include Media\nstatus: approved\napprovedBy: QA\n---\n# Include Media\n!include a/section.md\n!include b/section.md\n",
        )
        .expect("write root document");

    let response = compile(CompileRequest {
        text: fs::read_to_string(&doc).expect("read root document"),
        file_path: Some(path_to_string(&doc)),
    });
    assert_eq!(response.semantic.figures, 2);
    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error"));

    let options = json!({});
    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    assert!(zip_entry_text(&docx, "word/media/image1.svg").contains("<text>A</text>"));
    assert!(zip_entry_text(&docx, "word/media/image2.svg").contains("<text>B</text>"));
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains(r#"r:embed="rIdImage1""#));
    assert!(docx_document.contains(r#"r:embed="rIdImage2""#));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
    assert!(zip_entry_text(&pptx, "ppt/media/image1.svg").contains("<text>A</text>"));
    assert!(zip_entry_text(&pptx, "ppt/media/image2.svg").contains("<text>B</text>"));
    let slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/");
    assert!(slides
        .iter()
        .any(|slide| slide.contains(r#"r:embed="rIdImage1""#)));
    assert!(slides
        .iter()
        .any(|slide| slide.contains(r#"r:embed="rIdImage2""#)));

    fs::remove_dir_all(root).expect("clean include media export fixture");
}
