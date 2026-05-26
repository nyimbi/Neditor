#[test]
fn tts_command_builders_use_argument_safe_native_engines() {
    let supertonic = crate::tts::native_tts_command_for_request(&crate::tts::NativeTtsRequest {
        engine: "supertonic-cli".to_string(),
        text: "Read this selected paragraph.".to_string(),
        voice: Some("F1".to_string()),
        language: Some("en".to_string()),
        rate: None,
        command_path: Some("supertonic".to_string()),
        speed: Some(1.2),
        model_download_acknowledged: Some(true),
        model_storage_path: Some("~/.cache/supertonic/models".to_string()),
    })
    .expect("supertonic command");

    assert_eq!(supertonic.program, "supertonic");
    assert_eq!(
        supertonic.args,
        vec![
            "say",
            "Read this selected paragraph.",
            "--model",
            "supertonic-3",
            "--voice",
            "F1",
            "--lang",
            "en",
            "--speed",
            "1.20",
        ]
    );
    assert_eq!(supertonic.stdin_text, None);

    let empty = crate::tts::native_tts_command_for_request(&crate::tts::NativeTtsRequest {
        engine: "supertonic-cli".to_string(),
        text: "   ".to_string(),
        voice: None,
        language: None,
        rate: None,
        command_path: Some("supertonic".to_string()),
        speed: None,
        model_download_acknowledged: Some(true),
        model_storage_path: None,
    })
    .expect_err("empty speech request should fail");
    assert!(empty.contains("No text"));

    let unacknowledged =
        crate::tts::native_tts_command_for_request(&crate::tts::NativeTtsRequest {
            engine: "supertonic-cli".to_string(),
            text: "Read after consent.".to_string(),
            voice: None,
            language: None,
            rate: None,
            command_path: Some("supertonic".to_string()),
            speed: None,
            model_download_acknowledged: Some(false),
            model_storage_path: Some("~/.cache/supertonic/models".to_string()),
        })
        .expect_err("supertonic should require model download acknowledgement");
    assert!(unacknowledged.contains("~305 MB model download"));
    assert!(unacknowledged.contains("~/.cache/supertonic/models"));
}

#[test]
fn tts_model_download_command_requires_explicit_acknowledgement() {
    let blocked = crate::tts::native_tts_model_download_command_for_request(
        &crate::tts::NativeTtsModelDownloadRequest {
            engine: "supertonic-cli".to_string(),
            command_path: Some("supertonic".to_string()),
            model: Some("supertonic-3".to_string()),
            approximate_size: Some("~305 MB".to_string()),
            storage_path: Some("~/.cache/supertonic/models".to_string()),
            acknowledged: Some(false),
        },
    )
    .expect_err("download should require consent");
    assert!(blocked.contains("model name, download size, and storage location"));

    let command = crate::tts::native_tts_model_download_command_for_request(
        &crate::tts::NativeTtsModelDownloadRequest {
            engine: "supertonic-cli".to_string(),
            command_path: Some("supertonic".to_string()),
            model: Some("supertonic-3".to_string()),
            approximate_size: Some("~305 MB".to_string()),
            storage_path: Some("~/.cache/supertonic/models".to_string()),
            acknowledged: Some(true),
        },
    )
    .expect("download command");
    assert_eq!(command.program, "supertonic");
    assert_eq!(command.args, vec!["download"]);
    assert_eq!(command.stdin_text, None);
}

#[test]
fn tts_inspection_reports_browser_and_configured_native_engines_without_launching() {
    let report = crate::tts::inspect_native_tts(crate::tts::NativeTtsInspectionRequest {
        supertonic_command: Some("supertonic-command-that-should-not-exist".to_string()),
    })
    .expect("tts inspection");

    assert!(report
        .engines
        .iter()
        .any(|engine| engine.id == "browser-speech" && engine.available));
    assert!(report.engines.iter().any(|engine| engine.id == "macos-say"));
    let supertonic = report
        .engines
        .iter()
        .find(|engine| engine.id == "supertonic-cli")
        .expect("supertonic status");
    assert!(!supertonic.available);
    assert!(supertonic.detail.contains("not found"));
}

#[cfg(target_os = "macos")]
#[test]
fn macos_say_reads_text_via_stdin_instead_of_shell_interpolation() {
    let command = crate::tts::native_tts_command_for_request(&crate::tts::NativeTtsRequest {
        engine: "macos-say".to_string(),
        text: "-not-an-option\nand safe".to_string(),
        voice: Some("Samantha".to_string()),
        language: None,
        rate: Some(180),
        command_path: None,
        speed: None,
        model_download_acknowledged: None,
        model_storage_path: None,
    })
    .expect("macOS say command");

    assert_eq!(command.program, "say");
    assert_eq!(command.args, vec!["-v", "Samantha", "-r", "180"]);
    assert_eq!(
        command.stdin_text,
        Some("-not-an-option\nand safe".to_string())
    );
}
