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
    })
    .expect_err("empty speech request should fail");
    assert!(empty.contains("No text"));
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
    })
    .expect("macOS say command");

    assert_eq!(command.program, "say");
    assert_eq!(command.args, vec!["-v", "Samantha", "-r", "180"]);
    assert_eq!(command.stdin_text, Some("-not-an-option\nand safe".to_string()));
}
