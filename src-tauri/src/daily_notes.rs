use serde::Serialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct DailyNoteResult {
    pub path: String,
    pub created: bool,
}

fn adjacent_date(date: &str, delta_days: i64) -> Option<String> {
    // Simple date arithmetic for YYYY-MM-DD format
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;

    // Days in each month (non-leap-year safe approximation)
    let days_in_month = |y: i32, m: u32| -> u32 {
        match m {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    };

    let mut d = day as i64 + delta_days;
    let mut m = month as i64;
    let mut y = year as i64;

    while d < 1 {
        m -= 1;
        if m < 1 {
            m = 12;
            y -= 1;
        }
        d += days_in_month(y as i32, m as u32) as i64;
    }
    loop {
        let dim = days_in_month(y as i32, m as u32) as i64;
        if d <= dim {
            break;
        }
        d -= dim;
        m += 1;
        if m > 12 {
            m = 1;
            y += 1;
        }
    }

    Some(format!("{y:04}-{m:02}-{d:02}"))
}

fn default_template(date: &str) -> String {
    let prev = adjacent_date(date, -1).unwrap_or_default();
    let next = adjacent_date(date, 1).unwrap_or_default();

    format!(
        r#"---
title: Journal — {date}
date: {date}
type: daily-note
tags: [journal]
previous: "[[{prev}]]"
next: "[[{next}]]"
---

# Journal — {date}

## Today


## Notes


## Tasks
- [ ]

"#
    )
}

#[tauri::command]
pub(crate) fn open_daily_note(
    workspace_root: String,
    date: String,
) -> Result<DailyNoteResult, String> {
    // Validate date format YYYY-MM-DD with range checks
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return Err(format!("Invalid date format: {date}. Expected YYYY-MM-DD."));
    }
    let (year, month, day) = (
        parts[0]
            .parse::<u32>()
            .map_err(|_| format!("Invalid year in date: {date}"))?,
        parts[1]
            .parse::<u32>()
            .map_err(|_| format!("Invalid month in date: {date}"))?,
        parts[2]
            .parse::<u32>()
            .map_err(|_| format!("Invalid day in date: {date}"))?,
    );
    let days_in_month = |y: u32, m: u32| -> u32 {
        match m {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        }
    };
    if !(1900..=2200).contains(&year)
        || !(1..=12).contains(&month)
        || day < 1
        || day > days_in_month(year, month)
    {
        return Err(format!("Date out of valid range: {date}"));
    }

    let root = PathBuf::from(&workspace_root);
    if !root.exists() {
        return Err(format!("Workspace root does not exist: {workspace_root}"));
    }

    let notes_dir = root.join("daily-notes");
    if !notes_dir.exists() {
        fs::create_dir_all(&notes_dir)
            .map_err(|e| format!("Failed to create daily-notes directory: {e}"))?;
    }

    let file_name = format!("{date}.md");
    let note_path = notes_dir.join(&file_name);
    let created = !note_path.exists();

    if created {
        let content = default_template(&date);
        fs::write(&note_path, content).map_err(|e| format!("Failed to create note: {e}"))?;
    }

    let path_str = note_path.to_string_lossy().to_string();
    Ok(DailyNoteResult {
        path: path_str,
        created,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct DailyNotesMonth {
    pub dates: Vec<String>,
}

#[tauri::command]
pub(crate) fn list_daily_notes(
    workspace_root: String,
    year: u16,
    month: u8,
) -> Result<DailyNotesMonth, String> {
    let root = PathBuf::from(&workspace_root);
    let notes_dir = root.join("daily-notes");

    if !notes_dir.exists() {
        return Ok(DailyNotesMonth { dates: vec![] });
    }

    let prefix = format!("{year:04}-{month:02}-");
    let entries = fs::read_dir(&notes_dir).map_err(|e| e.to_string())?;

    let mut dates: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name();
            let s = name.to_string_lossy();
            // Use rsplit_once to handle exactly one .md suffix
            let stem = s
                .rsplit_once(".md")
                .filter(|(_, rest)| rest.is_empty())
                .map(|(s, _)| s)?;
            if !stem.starts_with(&prefix) {
                return None;
            }
            // Validate the stripped name is exactly YYYY-MM-DD
            let valid = stem.len() == 10 && {
                let b = stem.as_bytes();
                b[4] == b'-'
                    && b[7] == b'-'
                    && b[..4].iter().all(|c| c.is_ascii_digit())
                    && b[5..7].iter().all(|c| c.is_ascii_digit())
                    && b[8..10].iter().all(|c| c.is_ascii_digit())
            };
            if valid {
                Some(stem.to_string())
            } else {
                None
            }
        })
        .collect();

    dates.sort();
    Ok(DailyNotesMonth { dates })
}
