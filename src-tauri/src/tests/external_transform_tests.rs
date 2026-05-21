use super::*;

#[cfg(unix)]
#[test]
fn external_transforms_are_trust_gated_and_limited() {
    let graphviz = write_executable_script(
            "graphviz-adapter",
            "#!/bin/sh\nprintf '<svg data-args=\"%s\">' \"$*\"\nfor arg in \"$@\"; do if [ -f \"$arg\" ]; then cat \"$arg\"; fi; done\ncat\nprintf '</svg>'\n",
        );
    let graphviz_path = path_to_string(&graphviz);
    let trust_error = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: "digraph {}".to_string(),
        engine_path: Some(graphviz_path.clone()),
        trusted: false,
        input_mode: Some("stdin".to_string()),
        output_format: None,
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(1024),
    })
    .unwrap_err();
    assert!(trust_error.contains("explicit trust"));

    let unique_body = format!(
        "<svg>{}</svg>",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos()
    );
    let limit_error = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: "1234".to_string(),
        engine_path: Some(graphviz_path.clone()),
        trusted: true,
        input_mode: Some("stdin".to_string()),
        output_format: None,
        timeout_ms: Some(1000),
        max_input_bytes: Some(3),
        max_output_bytes: Some(1024),
    })
    .unwrap_err();
    assert!(limit_error.contains("above the 3 byte limit"));

    let output_limit_error = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: "1234".to_string(),
        engine_path: Some(graphviz_path.clone()),
        trusted: true,
        input_mode: Some("stdin".to_string()),
        output_format: None,
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(3),
    })
    .unwrap_err();
    assert!(output_limit_error.contains("output is"));
    assert!(output_limit_error.contains("above the 3 byte limit"));

    let stdin_artifact = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: unique_body.clone(),
        engine_path: Some(graphviz_path.clone()),
        trusted: true,
        input_mode: Some("stdin".to_string()),
        output_format: None,
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(1024),
    })
    .expect("stdin external transform");
    assert_eq!(stdin_artifact.execution_kind, "external");
    assert_eq!(stdin_artifact.input_mode, "stdin");
    assert!(stdin_artifact.html.contains(&unique_body));
    assert!(!stdin_artifact.cache_key.is_empty());
    let success_diagnostic = stdin_artifact
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("completed"))
        .expect("success diagnostic");
    assert!(success_diagnostic
        .related
        .iter()
        .any(|related| related == &format!("cache_key: {}", stdin_artifact.cache_key)));
    assert!(success_diagnostic
        .related
        .iter()
        .any(|related| related == "input_mode: stdin"));
    assert!(success_diagnostic
        .related
        .iter()
        .any(|related| related == "adapter: graphviz"));
    assert!(success_diagnostic
        .related
        .iter()
        .any(|related| related == "adapter_args: -Tsvg"));
    assert!(success_diagnostic
        .related
        .iter()
        .any(|related| related == &format!("input_bytes: {}", unique_body.len())));
    assert!(success_diagnostic
        .related
        .iter()
        .any(|related| related.starts_with("output_bytes: ")));
    assert!(success_diagnostic
        .related
        .iter()
        .any(|related| related == "timeout_ms: 1000"));
    assert!(success_diagnostic
        .related
        .iter()
        .any(|related| related == "output_channel: stdout"));
    assert!(success_diagnostic
        .related
        .iter()
        .any(|related| related == "status: 0"));
    assert!(stdin_artifact
        .engine_version
        .as_deref()
        .is_some_and(|version| version.contains("file-size:")));
    let cached_artifact = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: unique_body.clone(),
        engine_path: Some(graphviz_path.clone()),
        trusted: true,
        input_mode: Some("stdin".to_string()),
        output_format: None,
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(1024),
    })
    .expect("cached stdin external transform");
    assert_eq!(cached_artifact.cache_key, stdin_artifact.cache_key);
    assert_eq!(cached_artifact.output_hash, stdin_artifact.output_hash);
    assert_eq!(
        cached_artifact.engine_version,
        stdin_artifact.engine_version
    );
    assert_eq!(cached_artifact.duration_ms, Some(0));
    assert!(cached_artifact
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("served from cache")));
    assert!(cached_artifact.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .related
            .iter()
            .any(|related| related == &format!("cache_key: {}", cached_artifact.cache_key))
    }));
    transforms::external::clear_external_transform_memory_cache_for_tests();
    let persistent_cached_artifact = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: unique_body,
        engine_path: Some(graphviz_path.clone()),
        trusted: true,
        input_mode: Some("stdin".to_string()),
        output_format: None,
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(1024),
    })
    .expect("persistent cached stdin external transform");
    assert_eq!(
        persistent_cached_artifact.cache_key,
        stdin_artifact.cache_key
    );
    assert_eq!(persistent_cached_artifact.duration_ms, Some(0));
    assert!(persistent_cached_artifact
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("persistent cache")));
    assert!(persistent_cached_artifact
        .diagnostics
        .iter()
        .any(|diagnostic| {
            diagnostic
                .related
                .iter()
                .any(|related| related.starts_with("cached_output_bytes: "))
        }));

    let file_artifact = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: "digraph {}".to_string(),
        engine_path: Some(graphviz_path),
        trusted: true,
        input_mode: Some("file".to_string()),
        output_format: None,
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(1024),
    })
    .expect("file external transform");
    assert_eq!(file_artifact.input_mode, "file");
    assert!(file_artifact.html.contains("digraph"));
    assert!(file_artifact.html.contains("-Tsvg"));
    let _ = fs::remove_file(graphviz);
}

#[cfg(unix)]
#[test]
fn external_transform_adapters_shape_engine_specific_invocations() {
    let d2 = write_executable_script(
        "d2-adapter",
        "#!/bin/sh\ncat >/dev/null\nprintf '<svg data-args=\"%s\">d2</svg>' \"$*\"\n",
    );
    let d2_artifact = run_external_transform(ExternalTransformRequest {
        name: "d2".to_string(),
        body: "source -> target".to_string(),
        engine_path: Some(path_to_string(&d2)),
        trusted: true,
        input_mode: Some("stdin".to_string()),
        output_format: None,
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(2048),
    })
    .expect("d2 adapter transform");
    assert!(d2_artifact.html.contains("data-args=\"- -\""));
    assert!(d2_artifact.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .related
            .iter()
            .any(|related| related == "adapter: d2")
    }));

    let plantuml = write_executable_script(
            "plantuml-adapter",
            "#!/bin/sh\nlast=\"\"\nfor arg in \"$@\"; do last=\"$arg\"; done\nout=\"${last%.*}.svg\"\nprintf '<svg data-args=\"%s\">plantuml sidecar</svg>' \"$*\" > \"$out\"\n",
        );
    let plantuml_artifact = run_external_transform(ExternalTransformRequest {
        name: "plantuml".to_string(),
        body: "@startuml\nAlice -> Bob: hi\n@enduml".to_string(),
        engine_path: Some(path_to_string(&plantuml)),
        trusted: true,
        input_mode: Some("file".to_string()),
        output_format: None,
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(2048),
    })
    .expect("plantuml file adapter transform");
    assert!(plantuml_artifact.html.contains("plantuml sidecar"));
    assert!(plantuml_artifact.html.contains("-tsvg"));
    assert_eq!(plantuml_artifact.input_mode, "file");
    assert!(plantuml_artifact.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .related
            .iter()
            .any(|related| related == "adapter: plantuml")
    }));
    assert!(plantuml_artifact.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .related
            .iter()
            .any(|related| related == "output_channel: sidecar svg")
    }));

    let plantuml_png = write_executable_script(
            "plantuml-png-adapter",
            "#!/bin/sh\nlast=\"\"\nfor arg in \"$@\"; do last=\"$arg\"; done\nout=\"${last%.*}.png\"\nprintf 'png-bytes' > \"$out\"\n",
        );
    let plantuml_png_artifact = run_external_transform(ExternalTransformRequest {
        name: "plantuml".to_string(),
        body: "@startuml\nAlice -> Bob: hi\n@enduml".to_string(),
        engine_path: Some(path_to_string(&plantuml_png)),
        trusted: true,
        input_mode: Some("file".to_string()),
        output_format: Some("png".to_string()),
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(2048),
    })
    .expect("plantuml png file adapter transform");
    assert_eq!(plantuml_png_artifact.output_kind, "png");
    assert!(plantuml_png_artifact
        .html
        .contains("data:image/png;base64,cG5nLWJ5dGVz"));
    assert!(plantuml_png_artifact
        .html
        .contains("transform-plantuml-png"));
    assert!(plantuml_png_artifact.diagnostics.iter().any(|diagnostic| {
        diagnostic.related.iter().any(|related| {
            related.starts_with("adapter_args: -tpng ")
                && related.contains("neditor-plantuml-")
                && related.ends_with(".puml")
        })
    }));
    assert!(plantuml_png_artifact.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .related
            .iter()
            .any(|related| related == "output_channel: sidecar png")
    }));
    assert!(plantuml_png_artifact.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .related
            .iter()
            .any(|related| related == "output_format: png")
    }));

    use std::os::unix::fs::PermissionsExt;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let pikchr_cli = std::env::temp_dir().join(format!("pikchr-cli-{unique}.sh"));
    fs::write(
        &pikchr_cli,
        "#!/bin/sh\nif [ \"$#\" -ne 1 ]; then echo 'missing source path' >&2; exit 2; fi\nif [ ! -f \"$1\" ]; then echo 'source path missing' >&2; exit 1; fi\nprintf '<svg data-args=\"%s\">' \"$#\"\ncat \"$1\"\nprintf '</svg>'\n",
    )
    .expect("write fake pikchr-cli");
    let mut permissions = fs::metadata(&pikchr_cli)
        .expect("fake pikchr-cli metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&pikchr_cli, permissions).expect("make fake pikchr-cli executable");
    let pikchr_body = "box \"CI\"; arrow; box \"Done\"".to_string();
    let pikchr_artifact = run_external_transform(ExternalTransformRequest {
        name: "pikchr".to_string(),
        body: pikchr_body.clone(),
        engine_path: Some(path_to_string(&pikchr_cli)),
        trusted: true,
        input_mode: Some("stdin".to_string()),
        output_format: None,
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(2048),
    })
    .expect("pikchr-cli positional source path adapter transform");
    assert!(pikchr_artifact.html.contains(&pikchr_body));
    assert!(pikchr_artifact.diagnostics.iter().any(|diagnostic| {
        diagnostic.related.iter().any(|related| {
            related.starts_with("adapter_args: ")
                && related.contains("neditor-pikchr-")
                && related.ends_with(".pikchr")
        })
    }));

    let engines = list_transform_engines();
    let graphviz = engines
        .iter()
        .find(|engine| engine.get("name").and_then(Value::as_str) == Some("graphviz"))
        .expect("graphviz metadata");
    assert_eq!(
        graphviz.get("defaultCommand").and_then(Value::as_str),
        Some("dot")
    );
    assert!(graphviz
        .get("adapterProfile")
        .and_then(Value::as_str)
        .is_some_and(|profile| profile.contains("Graphviz DOT adapter")));
    assert_eq!(
        graphviz
            .pointer("/diagnosticProfile/versionProbe")
            .and_then(Value::as_str),
        Some("dot -V")
    );
    assert!(graphviz
        .pointer("/diagnosticProfile/failureHint")
        .and_then(Value::as_str)
        .is_some_and(|hint| hint.contains("Graphviz dot")));

    let neato = engines
        .iter()
        .find(|engine| engine.get("name").and_then(Value::as_str) == Some("neato"))
        .expect("neato metadata");
    assert_eq!(
        neato.get("defaultCommand").and_then(Value::as_str),
        Some("neato")
    );
    assert!(neato
        .get("adapterProfile")
        .and_then(Value::as_str)
        .is_some_and(|profile| profile.contains("Graphviz neato adapter")));
    assert_eq!(
        neato
            .pointer("/diagnosticProfile/versionProbe")
            .and_then(Value::as_str),
        Some("neato -V")
    );
    assert!(neato
        .pointer("/diagnosticProfile/failureHint")
        .and_then(Value::as_str)
        .is_some_and(|hint| hint.contains("Graphviz neato")));

    let _ = fs::remove_file(d2);
    let _ = fs::remove_file(plantuml);
    let _ = fs::remove_file(plantuml_png);
    let _ = fs::remove_file(pikchr_cli);
}

#[cfg(unix)]
#[test]
fn external_transform_cache_invalidates_when_trusted_executable_changes() {
    use std::os::unix::fs::PermissionsExt;

    let script = write_executable_script(
        "graphviz-cache-identity",
        "#!/bin/sh\ncat >/dev/null\nprintf '<svg>engine-v1</svg>'\n",
    );
    let script_path = path_to_string(&script);
    let body = format!(
        "digraph {{ cache_identity_{} }}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos()
    );

    let first = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: body.clone(),
        engine_path: Some(script_path.clone()),
        trusted: true,
        input_mode: Some("stdin".to_string()),
        output_format: None,
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(4096),
    })
    .expect("initial external transform");
    assert!(first.html.contains("engine-v1"));
    let first_version = first
        .engine_version
        .clone()
        .expect("first engine version identity");

    fs::write(
        &script,
        "#!/bin/sh\ncat >/dev/null\nprintf '<svg>engine-v2-with-new-size</svg>'\n# identity padding\n",
    )
    .expect("rewrite trusted executable");
    let mut permissions = fs::metadata(&script)
        .expect("rewritten script metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("keep rewritten executable");

    let second = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body,
        engine_path: Some(script_path),
        trusted: true,
        input_mode: Some("stdin".to_string()),
        output_format: None,
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(4096),
    })
    .expect("external transform after executable rewrite");

    assert!(second.html.contains("engine-v2-with-new-size"));
    assert_ne!(second.cache_key, first.cache_key);
    assert_ne!(second.output_hash, first.output_hash);
    assert_ne!(
        second.engine_version.as_deref(),
        Some(first_version.as_str())
    );
    assert!(second
        .engine_version
        .as_deref()
        .is_some_and(|version| version.contains("file-size:") && version.contains("mtime:")));
    assert!(second.diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains("completed")
            && diagnostic.related.iter().any(|related| {
                related.starts_with("engine_version: ")
                    && related.contains("file-size:")
                    && related.contains("mtime:")
            })
    }));
    assert!(!second
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("served from cache")));

    let _ = fs::remove_file(script);
}

#[test]
fn external_transform_conformance_runs_installed_engines() {
    struct EngineCase {
        name: &'static str,
        command: &'static str,
        env_var: &'static str,
        input_mode: &'static str,
        body: String,
    }

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let cases = [
        EngineCase {
            name: "dot",
            command: "dot",
            env_var: "NEDITOR_TEST_DOT",
            input_mode: "stdin",
            body: format!("digraph G {{ start -> done [label=\"{unique}\"]; }}"),
        },
        EngineCase {
            name: "circo",
            command: "circo",
            env_var: "NEDITOR_TEST_CIRCO",
            input_mode: "stdin",
            body: format!("graph G {{ start -- done [label=\"{unique}\"]; }}"),
        },
        EngineCase {
            name: "neato",
            command: "neato",
            env_var: "NEDITOR_TEST_NEATO",
            input_mode: "stdin",
            body: format!("graph G {{ start -- done [label=\"{unique}\"]; }}"),
        },
        EngineCase {
            name: "fdp",
            command: "fdp",
            env_var: "NEDITOR_TEST_FDP",
            input_mode: "stdin",
            body: format!("graph G {{ start -- done [label=\"{unique}\"]; }}"),
        },
        EngineCase {
            name: "osage",
            command: "osage",
            env_var: "NEDITOR_TEST_OSAGE",
            input_mode: "stdin",
            body: format!("graph G {{ start -- done [label=\"{unique}\"]; }}"),
        },
        EngineCase {
            name: "twopi",
            command: "twopi",
            env_var: "NEDITOR_TEST_TWOPI",
            input_mode: "stdin",
            body: format!("graph G {{ start -- done [label=\"{unique}\"]; }}"),
        },
        EngineCase {
            name: "d2",
            command: "d2",
            env_var: "NEDITOR_TEST_D2",
            input_mode: "stdin",
            body: format!("source -> target: {unique}"),
        },
        EngineCase {
            name: "plantuml",
            command: "plantuml",
            env_var: "NEDITOR_TEST_PLANTUML",
            input_mode: "file",
            body: format!("@startuml\nAlice -> Bob: {unique}\n@enduml\n"),
        },
        EngineCase {
            name: "pikchr",
            command: "pikchr",
            env_var: "NEDITOR_TEST_PIKCHR",
            input_mode: "stdin",
            body: format!("box \"{unique}\"; arrow; box \"Done\""),
        },
    ];

    let mut verified = Vec::new();
    let mut skipped = Vec::new();
    for case in cases {
        let Some(path) = installed_command_path(case.env_var, case.command) else {
            skipped.push(case.name);
            continue;
        };
        let artifact = run_external_transform(ExternalTransformRequest {
            name: case.name.to_string(),
            body: case.body,
            engine_path: Some(path_to_string(&path)),
            trusted: true,
            input_mode: Some(case.input_mode.to_string()),
            output_format: None,
            timeout_ms: Some(15_000),
            max_input_bytes: Some(16_384),
            max_output_bytes: Some(1_048_576),
        })
        .unwrap_or_else(|error| {
            panic!(
                "{} conformance failed with {}: {error}",
                case.name,
                path.display()
            )
        });

        assert_eq!(artifact.execution_kind, "external");
        assert_eq!(artifact.input_mode, case.input_mode);
        assert_eq!(artifact.output_kind, "svg");
        assert!(artifact.html.contains("<svg"));
        let engine_path = path_to_string(&path);
        assert_eq!(artifact.engine_path.as_deref(), Some(engine_path.as_str()));
        assert!(artifact.diagnostics.iter().any(|diagnostic| {
            diagnostic.related.iter().any(|related| {
                related == &format!("adapter: {}", external_conformance_adapter(case.name))
            })
        }));
        assert!(artifact.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .related
                .iter()
                .any(|related| related.starts_with("engine_version: file-size:"))
        }));
        verified.push(case.name);
    }

    eprintln!(
        "external transform conformance verified: {}; skipped: {}",
        verified.join(", "),
        skipped.join(", ")
    );
    if verified.is_empty() {
        eprintln!("No optional external transform engines were installed; set NEDITOR_TEST_DOT, NEDITOR_TEST_CIRCO, NEDITOR_TEST_NEATO, NEDITOR_TEST_FDP, NEDITOR_TEST_OSAGE, NEDITOR_TEST_TWOPI, NEDITOR_TEST_D2, NEDITOR_TEST_PLANTUML, or NEDITOR_TEST_PIKCHR to force a conformance run.");
    }
}

fn external_conformance_adapter(name: &str) -> &'static str {
    match name {
        "dot" | "circo" | "neato" | "fdp" | "osage" | "twopi" => "graphviz",
        "d2" => "d2",
        "plantuml" => "plantuml",
        "pikchr" => "pikchr",
        _ => "unknown",
    }
}

#[cfg(unix)]
#[test]
fn compiler_uses_trusted_external_transform_preferences() {
    let graphviz = write_executable_script(
        "compiler-graphviz-adapter",
        "#!/bin/sh\nprintf '<svg data-args=\"%s\">' \"$*\"\ncat\nprintf '</svg>'\n",
    );
    let response = compile_with_options(
        CompileRequest {
            text:
                "---\ntitle: External Dot\n---\n# External Dot\n```dot\ndigraph { a -> b }\n```\n"
                    .to_string(),
            file_path: None,
        },
        &json!({
            "transformEnginePaths": { "dot": path_to_string(&graphviz) },
            "trustedTransformEngines": { "dot": true },
            "transformInputModes": { "dot": "stdin" },
            "transformTimeoutMs": 1000
        }),
    );

    let artifact = response
        .transform_artifacts
        .iter()
        .find(|artifact| artifact.name == "dot")
        .expect("dot artifact");
    assert_eq!(artifact.execution_kind, "external");
    assert_eq!(artifact.input_mode, "stdin");
    assert!(artifact
        .engine_path
        .as_deref()
        .is_some_and(|path| path == path_to_string(&graphviz)));
    assert!(artifact.html.contains("digraph { a -> b }"));
    assert!(artifact.html.contains("-Tsvg"));
    assert!(response.html.contains("transform-external"));
    assert!(response.html.contains("transform-dot"));
    let ast_transform = response
        .document_ast
        .blocks
        .iter()
        .find_map(|block| match block {
            DocumentBlock::Transform {
                name,
                execution_kind,
                ..
            } if name == "dot" => Some(execution_kind),
            _ => None,
        })
        .expect("dot AST transform");
    assert_eq!(ast_transform.as_deref(), Some("external"));
    assert!(response.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("dot external transform completed")));
    let _ = fs::remove_file(graphviz);
}

#[cfg(unix)]
#[test]
fn compiler_uses_dot_settings_for_graphviz_alias() {
    let dot = write_executable_script(
        "compiler-graphviz-alias-adapter",
        "#!/bin/sh\nprintf '<svg data-args=\"%s\">' \"$*\"\ncat\nprintf '</svg>'\n",
    );
    let response = compile_with_options(
        CompileRequest {
            text: "---\ntitle: Graphviz Alias\n---\n# Graphviz Alias\n```graphviz\ndigraph { alias -> dot }\n```\n".to_string(),
            file_path: None,
        },
        &json!({
            "transformEnginePaths": { "dot": path_to_string(&dot) },
            "trustedTransformEngines": { "dot": true },
            "transformInputModes": { "dot": "stdin" },
            "transformTimeoutMs": 1000
        }),
    );

    let artifact = response
        .transform_artifacts
        .iter()
        .find(|artifact| artifact.name == "graphviz")
        .expect("graphviz artifact");
    assert_eq!(artifact.execution_kind, "external");
    assert_eq!(artifact.input_mode, "stdin");
    assert!(artifact
        .engine_path
        .as_deref()
        .is_some_and(|path| path == path_to_string(&dot)));
    assert!(artifact.html.contains("digraph { alias -> dot }"));
    assert!(artifact.html.contains("-Tsvg"));
    assert!(response.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("graphviz external transform completed")));
    let _ = fs::remove_file(dot);
}

#[cfg(unix)]
#[test]
fn compiler_uses_trusted_graphviz_variant_transform_preferences() {
    let neato = write_executable_script(
        "compiler-neato-adapter",
        "#!/bin/sh\nprintf '<svg data-args=\"%s\">' \"$*\"\ncat\nprintf '</svg>'\n",
    );
    let response = compile_with_options(
        CompileRequest {
            text:
                "---\ntitle: External Neato\n---\n# External Neato\n```neato\ngraph { a -- b }\n```\n"
                    .to_string(),
            file_path: None,
        },
        &json!({
            "transformEnginePaths": { "neato": path_to_string(&neato) },
            "trustedTransformEngines": { "neato": true },
            "transformInputModes": { "neato": "stdin" },
            "transformTimeoutMs": 1000
        }),
    );

    let artifact = response
        .transform_artifacts
        .iter()
        .find(|artifact| artifact.name == "neato")
        .expect("neato artifact");
    assert_eq!(artifact.execution_kind, "external");
    assert_eq!(artifact.input_mode, "stdin");
    assert!(artifact
        .engine_path
        .as_deref()
        .is_some_and(|path| path == path_to_string(&neato)));
    assert!(artifact.html.contains("graph { a -- b }"));
    assert!(artifact.html.contains("-Tsvg"));
    assert!(response.html.contains("transform-external"));
    assert!(response.html.contains("transform-neato"));
    assert!(response.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("neato external transform completed")));
    let _ = fs::remove_file(neato);
}

#[cfg(unix)]
#[test]
fn compiler_uses_plantuml_png_fence_output_format() {
    let plantuml = write_executable_script(
            "compiler-plantuml-png-adapter",
            "#!/bin/sh\nlast=\"\"\nfor arg in \"$@\"; do last=\"$arg\"; done\nout=\"${last%.*}.png\"\nprintf 'png-bytes' > \"$out\"\n",
        );
    let response = compile_with_options(
        CompileRequest {
            text:
                "---\ntitle: PlantUML PNG\n---\n# PlantUML PNG\n```plantuml format=png\n@startuml\nAlice -> Bob: hi\n@enduml\n```\n"
                    .to_string(),
            file_path: None,
        },
        &json!({
            "transformEnginePaths": { "plantuml": path_to_string(&plantuml) },
            "trustedTransformEngines": { "plantuml": true },
            "transformInputModes": { "plantuml": "file" },
            "transformTimeoutMs": 1000
        }),
    );

    let artifact = response
        .transform_artifacts
        .iter()
        .find(|artifact| artifact.name == "plantuml")
        .expect("plantuml artifact");
    assert_eq!(artifact.execution_kind, "external");
    assert_eq!(artifact.output_kind, "png");
    assert_eq!(artifact.input_mode, "file");
    assert_eq!(
        artifact.options.get("format").and_then(Value::as_str),
        Some("png")
    );
    assert!(artifact.html.contains("data:image/png;base64,cG5nLWJ5dGVz"));
    assert!(response.html.contains("data-output-kind=\"png\""));
    assert!(response.html.contains("transform-plantuml-png"));
    assert!(response.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("plantuml external transform completed")));
    let _ = fs::remove_file(plantuml);
}

#[test]
fn compiler_falls_back_when_external_transform_is_untrusted() {
    let cat = Path::new("/bin/cat");
    if !cat.exists() {
        return;
    }
    let response = compile_with_options(
        CompileRequest {
            text:
                "---\ntitle: Untrusted Dot\n---\n# Untrusted Dot\n```dot\ndigraph { a -> b }\n```\n"
                    .to_string(),
            file_path: None,
        },
        &json!({
            "transformEnginePaths": { "dot": path_to_string(cat) },
            "trustedTransformEngines": { "dot": false }
        }),
    );

    let artifact = response
        .transform_artifacts
        .iter()
        .find(|artifact| artifact.name == "dot")
        .expect("dot artifact");
    assert_eq!(artifact.execution_kind, "embedded");
    assert_eq!(artifact.output_kind, "svg");
    assert!(!artifact.html.contains("transform-pending"));
    assert!(artifact.html.contains("transform-dot"));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Transform { name, execution_kind, .. }
                if name == "dot" && execution_kind.as_deref() == Some("embedded")
        )
    }));
    assert!(response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("dot external transform failed")));
}

#[test]
fn compiler_reports_missing_external_engine_path_before_embedded_fallback() {
    let response = compile_with_options(
        CompileRequest {
            text:
                "---\ntitle: Missing Dot Path\n---\n# Missing Dot Path\n```dot\ndigraph { a -> b }\n```\n"
                    .to_string(),
            file_path: None,
        },
        &json!({}),
    );

    let artifact = response
        .transform_artifacts
        .iter()
        .find(|artifact| artifact.name == "dot")
        .expect("dot artifact");
    assert_eq!(artifact.execution_kind, "embedded");
    assert_eq!(artifact.input_mode, "embedded");
    assert!(artifact.html.contains("transform-dot"));
    let diagnostic = response
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic
                .message
                .contains("dot external transform path is not configured")
        })
        .expect("missing external engine path diagnostic");
    assert_eq!(diagnostic.severity, "info");
    assert!(diagnostic
        .suggestion
        .as_deref()
        .is_some_and(|suggestion| suggestion.contains("Configure and trust")));
}

#[test]
fn compiler_skips_disabled_external_transform_without_trust_warning() {
    let cat = Path::new("/bin/cat");
    if !cat.exists() {
        return;
    }
    let response = compile_with_options(
        CompileRequest {
            text:
                "---\ntitle: Disabled Dot\n---\n# Disabled Dot\n```dot\ndigraph { a -> b }\n```\n"
                    .to_string(),
            file_path: None,
        },
        &json!({
            "transformEnginePaths": { "dot": path_to_string(cat) },
            "trustedTransformEngines": { "dot": false },
            "disabledTransformEngines": { "dot": true }
        }),
    );

    let artifact = response
        .transform_artifacts
        .iter()
        .find(|artifact| artifact.name == "dot")
        .expect("dot artifact");
    assert_eq!(artifact.execution_kind, "embedded");
    assert_eq!(artifact.input_mode, "embedded");
    assert!(artifact.html.contains("transform-dot"));
    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("external transform failed")));
    assert!(response.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == "info"
            && diagnostic
                .message
                .contains("dot external transform disabled; using embedded renderer.")
    }));
}

#[cfg(unix)]
#[test]
fn external_transform_rejects_non_executable_engine_path() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let script = std::env::temp_dir().join(format!("neditor-not-executable-{unique}.sh"));
    fs::write(&script, "#!/bin/sh\ncat\n").expect("write non-executable script");

    let error = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: "digraph {}".to_string(),
        engine_path: Some(path_to_string(&script)),
        trusted: true,
        input_mode: Some("stdin".to_string()),
        output_format: None,
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(1024),
    })
    .unwrap_err();

    let _ = fs::remove_file(script);
    assert!(error.contains("not executable"));
}

#[cfg(unix)]
#[test]
fn external_transform_timeout_covers_blocked_stdin() {
    use std::os::unix::fs::PermissionsExt;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let script = std::env::temp_dir().join(format!("neditor-blocked-stdin-{unique}.sh"));
    fs::write(&script, "#!/bin/sh\nsleep 2\n").expect("write blocked stdin script");
    let mut permissions = fs::metadata(&script)
        .expect("script metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("make script executable");

    let started = std::time::Instant::now();
    let error = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: "x".repeat(512 * 1024),
        engine_path: Some(path_to_string(&script)),
        trusted: true,
        input_mode: Some("stdin".to_string()),
        output_format: None,
        timeout_ms: Some(50),
        max_input_bytes: Some(1024 * 1024),
        max_output_bytes: Some(1024),
    })
    .unwrap_err();

    let _ = fs::remove_file(script);
    assert!(error.contains("timed out"));
    assert!(
        started.elapsed() < std::time::Duration::from_secs(1),
        "blocked stdin write should not bypass the timeout"
    );
}

#[cfg(unix)]
#[test]
fn external_transform_exit_errors_include_stderr() {
    use std::os::unix::fs::PermissionsExt;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let script = std::env::temp_dir().join(format!("neditor-stderr-exit-{unique}.sh"));
    fs::write(&script, "#!/bin/sh\necho engine exploded >&2\nexit 7\n")
        .expect("write stderr script");
    let mut permissions = fs::metadata(&script)
        .expect("script metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("make script executable");

    let error = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: "digraph {}".to_string(),
        engine_path: Some(path_to_string(&script)),
        trusted: true,
        input_mode: Some("file".to_string()),
        output_format: None,
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(1024),
    })
    .unwrap_err();

    let _ = fs::remove_file(script);
    assert!(error.contains("status 7"));
    assert!(error.contains("engine exploded"));
    assert!(error.contains("Check DOT syntax"));
    assert!(error.contains("-Tsvg"));
}
