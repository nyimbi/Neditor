use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

// ── Filename generation ───────────────────────────────────────────────────────

#[test]
fn filename_take_photo_has_jpg_extension() {
    use chrono::Utc;
    use continuity_camera::make_asset_filename;
    use continuity_camera::ContinuityKind;
    let ts = Utc::now();
    let name = make_asset_filename(ContinuityKind::TakePhoto, ts, 0xdeadbeef);
    assert!(
        name.ends_with(".jpg"),
        "take-photo filename must end with .jpg, got: {name}"
    );
    assert!(
        name.starts_with("photo-"),
        "filename must start with photo-, got: {name}"
    );
}

#[test]
fn filename_scan_documents_has_jpg_extension() {
    use chrono::Utc;
    use continuity_camera::make_asset_filename;
    use continuity_camera::ContinuityKind;
    let ts = Utc::now();
    let name = make_asset_filename(ContinuityKind::ScanDocuments, ts, 0xabcd1234);
    assert!(
        name.ends_with(".jpg"),
        "scan filename must end with .jpg, got: {name}"
    );
}

#[test]
fn filename_add_sketch_has_png_extension() {
    use chrono::Utc;
    use continuity_camera::make_asset_filename;
    use continuity_camera::ContinuityKind;
    let ts = Utc::now();
    let name = make_asset_filename(ContinuityKind::AddSketch, ts, 0x00000001);
    assert!(
        name.ends_with(".png"),
        "sketch filename must end with .png, got: {name}"
    );
}

#[test]
fn filenames_are_sortable_by_timestamp() {
    use chrono::{Duration, Utc};
    use continuity_camera::make_asset_filename;
    use continuity_camera::ContinuityKind;
    let t1 = Utc::now();
    let t2 = t1 + Duration::seconds(1);
    let earlier = make_asset_filename(ContinuityKind::TakePhoto, t1, 0);
    let later = make_asset_filename(ContinuityKind::TakePhoto, t2, 0);
    assert!(
        earlier < later,
        "filenames must be lexicographically sortable by time"
    );
}

#[test]
fn filenames_with_same_timestamp_differ_by_uid() {
    use chrono::Utc;
    use continuity_camera::make_asset_filename;
    use continuity_camera::ContinuityKind;
    let ts = Utc::now();
    let a = make_asset_filename(ContinuityKind::TakePhoto, ts, 1);
    let b = make_asset_filename(ContinuityKind::TakePhoto, ts, 2);
    assert_ne!(a, b, "different uids must produce different filenames");
}

#[test]
fn filename_contains_date_segment() {
    use chrono::{TimeZone, Utc};
    use continuity_camera::make_asset_filename;
    use continuity_camera::ContinuityKind;
    // Fixed timestamp: 2025-03-15 12:34:56 UTC
    let ts = Utc.with_ymd_and_hms(2025, 3, 15, 12, 34, 56).unwrap();
    let name = make_asset_filename(ContinuityKind::TakePhoto, ts, 0);
    assert!(
        name.contains("20250315-123456"),
        "filename must embed YYYYMMDD-HHMMSS, got: {name}"
    );
}

// ── ContinuityKind serde round-trip ──────────────────────────────────────────

#[test]
fn continuity_kind_serde_round_trip_take_photo() {
    use continuity_camera::ContinuityKind;
    let original = ContinuityKind::TakePhoto;
    let json = serde_json::to_string(&original).expect("serialize TakePhoto");
    assert_eq!(
        json, r#""TakePhoto""#,
        "serde must use PascalCase variant name"
    );
    let deserialized: ContinuityKind = serde_json::from_str(&json).expect("deserialize TakePhoto");
    assert_eq!(deserialized, original);
}

#[test]
fn continuity_kind_serde_round_trip_scan_documents() {
    use continuity_camera::ContinuityKind;
    let original = ContinuityKind::ScanDocuments;
    let json = serde_json::to_string(&original).expect("serialize ScanDocuments");
    assert_eq!(json, r#""ScanDocuments""#);
    let deserialized: ContinuityKind =
        serde_json::from_str(&json).expect("deserialize ScanDocuments");
    assert_eq!(deserialized, original);
}

#[test]
fn continuity_kind_serde_round_trip_add_sketch() {
    use continuity_camera::ContinuityKind;
    let original = ContinuityKind::AddSketch;
    let json = serde_json::to_string(&original).expect("serialize AddSketch");
    assert_eq!(json, r#""AddSketch""#);
    let deserialized: ContinuityKind = serde_json::from_str(&json).expect("deserialize AddSketch");
    assert_eq!(deserialized, original);
}

// ── Workspace scoping ─────────────────────────────────────────────────────────

#[test]
fn write_asset_refuses_path_outside_workspace() {
    use continuity_camera::ContinuityKind;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time ok")
        .as_nanos();

    let workspace = std::env::temp_dir().join(format!("neditor-cc-scope-test-{unique}"));
    let doc = workspace.join("draft.md");
    fs::create_dir_all(&workspace).expect("create workspace");
    fs::write(&doc, "# Draft").expect("write document");

    // A symlink that points outside the workspace.
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let outside = std::env::temp_dir().join(format!("neditor-cc-outside-{unique}.jpg"));
        fs::write(&outside, b"not in workspace").expect("write outside file");
        let link = workspace.join("assets").join("escape.jpg");
        fs::create_dir_all(workspace.join("assets")).expect("create assets dir");
        symlink(&outside, &link).expect("create symlink");

        // resolve_within_workspace must refuse the symlink.
        let link_str = link.to_string_lossy().into_owned();
        let workspace_str = workspace.to_string_lossy().into_owned();
        let result = filesystem::resolve_within_workspace(&link_str, Some(&workspace_str), false);
        assert!(
            result.is_err(),
            "must refuse symlink inside workspace: {:?}",
            result
        );
        let msg = result.unwrap_err();
        assert!(
            msg.contains("symlink"),
            "error must mention symlink, got: {msg}"
        );

        fs::remove_file(&outside).ok();
    }

    fs::remove_dir_all(&workspace).ok();
}

#[test]
fn write_asset_confines_output_to_workspace_assets_dir() {
    use continuity_camera::ContinuityKind;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time ok")
        .as_nanos();

    let workspace = std::env::temp_dir().join(format!("neditor-cc-write-test-{unique}"));
    let doc = workspace.join("report.md");
    fs::create_dir_all(&workspace).expect("create workspace");
    fs::write(&doc, "# Report").expect("write document");

    let data = b"JFIF\xff\xd8\xff\xe0fake-jpeg-bytes".to_vec();
    let result = continuity_camera::write_asset(
        data,
        Some(&doc.to_string_lossy()),
        ContinuityKind::TakePhoto,
    )
    .expect("write_asset should succeed");

    assert!(
        result.relative_path.starts_with("assets/"),
        "relative_path must be under assets/, got: {}",
        result.relative_path
    );
    // Canonicalize workspace so /tmp → /private/tmp symlink (macOS) doesn't
    // cause a false mismatch; write_asset returns the canonicalized path.
    let canonical_workspace = workspace
        .canonicalize()
        .unwrap_or_else(|_| workspace.clone());
    assert!(
        result
            .absolute_path
            .starts_with(&canonical_workspace.to_string_lossy().to_string()),
        "absolute_path must be inside workspace, got: {}",
        result.absolute_path
    );
    assert!(result.bytes > 0, "bytes must be non-zero");

    // File must actually exist.
    assert!(
        std::path::Path::new(&result.absolute_path).exists(),
        "asset file must exist on disk"
    );

    fs::remove_dir_all(&workspace).ok();
}

// ── GUI-dependent (requires paired iPhone or iPad) ────────────────────────────

#[test]
#[ignore = "requires paired iPhone or iPad connected via Handoff/Continuity"]
fn capture_take_photo_returns_jpeg_bytes() {
    // This test must run on a Mac with an iPhone paired via Handoff.
    // Run manually: cargo test --manifest-path src-tauri/Cargo.toml \
    //   continuity_camera_tests::capture_take_photo_returns_jpeg_bytes -- --ignored
    //
    // The test validates:
    // - The command finds and triggers the TakePhoto service
    // - The resulting bytes are a valid JPEG (starts with 0xFF 0xD8)
    // - The file is written to the expected assets/ directory
    todo!("requires hardware")
}

#[test]
#[ignore = "requires paired iPhone or iPad connected via Handoff/Continuity"]
fn capture_scan_documents_returns_jpeg_bytes() {
    todo!("requires hardware")
}

#[test]
#[ignore = "requires paired iPhone or iPad connected via Handoff/Continuity"]
fn capture_add_sketch_returns_png_bytes() {
    todo!("requires hardware")
}
