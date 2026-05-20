use std::collections::BTreeSet;

#[test]
fn spec_25_4_ipc_commands_are_registered_and_documented() {
    let required = spec_25_4_commands();
    assert!(
        !required.is_empty(),
        "specification section 25.4 should list required IPC commands"
    );

    let registered = registered_tauri_commands();
    let documented = documented_ipc_commands();

    assert_subset(
        "registered Tauri commands",
        &required,
        &registered,
        "src-tauri/src/lib.rs tauri::generate_handler!",
    );
    assert_subset(
        "documented IPC coverage commands",
        &required,
        &documented,
        "docs/ipc-command-coverage.md",
    );
    assert_eq!(
        required, documented,
        "docs/ipc-command-coverage.md should cover the section 25.4 commands exactly"
    );
}

fn spec_25_4_commands() -> BTreeSet<String> {
    let spec = include_str!("../../../docs/specification.md");
    let section = spec
        .split("### 25.4 IPC Commands")
        .nth(1)
        .expect("specification should contain section 25.4")
        .split("## 26.")
        .next()
        .expect("section 25.4 should end before section 26");
    markdown_backtick_list_items(section)
}

fn documented_ipc_commands() -> BTreeSet<String> {
    let coverage = include_str!("../../../docs/ipc-command-coverage.md");
    coverage
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("| `") {
                return None;
            }
            trimmed
                .trim_start_matches("| `")
                .split('`')
                .next()
                .map(str::to_string)
        })
        .collect()
}

fn registered_tauri_commands() -> BTreeSet<String> {
    let lib = include_str!("../lib.rs");
    let handler = lib
        .split(".invoke_handler(tauri::generate_handler![")
        .nth(1)
        .expect("lib.rs should register a Tauri invoke handler")
        .split("])")
        .next()
        .expect("invoke handler should close the generate_handler macro");
    handler
        .lines()
        .filter_map(|line| {
            let command = line.trim().trim_end_matches(',');
            if command.is_empty() || command.contains(' ') {
                return None;
            }
            Some(command.to_string())
        })
        .collect()
}

fn markdown_backtick_list_items(section: &str) -> BTreeSet<String> {
    section
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("- `")
                .and_then(|rest| rest.split('`').next())
                .map(str::to_string)
        })
        .collect()
}

fn assert_subset(
    label: &str,
    required: &BTreeSet<String>,
    actual: &BTreeSet<String>,
    source: &str,
) {
    let missing = required.difference(actual).cloned().collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} are missing required commands from {source}: {missing:?}"
    );
}
