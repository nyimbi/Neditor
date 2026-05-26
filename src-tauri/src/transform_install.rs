use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TransformHandlerInstallerPlan {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) platform: String,
    pub(crate) manager: String,
    pub(crate) handlers: Vec<String>,
    pub(crate) commands: Vec<String>,
    pub(crate) installable: bool,
    pub(crate) requires_admin: bool,
    pub(crate) notes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TransformHandlerInstallRequest {
    pub(crate) plan_id: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct TransformHandlerInstallResponse {
    pub(crate) plan_id: String,
    pub(crate) started: bool,
    pub(crate) message: String,
    pub(crate) commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TransformHandlerInstallerStep {
    pub(crate) program: &'static str,
    pub(crate) args: Vec<&'static str>,
}

#[tauri::command]
pub(crate) fn list_transform_handler_installers() -> Vec<TransformHandlerInstallerPlan> {
    transform_handler_installer_plans_for_platform(std::env::consts::OS)
}

#[tauri::command]
pub(crate) fn install_transform_handlers(
    request: TransformHandlerInstallRequest,
) -> Result<TransformHandlerInstallResponse, String> {
    let platform = std::env::consts::OS;
    let plans = transform_handler_installer_plans_for_platform(platform);
    let plan = plans
        .iter()
        .find(|candidate| candidate.id == request.plan_id)
        .ok_or_else(|| "Unknown transform handler installer plan.".to_string())?;
    if !plan.installable {
        return Err("This installer plan is copy-only for the current platform.".to_string());
    }
    let steps = transform_handler_install_steps_for_platform(platform, &request.plan_id)
        .ok_or_else(|| "No allowlisted installer steps are available for this plan.".to_string())?;
    if steps.is_empty() {
        return Err("No transform handler installer steps are configured.".to_string());
    }
    let commands = steps.iter().map(format_installer_step).collect::<Vec<_>>();
    std::thread::spawn(move || {
        for step in steps {
            let _ = Command::new(step.program)
                .args(step.args)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    });

    Ok(TransformHandlerInstallResponse {
        plan_id: request.plan_id,
        started: true,
        message: "Started transform handler installation. Run Probe for each engine after the package manager finishes.".to_string(),
        commands,
    })
}

pub(crate) fn transform_handler_installer_plans_for_platform(
    platform: &str,
) -> Vec<TransformHandlerInstallerPlan> {
    match platform {
        "macos" => vec![homebrew_transform_handler_plan()],
        "windows" => vec![windows_transform_handler_plan()],
        "linux" => vec![linux_transform_handler_plan()],
        other => vec![manual_transform_handler_plan(other)],
    }
}

pub(crate) fn transform_handler_install_steps_for_platform(
    platform: &str,
    plan_id: &str,
) -> Option<Vec<TransformHandlerInstallerStep>> {
    match (platform, plan_id) {
        ("macos", "macos-homebrew-all") => Some(vec![TransformHandlerInstallerStep {
            program: "brew",
            args: vec![
                "install", "graphviz", "d2", "plantuml", "openjdk", "pikchr", "sqlite",
            ],
        }]),
        ("windows", "windows-winget-all") => Some(vec![
            winget_install("Graphviz.Graphviz"),
            winget_install("Terrastruct.D2"),
            winget_install("EclipseAdoptium.Temurin.21.JRE"),
            winget_install("SQLite.SQLite"),
        ]),
        _ => None,
    }
}

fn homebrew_transform_handler_plan() -> TransformHandlerInstallerPlan {
    plan(
        "macos-homebrew-all",
        "Install all handlers with Homebrew",
        "macOS",
        "Homebrew",
        vec![
            "Graphviz: dot, graphviz, circo, neato, fdp, osage, twopi",
            "D2",
            "PlantUML with Java runtime",
            "Pikchr",
            "SQLite sql transform",
        ],
        vec!["brew install graphviz d2 plantuml openjdk pikchr sqlite"],
        true,
        false,
        vec![
            "NEditor starts Homebrew directly without shell interpolation.",
            "After installation, run Probe beside each engine and trust only the executable path you expect.",
        ],
    )
}

fn windows_transform_handler_plan() -> TransformHandlerInstallerPlan {
    plan(
        "windows-winget-all",
        "Install core handlers with winget",
        "Windows",
        "winget",
        vec![
            "Graphviz: dot, graphviz, circo, neato, fdp, osage, twopi",
            "D2",
            "PlantUML Java runtime",
            "SQLite sql transform",
            "Pikchr: copy the note below if a local package is unavailable",
        ],
        vec![
            "winget install --id Graphviz.Graphviz -e --accept-package-agreements --accept-source-agreements",
            "winget install --id Terrastruct.D2 -e --accept-package-agreements --accept-source-agreements",
            "winget install --id EclipseAdoptium.Temurin.21.JRE -e --accept-package-agreements --accept-source-agreements",
            "winget install --id SQLite.SQLite -e --accept-package-agreements --accept-source-agreements",
        ],
        true,
        false,
        vec![
            "winget package availability can vary by source; if a handler is missing, use the copied commands as a starting point in a terminal.",
            "Install Pikchr manually or from source when your package source does not provide a trusted binary.",
        ],
    )
}

fn linux_transform_handler_plan() -> TransformHandlerInstallerPlan {
    plan(
        "linux-terminal-all",
        "Copy Linux package commands",
        "Linux",
        "system package manager",
        vec![
            "Graphviz: dot, graphviz, circo, neato, fdp, osage, twopi",
            "D2",
            "PlantUML with Java runtime",
            "Pikchr",
            "SQLite sql transform",
        ],
        vec![
            "sudo apt-get update",
            "sudo apt-get install -y graphviz default-jre plantuml sqlite3",
            "curl -fsSL https://d2lang.com/install.sh | sh -s --",
            "cargo install pikchr-cli --locked",
        ],
        false,
        true,
        vec![
            "Linux distributions use different package names and privilege prompts, so NEditor keeps this plan copy-only.",
            "Run the commands in a terminal, then return to NEditor to choose paths, trust engines, and probe them.",
        ],
    )
}

fn manual_transform_handler_plan(platform: &str) -> TransformHandlerInstallerPlan {
    plan(
        "manual-transform-handlers",
        "Copy transform handler checklist",
        platform,
        "manual",
        vec![
            "Graphviz: dot, graphviz, circo, neato, fdp, osage, twopi",
            "D2",
            "PlantUML with Java runtime",
            "Pikchr",
            "SQLite sql transform",
        ],
        vec![
            "Install Graphviz, D2, PlantUML plus Java, Pikchr, and SQLite using your platform package manager.",
            "Open NEditor settings, choose each executable path, mark the path trusted, and run Probe.",
        ],
        false,
        false,
        vec!["No native installer is available for this platform yet."],
    )
}

fn plan(
    id: &str,
    label: &str,
    platform: &str,
    manager: &str,
    handlers: Vec<&str>,
    commands: Vec<&str>,
    installable: bool,
    requires_admin: bool,
    notes: Vec<&str>,
) -> TransformHandlerInstallerPlan {
    TransformHandlerInstallerPlan {
        id: id.to_string(),
        label: label.to_string(),
        platform: platform.to_string(),
        manager: manager.to_string(),
        handlers: handlers.into_iter().map(str::to_string).collect(),
        commands: commands.into_iter().map(str::to_string).collect(),
        installable,
        requires_admin,
        notes: notes.into_iter().map(str::to_string).collect(),
    }
}

fn winget_install(id: &'static str) -> TransformHandlerInstallerStep {
    TransformHandlerInstallerStep {
        program: "winget",
        args: vec![
            "install",
            "--id",
            id,
            "-e",
            "--accept-package-agreements",
            "--accept-source-agreements",
        ],
    }
}

fn format_installer_step(step: &TransformHandlerInstallerStep) -> String {
    std::iter::once(step.program)
        .chain(step.args.iter().copied())
        .map(shell_display_token)
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_display_token(token: &str) -> String {
    if token
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "-_./:=+".contains(character))
    {
        return token.to_string();
    }
    format!("'{}'", token.replace('\'', "'\\''"))
}
