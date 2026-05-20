use super::*;
use std::io::{Cursor, Read};
use std::time::{SystemTime, UNIX_EPOCH};
use zip::ZipArchive;

#[cfg(unix)]
fn write_executable_script(prefix: &str, body: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("neditor-{prefix}-{unique}.sh"));
    fs::write(&path, body).expect("write executable test script");
    let mut permissions = fs::metadata(&path).expect("script metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("make script executable");
    path
}

fn installed_command_path(env_var: &str, command: &str) -> Option<PathBuf> {
    if let Some(path) = std::env::var_os(env_var)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute() && path.is_file())
    {
        return Some(path);
    }

    let path_value = std::env::var_os("PATH")?;
    for directory in std::env::split_paths(&path_value) {
        let direct = directory.join(command);
        if direct.is_file() {
            return Some(direct);
        }
        #[cfg(windows)]
        {
            for extension in ["exe", "bat", "cmd"] {
                let candidate = directory.join(format!("{command}.{extension}"));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

fn sample_document() -> String {
    r#"---
title: Test Report
version: 1.2.0
status: approved
approvedBy: QA
toc: true
client: Acme
brand:
  name: Acme
  logo: "data:image/svg+xml;base64,PHN2Zy8+"
---

# Test Report

[TOC]

Prepared for {{client}}.

```calc
revenue = 100
cost = 40
profit = revenue - cost
margin = profit / revenue
healthy = IF(revenue > cost, 1, 0)
target_met = IF(margin >= 0.60, 1, 0)
cost_match = IF(cost == 40, 1, 0)
spread = IF(revenue != cost, 1, 0)
discount = 12.5%
```

Margin: {{=margin | percent}}
After tax: {{=profit * 0.70 | currency}}
Healthy score: {{=IF(revenue > cost, profit, 0) | round}}
Discount: {{=discount | percent}}

```csv caption="Regional revenue" audited
Region,Revenue
East,100
West,80
```

```glossary
ARR: Annual recurring revenue.
```

[INDEX]
"#
    .to_string()
}

mod citation_tests;
mod compiler_core_tests;
mod document_structure_tests;
mod example_fixture_tests;
mod export_conformance_tests;
mod export_option_tests;
mod external_transform_tests;
mod validation_tests;

mod table_tests;

mod transform_tests;

mod export_tests;

mod media_export_tests;
mod review_provenance_tests;

fn zip_entry_text(bytes: &[u8], path: &str) -> String {
    let cursor = Cursor::new(bytes.to_vec());
    let mut archive = ZipArchive::new(cursor).expect("zip archive");
    let mut entry = archive.by_name(path).expect("zip entry");
    let mut text = String::new();
    entry.read_to_string(&mut text).expect("zip text");
    text
}

fn zip_has_entry(bytes: &[u8], path: &str) -> bool {
    let cursor = Cursor::new(bytes.to_vec());
    let mut archive = ZipArchive::new(cursor).expect("zip archive");
    let result = archive.by_name(path).is_ok();
    result
}

fn zip_entry_count_with_prefix(bytes: &[u8], prefix: &str, suffix: &str) -> usize {
    let cursor = Cursor::new(bytes.to_vec());
    let mut archive = ZipArchive::new(cursor).expect("zip archive");
    (0..archive.len())
        .filter(|index| {
            let entry = archive.by_index(*index).expect("zip entry by index");
            entry.name().starts_with(prefix) && entry.name().ends_with(suffix)
        })
        .count()
}

fn zip_entry_texts_with_prefix(bytes: &[u8], prefix: &str) -> Vec<String> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut archive = ZipArchive::new(cursor).expect("zip archive");
    let mut entries = Vec::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).expect("zip entry by index");
        if !entry.name().starts_with(prefix) || !entry.name().ends_with(".xml") {
            continue;
        }
        let mut text = String::new();
        entry.read_to_string(&mut text).expect("zip text");
        entries.push(text);
    }
    entries
}

mod export_command_tests;

mod ai_cleanup_tests;
mod file_command_tests;
mod ipc_command_tests;
