#[test]
fn transform_handler_installer_plans_cover_supported_platforms() {
    for platform in ["macos", "windows", "linux"] {
        let plans =
            crate::transform_install::transform_handler_installer_plans_for_platform(platform);
        assert!(
            !plans.is_empty(),
            "{platform} should expose an installer plan"
        );
        let joined_handlers = plans
            .iter()
            .flat_map(|plan| plan.handlers.iter())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
            .to_lowercase();
        for handler in ["graphviz", "d2", "plantuml", "pikchr", "sqlite"] {
            assert!(
                joined_handlers.contains(handler),
                "{platform} installer plans should mention {handler}"
            );
        }
        assert!(
            plans.iter().all(|plan| !plan.commands.is_empty()),
            "{platform} installer plans should show user-visible commands"
        );
    }
}

#[test]
fn transform_handler_install_steps_are_allowlisted() {
    let macos = crate::transform_install::transform_handler_install_steps_for_platform(
        "macos",
        "macos-homebrew-all",
    )
    .expect("macOS steps");
    assert_eq!(macos.len(), 1);
    assert_eq!(macos[0].program, "brew");
    assert!(macos[0].args.contains(&"graphviz"));
    assert!(macos[0].args.contains(&"d2"));
    assert!(macos[0].args.contains(&"plantuml"));
    assert!(macos[0].args.contains(&"pikchr"));
    assert!(macos[0].args.contains(&"sqlite"));

    let windows = crate::transform_install::transform_handler_install_steps_for_platform(
        "windows",
        "windows-winget-all",
    )
    .expect("Windows steps");
    assert!(windows.iter().all(|step| step.program == "winget"));
    assert!(windows
        .iter()
        .flat_map(|step| step.args.iter())
        .any(|arg| *arg == "Graphviz.Graphviz"));
    assert!(windows
        .iter()
        .flat_map(|step| step.args.iter())
        .any(|arg| *arg == "Terrastruct.D2"));
    assert!(windows
        .iter()
        .flat_map(|step| step.args.iter())
        .any(|arg| *arg == "SQLite.SQLite"));

    for step in macos.iter().chain(windows.iter()) {
        assert_ne!(step.program, "sh");
        assert_ne!(step.program, "bash");
        assert!(!step.program.contains(' '));
        assert!(step.args.iter().all(|arg| !arg.contains('|')));
    }

    assert!(
        crate::transform_install::transform_handler_install_steps_for_platform(
            "linux",
            "linux-terminal-all",
        )
        .is_none(),
        "Linux plan is intentionally copy-only because it needs distro/admin-specific commands"
    );
}
