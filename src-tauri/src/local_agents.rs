use crate::{path_to_string, sha256_hex};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
pub(crate) struct PrepareLocalAgentHandoffRequest {
    pub(crate) profile_id: String,
    pub(crate) prompt_markdown: String,
    pub(crate) workspace_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ImportLocalAgentResponseRequest {
    pub(crate) profile_id: String,
    pub(crate) workspace_path: Option<String>,
    pub(crate) response_path: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct LocalAgentHandoffResponse {
    pub(crate) profile_id: String,
    pub(crate) label: String,
    pub(crate) command: String,
    pub(crate) available: bool,
    pub(crate) executable_path: Option<String>,
    pub(crate) workspace_path: String,
    pub(crate) handoff_path: String,
    pub(crate) response_path: String,
    pub(crate) launch_command: Vec<String>,
    pub(crate) instructions: Vec<String>,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct LocalAgentResponseImport {
    pub(crate) profile_id: String,
    pub(crate) label: String,
    pub(crate) response_path: String,
    pub(crate) markdown: String,
    pub(crate) sha256: String,
    pub(crate) characters: usize,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct LocalAgentSpec {
    profile_id: &'static str,
    label: &'static str,
    command: &'static str,
}

const LOCAL_AGENT_SPECS: &[LocalAgentSpec] = &[
    LocalAgentSpec {
        profile_id: "claude-code-cli",
        label: "Claude Code",
        command: "claude",
    },
    LocalAgentSpec {
        profile_id: "codex-cli",
        label: "Codex",
        command: "codex",
    },
    LocalAgentSpec {
        profile_id: "opencode-cli",
        label: "OpenCode",
        command: "opencode",
    },
    LocalAgentSpec {
        profile_id: "google-antigravity-cli",
        label: "Google Antigravity",
        command: "antigravity",
    },
];

#[tauri::command]
pub(crate) fn prepare_local_agent_handoff(
    request: PrepareLocalAgentHandoffRequest,
) -> Result<LocalAgentHandoffResponse, String> {
    let spec = local_agent_spec(&request.profile_id).ok_or_else(|| {
        "Unsupported local agent profile. Choose Claude Code, Codex, OpenCode, or Google Antigravity."
            .to_string()
    })?;
    let prompt = request.prompt_markdown.trim();
    if prompt.is_empty() {
        return Err("Cannot prepare an empty local agent handoff.".to_string());
    }

    let workspace_path = resolve_workspace_path(request.workspace_path.as_deref())?;
    let handoff_dir = agent_handoff_dir(&workspace_path);
    fs::create_dir_all(&handoff_dir).map_err(|err| {
        format!(
            "Cannot create local agent handoff folder {}: {err}",
            path_to_string(&handoff_dir)
        )
    })?;
    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let handoff_path = handoff_dir.join(agent_file_name(spec.profile_id, &timestamp, "prompt"));
    let response_path = handoff_dir.join(agent_file_name(spec.profile_id, &timestamp, "response"));
    let response_path_text = path_to_string(&response_path);
    let handoff_markdown = local_agent_handoff_markdown(spec, prompt, &response_path_text);
    fs::write(&handoff_path, handoff_markdown.as_bytes()).map_err(|err| {
        format!(
            "Cannot write local agent handoff {}: {err}",
            path_to_string(&handoff_path)
        )
    })?;

    let executable_path = find_executable(spec.command);
    let available = executable_path.is_some();
    let mut warnings = Vec::new();
    if !available {
        warnings.push(format!(
            "{} was not found on PATH. Install it or launch it manually, then open the prepared handoff file.",
            spec.command
        ));
    }

    Ok(LocalAgentHandoffResponse {
        profile_id: spec.profile_id.to_string(),
        label: spec.label.to_string(),
        command: spec.command.to_string(),
        available,
        executable_path: executable_path.as_deref().map(path_to_string),
        workspace_path: path_to_string(&workspace_path),
        handoff_path: path_to_string(&handoff_path),
        response_path: response_path_text.clone(),
        launch_command: vec![spec.command.to_string()],
        instructions: vec![
            format!("Start {} from the workspace path below.", spec.label),
            "Open or paste the prepared Markdown handoff file into the local agent.".to_string(),
            format!("Ask the agent to write Markdown only to {response_path_text}."),
            "Return provenance, review notes, unresolved assumptions, and QA checklist items with the response.".to_string(),
            "Use Import local response in NEditor before accepting any returned content.".to_string(),
        ],
        warnings,
    })
}

#[tauri::command]
pub(crate) fn import_local_agent_response(
    request: ImportLocalAgentResponseRequest,
) -> Result<LocalAgentResponseImport, String> {
    let spec = local_agent_spec(&request.profile_id).ok_or_else(|| {
        "Unsupported local agent profile. Choose Claude Code, Codex, OpenCode, or Google Antigravity."
            .to_string()
    })?;
    let workspace_path = resolve_workspace_path(request.workspace_path.as_deref())?;
    let handoff_dir = agent_handoff_dir(&workspace_path)
        .canonicalize()
        .map_err(|err| format!("Cannot resolve local agent handoff folder: {err}"))?;
    let response_path = PathBuf::from(request.response_path.trim());
    if response_path.as_os_str().is_empty() {
        return Err("Choose the local agent response Markdown file to import.".to_string());
    }
    let canonical_response = response_path.canonicalize().map_err(|err| {
        format!(
            "Cannot read local agent response {}: {err}",
            path_to_string(&response_path)
        )
    })?;
    if !canonical_response.starts_with(&handoff_dir) {
        return Err(format!(
            "Local agent response must stay inside {}.",
            path_to_string(&handoff_dir)
        ));
    }
    if canonical_response
        .extension()
        .and_then(|value| value.to_str())
        != Some("md")
    {
        return Err("Local agent responses must be Markdown .md files.".to_string());
    }
    let markdown = fs::read_to_string(&canonical_response).map_err(|err| {
        format!(
            "Cannot read local agent response {}: {err}",
            path_to_string(&canonical_response)
        )
    })?;
    if markdown.trim().is_empty() {
        return Err("Local agent response file is empty.".to_string());
    }
    let mut warnings = Vec::new();
    if !markdown.contains("```ai-source") && !markdown.contains("<!-- ai-assisted:") {
        warnings.push(
            "Response did not include explicit AI provenance; NEditor will wrap it before apply."
                .to_string(),
        );
    }
    if !markdown.to_ascii_lowercase().contains("review") {
        warnings.push(
            "Response did not mention review; confirm claims and assumptions before accepting."
                .to_string(),
        );
    }
    Ok(LocalAgentResponseImport {
        profile_id: spec.profile_id.to_string(),
        label: spec.label.to_string(),
        response_path: path_to_string(&canonical_response),
        sha256: sha256_hex(markdown.as_bytes()),
        characters: markdown.chars().count(),
        markdown,
        warnings,
    })
}

fn local_agent_spec(profile_id: &str) -> Option<&'static LocalAgentSpec> {
    LOCAL_AGENT_SPECS
        .iter()
        .find(|spec| spec.profile_id == profile_id.trim())
}

fn resolve_workspace_path(workspace_path: Option<&str>) -> Result<PathBuf, String> {
    let raw_path = workspace_path
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let candidate = if raw_path.extension().is_some() {
        raw_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        raw_path
    };
    if !candidate.exists() {
        return Err(format!(
            "Local agent workspace folder does not exist: {}",
            path_to_string(&candidate)
        ));
    }
    if !candidate.is_dir() {
        return Err(format!(
            "Local agent workspace path is not a folder: {}",
            path_to_string(&candidate)
        ));
    }
    candidate
        .canonicalize()
        .map_err(|err| format!("Cannot resolve local agent workspace: {err}"))
}

fn agent_handoff_dir(workspace_path: &Path) -> PathBuf {
    workspace_path.join(".neditor").join("agent-handoffs")
}

fn agent_file_name(profile_id: &str, timestamp: &str, role: &str) -> String {
    format!("neditor-{profile_id}-{timestamp}.{role}.md")
}

fn local_agent_handoff_markdown(
    spec: &LocalAgentSpec,
    prompt: &str,
    response_path: &str,
) -> String {
    [
        "# NEditor Local Agent Handoff".to_string(),
        String::new(),
        format!("Agent: {}", spec.label),
        format!("Command: {}", spec.command),
        format!("Required response file: `{response_path}`"),
        String::new(),
        "## Response Contract".to_string(),
        String::new(),
        "- Return Markdown only.".to_string(),
        "- Write the final response to the required response file path above.".to_string(),
        "- Preserve source-sensitive claims, citations, unresolved assumptions, AI provenance, QA notes, and review handoff instructions.".to_string(),
        "- Do not mark content final; NEditor will import the response as needs-review material.".to_string(),
        String::new(),
        "## Request Package".to_string(),
        String::new(),
        prompt.to_string(),
        String::new(),
    ]
    .join("\n")
}

fn find_executable(command: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    find_executable_in_paths(command, env::split_paths(&path_var).collect())
}

fn find_executable_in_paths(command: &str, paths: Vec<PathBuf>) -> Option<PathBuf> {
    for dir in paths {
        for candidate in executable_candidates(&dir, command) {
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn executable_candidates(dir: &Path, command: &str) -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        let mut candidates = vec![dir.join(command)];
        if !command.to_ascii_lowercase().ends_with(".exe") {
            candidates.push(dir.join(format!("{command}.exe")));
        }
        candidates
    }
    #[cfg(not(windows))]
    {
        vec![dir.join(command)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn rejects_unknown_local_agent_profiles() {
        let result = prepare_local_agent_handoff(PrepareLocalAgentHandoffRequest {
            profile_id: "bash".to_string(),
            prompt_markdown: "# prompt".to_string(),
            workspace_path: None,
        });
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Unsupported local agent profile"));
    }

    #[test]
    fn allowlists_expected_local_agent_profiles() {
        let expected = [
            ("claude-code-cli", "Claude Code", "claude"),
            ("codex-cli", "Codex", "codex"),
            ("opencode-cli", "OpenCode", "opencode"),
            (
                "google-antigravity-cli",
                "Google Antigravity",
                "antigravity",
            ),
        ];

        for (profile_id, label, command) in expected {
            let spec = local_agent_spec(profile_id).unwrap();
            assert_eq!(spec.label, label);
            assert_eq!(spec.command, command);
        }
    }

    #[test]
    fn rejects_empty_local_agent_handoff_prompts() {
        let result = prepare_local_agent_handoff(PrepareLocalAgentHandoffRequest {
            profile_id: "codex-cli".to_string(),
            prompt_markdown: " \n\t ".to_string(),
            workspace_path: None,
        });

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Cannot prepare an empty local agent handoff"));
    }

    #[test]
    fn prepares_handoff_file_inside_workspace() {
        let workspace = unique_temp_workspace("neditor-local-agent");
        fs::create_dir_all(&workspace).unwrap();

        let response = prepare_local_agent_handoff(PrepareLocalAgentHandoffRequest {
            profile_id: "codex-cli".to_string(),
            prompt_markdown: "# Codex handoff\n\nReturn Markdown only.".to_string(),
            workspace_path: Some(path_to_string(&workspace)),
        })
        .unwrap();

        let handoff_path = PathBuf::from(&response.handoff_path);
        assert_eq!(response.command, "codex");
        assert_eq!(response.launch_command, vec!["codex".to_string()]);
        assert!(response.response_path.ends_with(".response.md"));
        assert!(handoff_path.starts_with(
            workspace
                .canonicalize()
                .unwrap()
                .join(".neditor")
                .join("agent-handoffs")
        ));
        assert!(fs::read_to_string(&handoff_path)
            .unwrap()
            .contains("Required response file"));
        assert!(fs::read_to_string(handoff_path)
            .unwrap()
            .contains("Return Markdown only."));

        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn imports_local_agent_response_from_handoff_folder() {
        let workspace = unique_temp_workspace("neditor-local-agent-response");
        fs::create_dir_all(&workspace).unwrap();

        let handoff = prepare_local_agent_handoff(PrepareLocalAgentHandoffRequest {
            profile_id: "opencode-cli".to_string(),
            prompt_markdown: "# OpenCode handoff\n\nReturn Markdown only.".to_string(),
            workspace_path: Some(path_to_string(&workspace)),
        })
        .unwrap();
        fs::write(
            &handoff.response_path,
            "## Revised Draft\n\n```ai-source\nprovider: OpenCode\nstatus: needs-review\n```\n\nReview notes included.",
        )
        .unwrap();

        let response = import_local_agent_response(ImportLocalAgentResponseRequest {
            profile_id: "opencode-cli".to_string(),
            workspace_path: Some(path_to_string(&workspace)),
            response_path: handoff.response_path,
        })
        .unwrap();

        assert_eq!(response.profile_id, "opencode-cli");
        assert_eq!(response.label, "OpenCode");
        assert!(response.markdown.contains("Revised Draft"));
        assert_eq!(response.sha256.len(), 64);
        assert!(response.warnings.is_empty());

        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn blocks_local_agent_response_outside_handoff_folder() {
        let workspace = unique_temp_workspace("neditor-local-agent-response-boundary");
        fs::create_dir_all(&workspace).unwrap();
        fs::create_dir_all(agent_handoff_dir(&workspace)).unwrap();
        let outside = workspace.with_extension("md");
        fs::write(&outside, "# Outside").unwrap();

        let error = import_local_agent_response(ImportLocalAgentResponseRequest {
            profile_id: "codex-cli".to_string(),
            workspace_path: Some(path_to_string(&workspace)),
            response_path: path_to_string(&outside),
        })
        .unwrap_err();

        assert!(error.contains("must stay inside"));

        let _ = fs::remove_dir_all(workspace);
        let _ = fs::remove_file(outside);
    }

    #[test]
    fn document_workspace_paths_resolve_to_their_parent_folder() {
        let workspace = unique_temp_workspace("neditor-local-agent-document-parent");
        fs::create_dir_all(&workspace).unwrap();
        let document_path = workspace.join("proposal.md");
        fs::write(&document_path, "# Proposal").unwrap();

        let response = prepare_local_agent_handoff(PrepareLocalAgentHandoffRequest {
            profile_id: "claude-code-cli".to_string(),
            prompt_markdown: "# Claude handoff\n\nReview this proposal.".to_string(),
            workspace_path: Some(path_to_string(&document_path)),
        })
        .unwrap();

        assert_eq!(response.command, "claude");
        assert_eq!(
            response.workspace_path,
            path_to_string(&workspace.canonicalize().unwrap())
        );
        assert!(PathBuf::from(&response.handoff_path).starts_with(
            workspace
                .canonicalize()
                .unwrap()
                .join(".neditor")
                .join("agent-handoffs")
        ));

        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn prepares_google_antigravity_handoff_file_inside_workspace() {
        let workspace = unique_temp_workspace("neditor-antigravity-agent");
        fs::create_dir_all(&workspace).unwrap();

        let response = prepare_local_agent_handoff(PrepareLocalAgentHandoffRequest {
            profile_id: "google-antigravity-cli".to_string(),
            prompt_markdown: "# Antigravity handoff\n\nReturn reviewed Markdown only.".to_string(),
            workspace_path: Some(path_to_string(&workspace)),
        })
        .unwrap();

        let handoff_path = PathBuf::from(&response.handoff_path);
        assert_eq!(response.profile_id, "google-antigravity-cli");
        assert_eq!(response.label, "Google Antigravity");
        assert_eq!(response.command, "antigravity");
        assert_eq!(response.launch_command, vec!["antigravity".to_string()]);
        assert!(response.response_path.contains("google-antigravity-cli"));
        assert!(handoff_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("neditor-google-antigravity-cli-"));
        assert!(fs::read_to_string(handoff_path)
            .unwrap()
            .contains("Return reviewed Markdown only."));

        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn resolves_known_executable_in_paths() {
        let workspace = unique_temp_workspace("neditor-local-agent-path");
        fs::create_dir_all(&workspace).unwrap();
        let executable = workspace.join(executable_file_name("opencode"));
        fs::write(&executable, b"").unwrap();

        let found = find_executable_in_paths("opencode", vec![workspace.clone()]);
        assert_eq!(found, Some(executable));

        let _ = fs::remove_dir_all(workspace);
    }

    fn unique_temp_workspace(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("{prefix}-{nanos}"))
    }

    fn executable_file_name(command: &str) -> String {
        #[cfg(windows)]
        {
            format!("{command}.exe")
        }
        #[cfg(not(windows))]
        {
            command.to_string()
        }
    }
}
